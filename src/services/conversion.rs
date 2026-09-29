//! Local media conversion queue using FFmpeg.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::models::{
    ConversionId, ConversionProfile, ConversionStatus, ConversionTask,
};
use crate::services::FfmpegService;

pub struct ConversionService {
    tasks: Arc<Mutex<HashMap<ConversionId, ConversionTask>>>,
    running: Arc<Mutex<HashMap<ConversionId, u32>>>,
    ffmpeg: FfmpegService,
    output_dir: PathBuf,
}

impl Default for ConversionService {
    fn default() -> Self {
        let output_dir = dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("VideoDownloader")
            .join("converted");
        let _ = std::fs::create_dir_all(&output_dir);
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(HashMap::new())),
            ffmpeg: FfmpegService::default(),
            output_dir,
        }
    }
}

impl ConversionService {
    pub fn new(output_dir: PathBuf, ffmpeg: FfmpegService) -> Self {
        let _ = std::fs::create_dir_all(&output_dir);
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(HashMap::new())),
            ffmpeg,
            output_dir,
        }
    }

    pub fn output_dir(&self) -> &PathBuf {
        &self.output_dir
    }

    pub fn enqueue(&self, input_path: impl Into<String>, profile: ConversionProfile) -> ConversionId {
        self.enqueue_with_options(input_path, profile, true)
    }

    pub fn enqueue_with_options(
        &self,
        input_path: impl Into<String>,
        profile: ConversionProfile,
        keep_original: bool,
    ) -> ConversionId {
        let task = ConversionTask::new_with_options(input_path, profile, keep_original);
        let id = task.id;
        if let Ok(mut map) = self.tasks.lock() {
            map.insert(id, task);
        }
        self.spawn(id);
        id
    }

    pub fn list(&self) -> Vec<ConversionTask> {
        self.tasks
            .lock()
            .map(|m| {
                let mut v: Vec<_> = m.values().cloned().collect();
                v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                v
            })
            .unwrap_or_default()
    }

    pub fn cancel(&self, id: ConversionId) {
        if let Ok(mut map) = self.tasks.lock() {
            if let Some(t) = map.get_mut(&id) {
                if !t.status.is_terminal() {
                    t.status = ConversionStatus::Cancelled;
                }
            }
        }

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
    }

    pub fn retry(&self, id: ConversionId) {
        if let Ok(mut map) = self.tasks.lock() {
            if let Some(t) = map.get_mut(&id) {
                if t.status.is_terminal() {
                    t.status = ConversionStatus::Queued;
                    t.progress = 0.0;
                    t.error = None;
                }
            }
        }
        self.spawn(id);
    }

    pub fn remove(&self, id: ConversionId) {
        self.cancel(id);
        if let Ok(mut map) = self.tasks.lock() {
            map.remove(&id);
        }
    }

    pub fn clear_completed(&self) {
        if let Ok(mut map) = self.tasks.lock() {
            map.retain(|_, t| !t.status.is_terminal());
        }
    }

    fn spawn(&self, id: ConversionId) {
        let tasks = Arc::clone(&self.tasks);
        let running = Arc::clone(&self.running);
        let output_dir = self.output_dir.clone();
        let binary = self.ffmpeg.binary().to_path_buf();

        thread::spawn(move || {
            let (input, profile, keep_original) = {
                let mut map = match tasks.lock() {
                    Ok(m) => m,
                    Err(_) => return,
                };
                let task = match map.get_mut(&id) {
                    Some(t) => t,
                    None => return,
                };
                if task.status == ConversionStatus::Cancelled {
                    return;
                }
                task.status = ConversionStatus::Running;
                (task.input_path.clone(), task.profile, task.keep_original)
            };

            let input_path = PathBuf::from(&input);
            let stem = input_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            let out_path = output_dir.join(format!("{stem}.{}", profile.output_ext()));

            let args = build_ffmpeg_args(&input_path, &out_path, profile);

            let mut cmd = Command::new(&binary);
            cmd.args(&args)
                .stdout(Stdio::null())
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
                            t.status = ConversionStatus::Failed;
                            t.error = Some(format!("Failed to spawn ffmpeg: {e}"));
                        }
                    }
                    return;
                }
            };

            let pid = child.id();
            if let Ok(mut r) = running.lock() {
                r.insert(id, pid);
            }

            let status = child.wait();
            let _ = running.lock().map(|mut r| r.remove(&id));

            if let Ok(mut map) = tasks.lock() {
                if let Some(t) = map.get_mut(&id) {
                    if t.status == ConversionStatus::Cancelled {
                        return;
                    }
                    match status {
                        Ok(s) if s.success() => {
                            t.status = ConversionStatus::Completed;
                            t.progress = 1.0;
                            t.output_path = Some(out_path.to_string_lossy().to_string());

                            if !keep_original && input_path.exists() {
                                let _ = std::fs::remove_file(&input_path);
                            }
                        }
                        Ok(s) => {
                            t.status = ConversionStatus::Failed;
                            t.error = Some(format!("ffmpeg exited with code {:?}", s.code()));
                        }
                        Err(e) => {
                            t.status = ConversionStatus::Failed;
                            t.error = Some(format!("ffmpeg wait error: {e}"));
                        }
                    }
                }
            }
        });
    }
}

