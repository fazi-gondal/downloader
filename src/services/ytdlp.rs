//! yt-dlp CLI wrapper.
//!
//! Spawns `yt-dlp` as a subprocess, parses JSON metadata, and later streams
//! progress. All heavy work must be done off the UI thread via `cx.spawn` /
//! `background_spawn`.

use std::path::PathBuf;
use std::process::Stdio;

use serde_json::Value;

use crate::core::{AppError, Result};
use crate::models::{
    AudioTrack, FormatInfo, MediaInfo, PlaylistEntry, PlaylistInfo, StreamType, SubtitleTrack,
};
use crate::services::NetworkOptions;

/// Service that talks to the yt-dlp binary.
#[derive(Debug, Clone)]
pub struct YtDlpService {
    binary: PathBuf,
    network: NetworkOptions,
}

impl Default for YtDlpService {
    fn default() -> Self {
        Self {
            binary: PathBuf::from("yt-dlp"),
            network: NetworkOptions::default(),
        }
    }
}

impl YtDlpService {
    pub fn new(binary: impl Into<PathBuf>) -> Self {
        Self {
            binary: binary.into(),
            network: NetworkOptions::default(),
        }
    }

    pub fn with_network(mut self, network: NetworkOptions) -> Self {
        self.network = network;
        self
    }

    pub fn set_network(&mut self, network: NetworkOptions) {
        self.network = network;
    }

    /// Extract media or playlist info without downloading.
    /// Blocking — call only from a background task.
    pub fn extract_info(&self, url: &str) -> Result<ExtractedInfo> {
        let mut args = vec![
            "--dump-single-json".into(),
            "--no-playlist".into(),
            "--no-warnings".into(),
            "--no-call-home".into(),
        ];
        self.network.append_cli_args(&mut args);
        args.push(url.to_string());

        let output = std::process::Command::new(&self.binary)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| AppError::ytdlp(format!("failed to spawn yt-dlp: {e}")))?;

        if !output.status.success() {
            return self.extract_info_allow_playlist(url);
        }

