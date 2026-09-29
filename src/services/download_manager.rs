//! Download queue, concurrency control, progress parsing, and cancellation.
//!
//! yt-dlp is spawned as a subprocess. Progress lines are parsed and written
//! back into the shared task map. UI observes via polling or future Entity
//! events. All process I/O stays off the UI thread.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::models::{DownloadId, DownloadOptions, DownloadStatus, DownloadTask, MediaInfo};
use crate::services::{FormatBuilder, NetworkOptions};

const DEFAULT_MAX_CONCURRENT: usize = 8;

pub fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other => other,
        })
        .collect();
    let trimmed = sanitized.trim();
    if trimmed.is_empty() {
        "playlist".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Internal handle so we can kill a running process on cancel.
pub struct DownloadManager {
    tasks: Arc<Mutex<HashMap<DownloadId, DownloadTask>>>,
    running: Arc<Mutex<HashMap<DownloadId, u32>>>,
    max_concurrent: usize,
    download_dir: PathBuf,
    ytdlp_binary: PathBuf,
    network: NetworkOptions,
}

impl Default for DownloadManager {
    fn default() -> Self {
        let download_dir = dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("VideoDownloader");
        let _ = std::fs::create_dir_all(&download_dir);
        Self::new(DEFAULT_MAX_CONCURRENT, download_dir, PathBuf::from("yt-dlp"))
    }
}

impl DownloadManager {
    pub fn new(max_concurrent: usize, download_dir: PathBuf, ytdlp_binary: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&download_dir);
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(HashMap::<DownloadId, u32>::new())),
            max_concurrent: max_concurrent.max(1).min(16),
            download_dir,
            ytdlp_binary,
            network: NetworkOptions::default(),
        }
    }

    pub fn set_network(&mut self, network: NetworkOptions) {
        self.network = network;
    }

    pub fn set_max_concurrent(&mut self, n: usize) {
        self.max_concurrent = n.max(1).min(16);
    }

    pub fn set_download_dir(&mut self, dir: PathBuf) {
        let _ = std::fs::create_dir_all(&dir);
        self.download_dir = dir;
    }

    pub fn download_dir(&self) -> &PathBuf {
        &self.download_dir
    }

    /// Enqueue a new download. Returns the task id.
    /// If a MediaInfo is supplied, title and size estimates are filled in.
    pub fn enqueue(
        &self,
        url: impl Into<String>,
        options: DownloadOptions,
        media: Option<&MediaInfo>,
    ) -> DownloadId {
        let mut task = DownloadTask::new(url, options.clone());
        if let Some(m) = media {
            task.title = m.title.clone();
            task.media = Some(m.clone());
            if let Some(size) = FormatBuilder::compile(m, &options).estimated_size {
                task.total_bytes = Some(size);
            }
        }

        let id = task.id;
        if let Ok(mut map) = self.tasks.lock() {
            map.insert(id, task);
        }

        self.try_start_next();
        id
    }

    /// Enqueue a playlist item with known title and playlist title.
    pub fn enqueue_playlist_item(
        &self,
        url: impl Into<String>,
        title: String,
        playlist_title: Option<String>,
        options: DownloadOptions,
    ) -> DownloadId {
        let mut task = DownloadTask::new(url, options);
        task.title = title;
        task.playlist_title = playlist_title;

        let id = task.id;
        if let Ok(mut map) = self.tasks.lock() {
            map.insert(id, task);
        }

        self.try_start_next();
        id
    }

    pub fn cancel(&self, id: DownloadId) {
        let maybe_out = if let Ok(mut map) = self.tasks.lock() {
            if let Some(task) = map.get_mut(&id) {
                if !task.status.is_terminal() {
                    task.status = DownloadStatus::Cancelled;
                }
                task.output_path.clone()
            } else {
                None
            }
        } else {
            None
        };

        // Terminate running child process tree
        if let Ok(mut running) = self.running.lock() {
            if let Some(pid) = running.remove(&id) {
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    const CREATE_NO_WINDOW: u32 = 0x08000000;
                    let _ = Command::new("taskkill")
                        .args(["/PID", &pid.to_string(), "/F", "/T"])
                        .creation_flags(CREATE_NO_WINDOW)
                        .output();
                }
                #[cfg(not(windows))]
                {
                    let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
                }
            }
        }

        // Clean up partial files (.part, .ytdl) if output path was identified
        if let Some(out) = maybe_out {
            let p = PathBuf::from(out);
            let part = p.with_extension(format!("{}.part", p.extension().unwrap_or_default().to_string_lossy()));
            let ytdl = p.with_extension(format!("{}.ytdl", p.extension().unwrap_or_default().to_string_lossy()));
            let _ = std::fs::remove_file(part);
            let _ = std::fs::remove_file(ytdl);
        }

        self.try_start_next();
    }

    /// Retry a failed or cancelled task.
    pub fn retry(&self, id: DownloadId) {
        if let Ok(mut map) = self.tasks.lock() {
            if let Some(task) = map.get_mut(&id) {
                if task.status.is_terminal() {
                    task.status = DownloadStatus::Queued;
                    task.progress = 0.0;
                    task.speed = None;
                    task.eta = None;
                    task.downloaded_bytes = 0;
                    task.error = None;
                }
            }
        }
        self.try_start_next();
    }

    /// Remove a task completely from the manager.
    pub fn remove(&self, id: DownloadId) {
        self.cancel(id);
        if let Ok(mut map) = self.tasks.lock() {
            map.remove(&id);
        }
    }

    /// Remove all terminal (completed, failed, cancelled) tasks.
    pub fn clear_completed(&self) {
        if let Ok(mut map) = self.tasks.lock() {
            map.retain(|_, task| !task.status.is_terminal());
        }
    }

    /// Cancel all non-terminal tasks.
    pub fn cancel_all(&self) {
        let ids: Vec<DownloadId> = if let Ok(map) = self.tasks.lock() {
            map.iter()
                .filter(|(_, t)| !t.status.is_terminal())
                .map(|(&id, _)| id)
                .collect()
        } else {
            Vec::new()
        };
        for id in ids {
            self.cancel(id);
        }
    }

    pub fn list(&self) -> Vec<DownloadTask> {
        self.tasks
            .lock()
            .map(|m| {
                let mut v: Vec<_> = m.values().cloned().collect();
                v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                v
            })
            .unwrap_or_default()
    }

    pub fn get(&self, id: DownloadId) -> Option<DownloadTask> {
        self.tasks.lock().ok()?.get(&id).cloned()
    }

    /// Active (non-terminal) count.
    pub fn active_count(&self) -> usize {
        self.tasks
            .lock()
            .map(|m| m.values().filter(|t| !t.status.is_terminal()).count())
            .unwrap_or(0)
    }

    fn running_count(&self) -> usize {
        self.running.lock().map(|m| m.len()).unwrap_or(0)
    }

    /// Start queued tasks up to max_concurrent.
    fn try_start_next(&self) {
        let slots = self
            .max_concurrent
            .saturating_sub(self.running_count());
        if slots == 0 {
            return;
        }

        let to_start: Vec<DownloadTask> = {
            let map = match self.tasks.lock() {
                Ok(m) => m,
                Err(_) => return,
            };
            map.values()
                .filter(|t| t.status == DownloadStatus::Queued)
                .take(slots)
                .cloned()
                .collect()
        };

        for task in to_start {
            self.spawn_download(task);
        }
    }

    fn spawn_download(&self, task: DownloadTask) {
        let id = task.id;
        let url = task.url.clone();
        let options = task.options.clone();
        let media = task.media.clone();
        let binary = self.ytdlp_binary.clone();
        let network = self.network.clone();

        // Determine destination folder (support playlist subfolder if set)
        let target_dir = if let Some(ref pl_title) = task.playlist_title {
            let sanitized = sanitize_filename(pl_title);
            let dir = self.download_dir.join(sanitized);
            let _ = std::fs::create_dir_all(&dir);
            dir
        } else {
            self.download_dir.clone()
        };

        // Mark as downloading
        if let Ok(mut map) = self.tasks.lock() {
            if let Some(t) = map.get_mut(&id) {
                t.status = DownloadStatus::Downloading;
            }
        }

        let tasks = Arc::clone(&self.tasks);
        let running = Arc::clone(&self.running);

        thread::spawn(move || {
            let compiled = media
                .as_ref()
                .map(|m| FormatBuilder::compile(m, &options));

            let output_template = target_dir
                .join("%(title)s [%(id)s].%(ext)s")
                .to_string_lossy()
                .to_string();

            let mut args = vec![
                "--newline".into(),
                "--progress".into(),
                "-o".into(),
                output_template,
                "--no-playlist".into(),
                "--no-warnings".into(),
                "--remote-components".into(),
                "ejs:github".into(),
                "--extractor-args".into(),
                "youtube:player_client=all".into(),
            ];

            if let Some(ref c) = compiled {
                args.push("-f".into());
                args.push(c.format_selector.clone());
                args.extend(c.extra_args.clone());
            }
            network.append_cli_args(&mut args);

            args.push(url);

            let mut cmd = Command::new(&binary);
            cmd.args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                cmd.creation_flags(CREATE_NO_WINDOW);
            }

            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    if let Ok(mut map) = tasks.lock() {
                        if let Some(t) = map.get_mut(&id) {
                            t.status = DownloadStatus::Failed;
                            t.error = Some(format!("failed to spawn yt-dlp: {e}"));
                        }
                    }
                    return;
                }
            };

            // Track process ID for cancellation
            let pid = child.id();
            if let Ok(mut r) = running.lock() {
                r.insert(id, pid);
            }

            // Parse stdout progress
            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);
                for line in reader.lines().flatten() {
                    // Cooperative cancel check
                    let cancelled = tasks
                        .lock()
                        .map(|m| {
                            m.get(&id)
                                .map(|t| t.status == DownloadStatus::Cancelled)
                                .unwrap_or(true)
                        })
                        .unwrap_or(true);
                    if cancelled {
                        let _ = child.kill();
                        break;
                    }

                    // Destination output parsing
                    if line.starts_with("[download] Destination: ") {
                        let path = line.trim_start_matches("[download] Destination: ").trim();
                        if let Ok(mut map) = tasks.lock() {
                            if let Some(t) = map.get_mut(&id) {
                                t.output_path = Some(path.to_string());
                            }
                        }
                    } else if line.starts_with("[Merger] Merging formats into ") {
                        let path = line.trim_start_matches("[Merger] Merging formats into ").trim().trim_matches('"');
                        if let Ok(mut map) = tasks.lock() {
                            if let Some(t) = map.get_mut(&id) {
                                t.output_path = Some(path.to_string());
                            }
                        }
                    } else if line.starts_with("[ExtractAudio] Destination: ") {
                        let path = line.trim_start_matches("[ExtractAudio] Destination: ").trim();
                        if let Ok(mut map) = tasks.lock() {
                            if let Some(t) = map.get_mut(&id) {
                                t.output_path = Some(path.to_string());
                            }
                        }
                    }

                    if let Some((pct, speed, eta, downloaded, total)) = parse_progress_line(&line) {
                        if let Ok(mut map) = tasks.lock() {
                            if let Some(t) = map.get_mut(&id) {
                                t.progress = pct;
                                t.speed = speed;
                                t.eta = eta;
                                if let Some(d) = downloaded {
                                    t.downloaded_bytes = d;
                                }
                                if let Some(tot) = total {
                                    t.total_bytes = Some(tot);
                                }
                            }
                        }
                    }
                }
            }

            let status = child.wait();
            let _ = running.lock().map(|mut r| r.remove(&id));

            if let Ok(mut map) = tasks.lock() {
                if let Some(t) = map.get_mut(&id) {
                    if t.status == DownloadStatus::Cancelled {
                        // already marked
                    } else if status.map(|s| s.success()).unwrap_or(false) {
                        t.status = DownloadStatus::Completed;
                        t.progress = 1.0;
                    } else {
                        t.status = DownloadStatus::Failed;
                        if t.error.is_none() {
                            t.error = Some("yt-dlp exited with error".into());
                        }
                    }
                }
            }
        });
    }
}