fn build_ffmpeg_args(input: &Path, output: &Path, profile: ConversionProfile) -> Vec<String> {
    let in_s = input.to_string_lossy().to_string();
    let out_s = output.to_string_lossy().to_string();

    match profile {
        ConversionProfile::RemuxCopy
        | ConversionProfile::RemuxMp4
        | ConversionProfile::RemuxMkv
        | ConversionProfile::RemuxWebm => {
            vec![
                "-y".into(),
                "-i".into(),
                in_s,
                "-c".into(),
                "copy".into(),
                out_s,
            ]
        }
        ConversionProfile::H264AacMp4 => {
            vec![
                "-y".into(),
                "-i".into(),
                in_s,
                "-c:v".into(),
                "libx264".into(),
                "-preset".into(),
                "medium".into(),
                "-crf".into(),
                "23".into(),
                "-c:a".into(),
                "aac".into(),
                "-b:a".into(),
                "192k".into(),
                out_s,
            ]
        }
        ConversionProfile::H265AacMkv => {
            vec![
                "-y".into(),
                "-i".into(),
                in_s,
                "-c:v".into(),
                "libx265".into(),
                "-preset".into(),
                "medium".into(),
                "-crf".into(),
                "28".into(),
                "-c:a".into(),
                "aac".into(),
                "-b:a".into(),
                "192k".into(),
                out_s,
            ]
        }
        ConversionProfile::Vp9OpusWebm => {
            vec![
                "-y".into(),
                "-i".into(),
                in_s,
                "-c:v".into(),
                "libvpx-vp9".into(),
                "-crf".into(),
                "30".into(),
                "-b:v".into(),
                "0".into(),
                "-c:a".into(),
                "libopus".into(),
                "-b:a".into(),
                "128k".into(),
                out_s,
            ]
        }
        ConversionProfile::AudioMp3 => {
            vec![
                "-y".into(),
                "-i".into(),
                in_s,
                "-vn".into(),
                "-c:a".into(),
                "libmp3lame".into(),
                "-b:a".into(),
                "320k".into(),
                out_s,
            ]
        }
        ConversionProfile::AudioM4a => {
            vec![
                "-y".into(),
                "-i".into(),
                in_s,
                "-vn".into(),
                "-c:a".into(),
                "aac".into(),
                "-b:a".into(),
                "256k".into(),
                out_s,
            ]
        }
        ConversionProfile::AudioAac => {
            vec![
                "-y".into(),
                "-i".into(),
                in_s,
                "-vn".into(),
                "-c:a".into(),
                "aac".into(),
                "-b:a".into(),
                "256k".into(),
                out_s,
            ]
        }
        ConversionProfile::AudioFlac => {
            vec![
                "-y".into(),
                "-i".into(),
                in_s,
                "-vn".into(),
                "-c:a".into(),
                "flac".into(),
                out_s,
            ]
        }
        ConversionProfile::AudioOpus => {
            vec![
                "-y".into(),
                "-i".into(),
                in_s,
                "-vn".into(),
                "-c:a".into(),
                "libopus".into(),
                "-b:a".into(),
                "192k".into(),
                out_s,
            ]
        }
        ConversionProfile::AudioWav => {
            vec![
                "-y".into(),
                "-i".into(),
                in_s,
                "-vn".into(),
                "-c:a".into(),
                "pcm_s16le".into(),
                out_s,
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enqueue_creates_task() {
        let svc = ConversionService::default();
        let id = svc.enqueue("/tmp/fake.mp4", ConversionProfile::RemuxCopy);
        let list = svc.list();
        assert!(list.iter().any(|t| t.id == id));
    }
}