        let json: Value = serde_json::from_slice(&output.stdout)?;
        Self::parse_info(json)
    }

    fn extract_info_allow_playlist(&self, url: &str) -> Result<ExtractedInfo> {
        let mut args = vec![
            "--dump-single-json".into(),
            "--yes-playlist".into(),
            "--no-warnings".into(),
            "--no-call-home".into(),
            "--flat-playlist".into(),
        ];
        self.network.append_cli_args(&mut args);
        args.push(url.to_string());

        let output = std::process::Command::new(&self.binary)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| AppError::ytdlp(format!("failed to spawn yt-dlp: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::ytdlp(stderr.trim().to_string()));
        }

        let json: Value = serde_json::from_slice(&output.stdout)?;
        Self::parse_info(json)
    }

    fn parse_info(json: Value) -> Result<ExtractedInfo> {
        let entry_type = json
            .get("_type")
            .and_then(|v| v.as_str())
            .unwrap_or("video");

        if entry_type == "playlist" {
            Ok(ExtractedInfo::Playlist(Self::parse_playlist(&json)?))
        } else {
            Ok(ExtractedInfo::Single(Self::parse_media(&json)?))
        }
    }

    fn parse_media(json: &Value) -> Result<MediaInfo> {
        let id = json
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let title = json
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown title")
            .to_string();

        let formats = json
            .get("formats")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(Self::parse_format).collect::<Vec<_>>())
            .unwrap_or_default();

        let subtitles = Self::parse_subtitles(json);
        let audio_tracks = Self::derive_audio_tracks(json);

        Ok(MediaInfo {
            id,
            title,
            uploader: json
                .get("uploader")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            duration: json.get("duration").and_then(|v| v.as_f64()),
            thumbnail: json
                .get("thumbnail")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            webpage_url: json
                .get("webpage_url")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            formats,
            audio_tracks,
            subtitles,
            description: json
                .get("description")
                .and_then(|v| v.as_str())
                .map(str::to_string),
        })
    }

    fn parse_playlist(json: &Value) -> Result<PlaylistInfo> {
        let id = json
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let title = json
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Playlist")
            .to_string();

        let entries = json
            .get("entries")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|e| {
                        let id = e.get("id")?.as_str()?.to_string();
                        let title = e
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Untitled")
                            .to_string();
                        let url = e
                            .get("url")
                            .or_else(|| e.get("webpage_url"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if url.is_empty() {
                            return None;
                        }
                        Some(PlaylistEntry {
                            id,
                            title,
                            url,
                            duration: e.get("duration").and_then(|v| v.as_f64()),
                            thumbnail: e
                                .get("thumbnail")
                                .and_then(|v| v.as_str())
                                .map(str::to_string),
                            selected: true,
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Ok(PlaylistInfo {
            id,
            title,
            uploader: json
                .get("uploader")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            entries,
            webpage_url: json
                .get("webpage_url")
                .and_then(|v| v.as_str())
                .map(str::to_string),
        })
    }

    fn parse_format(f: &Value) -> Option<FormatInfo> {
        let format_id = f.get("format_id")?.as_str()?.to_string();
        let ext = f
            .get("ext")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let vcodec = f.get("vcodec").and_then(|v| v.as_str()).map(str::to_string);
        let acodec = f.get("acodec").and_then(|v| v.as_str()).map(str::to_string);

        let stream_type = classify_stream(vcodec.as_deref(), acodec.as_deref(), f);

        let (filesize, filesize_is_approx) = match f.get("filesize").and_then(|v| v.as_u64()) {
            Some(s) => (Some(s), false),
            None => (
                f.get("filesize_approx").and_then(|v| v.as_u64()),
                true,
            ),
        };

        Some(FormatInfo {
            format_id,
            ext,
            resolution: f
                .get("resolution")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            height: f.get("height").and_then(|v| v.as_u64()).map(|h| h as u32),
            fps: f.get("fps").and_then(|v| v.as_f64()),
            vcodec: if stream_type != StreamType::AudioOnly {
                vcodec
            } else {
                None
            },
            acodec: if stream_type != StreamType::VideoOnly {
                acodec
            } else {
                None
            },
            tbr: f.get("tbr").and_then(|v| v.as_f64()),
            abr: f.get("abr").and_then(|v| v.as_f64()),
            filesize,
            filesize_is_approx,
            stream_type,
            format_note: f
                .get("format_note")
                .and_then(|v| v.as_str())
                .map(str::to_string),
        })
    }

    fn parse_subtitles(json: &Value) -> Vec<SubtitleTrack> {
        let mut tracks = Vec::new();
        // ONLY parse uploaded subtitles ("subtitles"), NEVER "automatic_captions"
        if let Some(subs) = json.get("subtitles").and_then(|v| v.as_object()) {
            for (lang, entries) in subs {
                let name = entries
                    .as_array()
                    .and_then(|arr| arr.first())
                    .and_then(|e| e.get("name"))
                    .and_then(|n| n.as_str())
                    .map(str::to_string);

                tracks.push(SubtitleTrack {
                    language: lang.clone(),
                    name,
                    is_automatic: false,
                });
            }
        }
        tracks.sort_by(|a, b| a.language.cmp(&b.language));
        tracks
    }

    fn derive_audio_tracks(json: &Value) -> Vec<AudioTrack> {
        let formats = match json.get("formats").and_then(|v| v.as_array()) {
            Some(f) => f,
            None => return Vec::new(),
        };

        struct TrackCandidate {
            format_id: String,
            language: Option<String>,
            track_name: String,
            abr: f64,
            ext: String,
            acodec: String,
            note: Option<String>,
            is_opus: bool,
            asr: Option<i64>,
            audio_channels: Option<i64>,
        }

        let mut track_best: std::collections::BTreeMap<String, TrackCandidate> = std::collections::BTreeMap::new();

        for f in formats {
            let vcodec = f.get("vcodec").and_then(|v| v.as_str());
            let acodec_val = f.get("acodec").and_then(|v| v.as_str()).unwrap_or("");
            let resolution = f.get("resolution").and_then(|v| v.as_str()).unwrap_or("");
            let height = f.get("height").and_then(|v| v.as_u64());

            let is_audio_only = vcodec == Some("none")
                || resolution == "audio only"
                || (vcodec.is_none() && height.is_none() && !acodec_val.is_empty() && acodec_val != "none");

            if !is_audio_only {
                continue;
            }

            let fid = f
                .get("format_id")
                .and_then(|v| {
                    if let Some(s) = v.as_str() {
                        Some(s.to_string())
                    } else if let Some(n) = v.as_i64() {
                        Some(n.to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            if fid.is_empty() {
                continue;
            }

            let ext = f.get("ext").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if ext == "mhtml" {
                continue;
            }

            let acodec = acodec_val.to_string();
            let acodec_base = acodec.split('.').next().unwrap_or(&acodec).to_lowercase();
            let is_opus = ext == "webm" || acodec_base == "opus";

            let lang = f
                .get("language")
                .and_then(|v| v.as_str())
                .or_else(|| f.get("language_preference").and_then(|v| v.as_str()))
                .unwrap_or("");

            let note = f.get("format_note").and_then(|v| v.as_str()).unwrap_or("");
            let lang_name = f.get("language_name").and_then(|v| v.as_str()).unwrap_or("");

            let track_name = clean_track_name(lang, note, lang_name);

            let track_key = if !lang.is_empty() && !track_name.is_empty() {
                format!("{lang}_{track_name}")
            } else if !lang.is_empty() {
                lang.to_string()
            } else if !track_name.is_empty() {
                track_name.clone()
            } else {
                fid.clone()
            };

            let abr = f
                .get("abr")
                .and_then(|v| v.as_f64())
                .or_else(|| f.get("tbr").and_then(|v| v.as_f64()))
                .unwrap_or(0.0);

            let asr = f.get("asr").and_then(|v| v.as_i64());
            let audio_channels = f.get("audio_channels").and_then(|v| v.as_i64());

            let replace = match track_best.get(&track_key) {
                None => true,
                Some(existing) => {
                    if is_opus && !existing.is_opus {
                        true
                    } else if is_opus == existing.is_opus && abr > existing.abr {
                        true
                    } else {
                        false
                    }
                }
            };

            if replace {
                track_best.insert(
                    track_key,
                    TrackCandidate {
                        format_id: fid,
                        language: if lang.is_empty() { None } else { Some(lang.to_string()) },
                        track_name,
                        abr,
                        ext,
                        acodec,
                        note: if note.is_empty() { None } else { Some(note.to_string()) },
                        is_opus,
                        asr,
                        audio_channels,
                    },
                );
            }
        }

        let mut tracks: Vec<AudioTrack> = track_best
            .into_values()
            .map(|best| {
                let mut parts = vec![best.track_name.clone()];
                let acodec_base = best.acodec.split('.').next().unwrap_or(&best.acodec).to_lowercase();

                if !best.ext.is_empty() && !acodec_base.is_empty() && acodec_base != "none" && acodec_base != best.ext {
                    parts.push(format!("{} ({})", best.ext, acodec_base));
                } else if !best.ext.is_empty() {
                    parts.push(best.ext.clone());
                }

                if best.abr > 0.0 {
                    parts.push(format!("{} kbps", best.abr.round() as u64));
                }
                if let Some(ch) = best.audio_channels {
                    if ch > 2 {
                        parts.push(format!("{ch}ch"));
                    }
                }
                if let Some(asr) = best.asr {
                    if asr >= 1000 {
                        parts.push(format!("{}kHz", asr / 1000));
                    } else {
                        parts.push(format!("{asr}Hz"));
                    }
                }
                parts.push(format!("[id: {}]", best.format_id));
                let label = parts.join(" · ");

                AudioTrack {
                    language: best.language,
                    title: Some(best.track_name),
                    format_id: best.format_id,
                    abr: if best.abr > 0.0 { Some(best.abr) } else { None },
                    ext: if best.ext.is_empty() { None } else { Some(best.ext) },
                    acodec: if best.acodec.is_empty() { None } else { Some(best.acodec) },
                    label: Some(label),
                    note: best.note,
                }
            })
            .collect();

        // Sort: tracks with language first, then by descending abr
        tracks.sort_by(|a, b| {
            let a_has_lang = a.language.is_some();
            let b_has_lang = b.language.is_some();
            b_has_lang
                .cmp(&a_has_lang)
                .then_with(|| {
                    let a_abr = a.abr.unwrap_or(0.0);
                    let b_abr = b.abr.unwrap_or(0.0);
                    b_abr.partial_cmp(&a_abr).unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        tracks
    }

    pub fn check_available(&self) -> Result<String> {
        let output = std::process::Command::new(&self.binary)
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| AppError::ytdlp(format!("yt-dlp not found: {e}")))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(AppError::ytdlp("yt-dlp --version failed".to_string()))
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExtractedInfo {
    Single(MediaInfo),
    Playlist(PlaylistInfo),
}

fn classify_stream(vcodec: Option<&str>, acodec: Option<&str>, f: &Value) -> StreamType {
    let v_none = vcodec.map(|c| c == "none").unwrap_or(false);
    let a_none = acodec.map(|c| c == "none").unwrap_or(false);
    let has_height = f.get("height").and_then(|v| v.as_u64()).is_some()
        || f.get("resolution")
            .and_then(|v| v.as_str())
            .map(|r| r != "audio only")
            .unwrap_or(false);

    if v_none {
        StreamType::AudioOnly
    } else if a_none {
        StreamType::VideoOnly
    } else if vcodec.is_some() || has_height {
        StreamType::Muxed
    } else {
        StreamType::AudioOnly
    }
}

fn common_lang_name(code: &str) -> &'static str {
    match code.to_lowercase().as_str() {
        "en" => "English",
        "de" => "German",
        "fr" => "French",
        "es" => "Spanish",
        "it" => "Italian",
        "pt" => "Portuguese",
        "ru" => "Russian",
        "ja" => "Japanese",
        "ko" => "Korean",
        "zh" => "Chinese",
        "zh-hans" => "Chinese (Simplified)",
        "zh-hant" => "Chinese (Traditional)",
        "hi" => "Hindi",
        "ar" => "Arabic",
        "bn" => "Bangla",
        "tr" => "Turkish",
        "pl" => "Polish",
        "vi" => "Vietnamese",
        "id" => "Indonesian",
        "th" => "Thai",
        "ta" => "Tamil",
        "te" => "Telugu",
        "ml" => "Malayalam",
        "ur" => "Urdu",
        "nl" => "Dutch",
        "sv" => "Swedish",
        "no" => "Norwegian",
        "da" => "Danish",
        "fi" => "Finnish",
        "cs" => "Czech",
        "el" => "Greek",
        "he" => "Hebrew",
        "hu" => "Hungarian",
        "ro" => "Romanian",
        "uk" => "Ukrainian",
        _ => "",
    }
}

fn clean_track_name(l_code: &str, f_note: &str, l_name: &str) -> String {
    let mut clean = f_note.to_string();
    for suffix in [", low", ", medium", ", high", ", ultralow", ", drc", ", DRC", "low", "medium", "high", "drc", "DRC"] {
        clean = clean.replace(suffix, "");
    }
    let lower = clean.to_lowercase();
    if lower.contains("original (default)") || lower.contains("original(default)") {
        clean = "Original Audio".to_string();
    } else if lower.trim() == "original" || lower.trim() == "default" {
        clean = "Original Audio".to_string();
    }
    let trimmed = clean.trim().to_string();
    if !trimmed.is_empty() && trimmed.to_lowercase() != "drc" {
        return trimmed;
    }
    if !l_name.trim().is_empty() {
        return l_name.trim().to_string();
    }
    let mapped = common_lang_name(l_code);
    if !mapped.is_empty() {
        return mapped.to_string();
    }
    if !l_code.trim().is_empty() {
        return l_code.trim().to_string();
    }
    "Audio".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_subtitles_ignores_automatic_captions() {
        let payload = json!({
            "subtitles": {
                "en": [{"ext": "vtt", "name": "English"}],
                "es": [{"ext": "vtt", "name": "Spanish"}]
            },
            "automatic_captions": {
                "fr": [{"ext": "vtt", "name": "French (auto)"}],
                "de": [{"ext": "vtt", "name": "German (auto)"}]
            }
        });

        let subs = YtDlpService::parse_subtitles(&payload);
        assert_eq!(subs.len(), 2);
        assert_eq!(subs[0].language, "en");
        assert_eq!(subs[0].name.as_deref(), Some("English"));
        assert_eq!(subs[0].display_label(), "English (en)");
        assert_eq!(subs[1].language, "es");
        assert!(!subs.iter().any(|s| s.language == "fr"));
    }

    #[test]
    fn derive_audio_tracks_parses_clean_labels_and_prefers_opus() {
        let payload = json!({
            "formats": [
                {
                    "format_id": "140",
                    "ext": "m4a",
                    "acodec": "mp4a.40.2",
                    "vcodec": "none",
                    "abr": 128.0,
                    "language": "en",
                    "format_note": "medium, original (default)"
                },
                {
                    "format_id": "251",
                    "ext": "webm",
                    "acodec": "opus",
                    "vcodec": "none",
                    "abr": 160.0,
                    "language": "en",
                    "format_note": "medium, original (default)"
                }
            ]
        });

        let tracks = YtDlpService::derive_audio_tracks(&payload);
        assert_eq!(tracks.len(), 1);
        let track = &tracks[0];
        assert_eq!(track.format_id, "251");
        assert!(track.display_label().contains("Original Audio"));
        assert!(track.display_label().contains("webm (opus)"));
        assert!(track.display_label().contains("160 kbps"));
        assert!(track.display_label().contains("[id: 251]"));
    }
}