/// Parse a yt-dlp progress line.
/// Example: `[download]  45.2% of 12.34MiB at 1.23MiB/s ETA 00:08`
fn parse_progress_line(
    line: &str,
) -> Option<(f64, Option<String>, Option<String>, Option<u64>, Option<u64>)> {
    if !line.contains("[download]") || !line.contains('%') {
        return None;
    }

    let pct = {
        let idx = line.find('%')?;
        let start = line[..idx].rfind(' ').map(|i| i + 1).unwrap_or(0);
        line[start..idx].trim().parse::<f64>().ok()? / 100.0
    };

    let speed = line
        .split(" at ")
        .nth(1)
        .and_then(|s| s.split(" ETA ").next())
        .map(|s| s.trim().to_string());

    let eta = line
        .split(" ETA ")
        .nth(1)
        .map(|s| s.trim().to_string());

    // Best-effort size parse is left for later; progress % is the main signal.
    Some((pct, speed, eta, None, None))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DownloadOptions;
    use std::path::PathBuf;

    fn test_mgr() -> DownloadManager {
        DownloadManager::new(2, PathBuf::from("/tmp/vd-test"), PathBuf::from("yt-dlp"))
    }

    #[test]
    fn enqueue_and_list() {
        let mgr = test_mgr();
        let id = mgr.enqueue("https://example.com/v", DownloadOptions::default(), None);
        let tasks = mgr.list();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, id);
    }

    #[test]
    fn cancel_marks_cancelled() {
        let mgr = test_mgr();
        let id = mgr.enqueue("https://example.com/v", DownloadOptions::default(), None);
        // Give the spawn thread a moment; status may already be Downloading
        std::thread::sleep(std::time::Duration::from_millis(50));
        mgr.cancel(id);
        let task = mgr.get(id).unwrap();
        assert!(
            task.status == DownloadStatus::Cancelled || task.status.is_terminal(),
            "status = {:?}",
            task.status
        );
    }

    #[test]
    fn parse_progress() {
        let line = "[download]  45.2% of 12.34MiB at 1.23MiB/s ETA 00:08";
        let (pct, speed, eta, _, _) = parse_progress_line(line).unwrap();
        assert!((pct - 0.452).abs() < 0.001);
        assert!(speed.unwrap().contains("1.23"));
        assert_eq!(eta.unwrap(), "00:08");
    }
}
