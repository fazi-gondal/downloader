//! Pure functional compiler: UI selections → yt-dlp format string + post-processors.
//!
//! This module has no I/O and is fully unit-testable.

use crate::models::{
    Container, DownloadMode, DownloadOptions, FormatInfo, MediaInfo, ResolutionPreset, StreamType,
    SubtitleMode,
};

/// Result of compiling user options into yt-dlp arguments.
#[derive(Debug, Clone)]
pub struct CompiledFormat {
    /// `-f` format selector string.
    pub format_selector: String,
    /// Extra CLI arguments (merge, convert, embed, …).
    pub extra_args: Vec<String>,
    /// Expected output extension.
    pub output_ext: String,
    /// Estimated total size in bytes (best effort).
    pub estimated_size: Option<u64>,
}

pub struct FormatBuilder;

impl FormatBuilder {
    /// Compile UI options against available media formats into deterministic yt-dlp options.
    pub fn compile(media: &MediaInfo, options: &DownloadOptions) -> CompiledFormat {
        match options.mode {
            DownloadMode::AudioOnly => Self::compile_audio(media, options),
            DownloadMode::VideoOnly => Self::compile_video_only(media, options),
            DownloadMode::VideoAudio => Self::compile_video_audio(media, options),
        }
    }

    fn compile_audio(media: &MediaInfo, options: &DownloadOptions) -> CompiledFormat {
        let ext = options.audio_format.extension().to_string();
        let quality = options.audio_quality.value().to_string();
        let mut extra = vec![
            "--extract-audio".into(),
            "--audio-format".into(),
            ext.clone(),
            "--audio-quality".into(),
            quality,
        ];
        if options.embed_thumbnail {
            extra.push("--embed-thumbnail".into());
        }
        if options.embed_metadata {
            extra.push("--embed-metadata".into());
        }

        let selector = if let Some(ref aid) = options.audio_format_id {
            aid.clone()
        } else {
            "bestaudio/best".into()
        };

        let estimated = media
            .formats
            .iter()
            .filter(|f| f.stream_type == StreamType::AudioOnly || f.stream_type == StreamType::Muxed)
            .filter_map(|f| f.estimated_size(media.duration))
            .max();

        CompiledFormat {
            format_selector: selector,
            extra_args: extra,
            output_ext: ext,
            estimated_size: estimated,
        }
    }

    fn compile_video_only(media: &MediaInfo, options: &DownloadOptions) -> CompiledFormat {
        let selector = if let Some(ref vid) = options.video_format_id {
            vid.clone()
        } else {
            Self::video_selector(options.resolution, options.fps_filter, false)
        };
        let ext = options.container.extension().to_string();
        let mut extra = Vec::new();
        if options.container != Container::Mp4 {
            extra.push("--merge-output-format".into());
            extra.push(ext.clone());
        }

        let estimated = Self::estimate_video_size(media, options.resolution, false);

        CompiledFormat {
            format_selector: selector,
            extra_args: extra,
            output_ext: ext,
            estimated_size: estimated,
        }
    }

    fn compile_video_audio(media: &MediaInfo, options: &DownloadOptions) -> CompiledFormat {
        // ── Format selector ─────────────────────────────────────────────────────
        // Build the video part of the selector.
        // "bv*" (any video codec) gives yt-dlp more room to pick good formats;
        // height/fps filters are appended as attribute filters.
        let height_filter = match options.resolution.height() {
            Some(h) => format!("[height<=?{h}]"),
            None => String::new(),
        };
        let fps_filter_str = match options.fps_filter {
            Some(f) => format!("[fps<=?{f}]"),
            None => String::new(),
        };
        let filters = format!("{height_filter}{fps_filter_str}");

        let selector = if !options.selected_audio_langs.is_empty() {
            // Custom audio track selection — keys are yt-dlp format IDs
            // (e.g. "251", "140", "233-dub-en") stored from the format list.
            let video_part = if let Some(ref v) = options.video_format_id {
                v.clone()
            } else if filters.is_empty() {
                "bv*".to_string()
            } else {
                format!("bv*{filters}")
            };

            let audio_chain = options.selected_audio_langs.join("+");

            if options.selected_audio_langs.len() == 1 {
                // Single track: try the specific format ID, fall back to best audio.
                let audio = &options.selected_audio_langs[0];
                format!("{video_part}+{audio}/{video_part}+ba/b")
            } else {
                // Multiple tracks: chain all format IDs with --audio-multistreams.
                // Fallback 1: mergeall grabs every audio stream yt-dlp finds.
                // Fallback 2: single best audio so the download never fails.
                format!(
                    "{video_part}+{audio_chain}\
/{video_part}+mergeall[vcodec=none]\
/{video_part}+ba/b"
                )
            }
        } else {
            match (&options.video_format_id, &options.audio_format_id) {
                (Some(v), Some(a)) => format!("{v}+{a}"),
                (Some(v), None)    => format!("{v}+ba/b"),
                (None,    Some(a)) => {
                    let vp = if filters.is_empty() { "bv*".to_string() } else { format!("bv*{filters}") };
                    format!("{vp}+{a}")
                }
                (None, None) => {
                    // "All Audio Tracks" mode: mergeall grabs every available audio stream.
                    if options.multi_audio
                        || options.audio_tracks_mode == crate::models::AudioTracksMode::All
                    {
                        if filters.is_empty() {
                            "bv*+mergeall[vcodec=none]/b".to_string()
                        } else {
                            format!("bv*{filters}+mergeall[vcodec=none]/b{filters}/b")
                        }
                    } else if filters.is_empty() {
                        "bv*+ba/b".to_string()
                    } else {
                        format!("bv*{filters}+ba/b{filters}/b")
                    }
                }
            }
        };

        // ── Container ───────────────────────────────────────────────────────────
        // Force MKV when multiple audio tracks or subtitles are involved;
        // MP4/WebM silently drop extra streams during FFmpeg merge.
        let is_multi_audio = options.multi_audio
            || options.audio_tracks_mode == crate::models::AudioTracksMode::All
            || options.selected_audio_langs.len() > 1;
        let has_subs = options.subtitle_mode != SubtitleMode::None
            && (options.subtitle_mode == SubtitleMode::All
                || !options.subtitle_langs.is_empty());
        let effective_ext = if is_multi_audio || has_subs {
            "mkv".to_string()
        } else {
            options.container.extension().to_string()
        };

        let mut extra = vec![
            "--merge-output-format".into(),
            effective_ext.clone(),
            "--remux-video".into(),
            effective_ext.clone(),
        ];
        if options.embed_thumbnail {
            extra.push("--embed-thumbnail".into());
        }
        if options.embed_metadata {
            extra.push("--embed-metadata".into());
        }

        // ── Subtitles ───────────────────────────────────────────────────────────
        // Strategy:
        //   All  mode  → fetch uploaded + auto-generated subtitles for all languages
        //   Custom mode → fetch ONLY the exact language codes selected by the user
        //
        // IMPORTANT: Do NOT use --write-subs (permanently keeps .vtt files on disk).
        //            Do NOT use --write-auto-subs in Custom mode — it causes yt-dlp
        //            to fetch every auto-generated variant (100s of requests) which
        //            triggers HTTP 429 (Too Many Requests) from YouTube.
        //
        // With --embed-subs (and no --write-subs), yt-dlp automatically deletes the
        // intermediate .vtt files after embedding them into the video.
        match options.subtitle_mode {
            SubtitleMode::None => {}
            SubtitleMode::All => {
                // All mode: grab uploaded + auto-generated for every language.
                // Add a 1-second delay between subtitle requests to avoid 429s.
                extra.push("--embed-subs".into());
                extra.push("--write-auto-subs".into());
                extra.push("--sub-langs".into());
                extra.push("all".into());
                extra.push("--sleep-subtitles".into());
                extra.push("1".into());
            }
            SubtitleMode::Custom => {
                let clean_langs: Vec<String> = options
                    .subtitle_langs
                    .iter()
                    .filter(|l| *l != "all" && !l.trim().is_empty())
                    .cloned()
                    .collect();
                if !clean_langs.is_empty() {
                    // Custom mode: embed only the exact languages; skip auto-subs
                    // to avoid the 429 rate-limit burst that --write-auto-subs causes.
                    extra.push("--embed-subs".into());
                    extra.push("--sub-langs".into());
                    extra.push(clean_langs.join(","));
                }
            }
        }

        // ── Multi-audio streams ──────────────────────────────────────────────────
        // --audio-multistreams tells FFmpeg to keep every audio stream during merge.
        // Without it only the first audio stream survives.
        if is_multi_audio {
            extra.push("--audio-multistreams".into());
        }

        let estimated = Self::estimate_video_size(media, options.resolution, true);

        CompiledFormat {
            format_selector: selector,
            extra_args: extra,
            output_ext: effective_ext,
            estimated_size: estimated,
        }
    }

    fn video_selector(res: ResolutionPreset, fps: Option<u32>, with_audio: bool) -> String {
        let height_filter = match res.height() {
            Some(h) => format!("[height<=?{h}]"),
            None => String::new(),
        };
        let fps_filter = match fps {
            Some(f) => format!("[fps<=?{f}]"),
            None => String::new(),
        };

        if with_audio {
            // Prefer separate best video + best audio, fall back to progressive.
            format!(
                "bestvideo{height_filter}{fps_filter}+bestaudio/best{height_filter}{fps_filter}/best"
            )
        } else {
            format!("bestvideo{height_filter}{fps_filter}/best{height_filter}")
        }
    }

    fn estimate_video_size(
        media: &MediaInfo,
        res: ResolutionPreset,
        include_audio: bool,
    ) -> Option<u64> {
        let max_h = res.height().unwrap_or(u32::MAX);
        let mut best_video: Option<&FormatInfo> = None;
        let mut best_audio: Option<&FormatInfo> = None;

        for f in &media.formats {
            match f.stream_type {
                StreamType::VideoOnly | StreamType::Muxed => {
                    if f.height.unwrap_or(0) <= max_h {
                        let better = best_video
                            .map(|b| f.height.unwrap_or(0) > b.height.unwrap_or(0))
                            .unwrap_or(true);
                        if better {
                            best_video = Some(f);
                        }
                    }
                }
                StreamType::AudioOnly => {
                    if include_audio {
                        let better = best_audio
                            .map(|b| f.abr.unwrap_or(0.0) > b.abr.unwrap_or(0.0))
                            .unwrap_or(true);
                        if better {
                            best_audio = Some(f);
                        }
                    }
                }
            }
        }

        let mut total = 0u64;
        if let Some(v) = best_video {
            total += v.estimated_size(media.duration).unwrap_or(0);
        }
        if include_audio {
            if let Some(a) = best_audio {
                total += a.estimated_size(media.duration).unwrap_or(0);
            }
        }
        if total > 0 {
            Some(total)
        } else {
            None
        }
    }

    /// Human-readable size string.
    pub fn format_size(bytes: Option<u64>) -> String {
        match bytes {
            None => "—".into(),
            Some(b) if b < 1024 => format!("{b} B"),
            Some(b) if b < 1024 * 1024 => format!("{:.1} KB", b as f64 / 1024.0),
            Some(b) if b < 1024 * 1024 * 1024 => {
                format!("{:.1} MB", b as f64 / (1024.0 * 1024.0))
            }
            Some(b) => format!("{:.2} GB", b as f64 / (1024.0 * 1024.0 * 1024.0)),
        }
    }

    /// Formats duration in seconds to HH:MM:SS or MM:SS.
    pub fn format_duration(seconds: Option<f64>) -> String {
        match seconds {
            None => "—".into(),
            Some(s) => {
                let s = s.round() as u64;
                let hrs = s / 3600;
                let mins = (s % 3600) / 60;
                let secs = s % 60;
                if hrs > 0 {
                    format!("{hrs}:{mins:02}:{secs:02}")
                } else {
                    format!("{mins}:{secs:02}")
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    fn sample_media() -> MediaInfo {
        MediaInfo {
            id: "test".into(),
            title: "Test Video".into(),
            uploader: Some("Uploader".into()),
            duration: Some(120.0),
            thumbnail: None,
            webpage_url: None,
            formats: vec![
                FormatInfo {
                    format_id: "137".into(),
                    ext: "mp4".into(),
                    resolution: Some("1920x1080".into()),
                    height: Some(1080),
                    fps: Some(30.0),
                    vcodec: Some("avc1".into()),
                    acodec: None,
                    tbr: Some(2500.0),
                    abr: None,
                    filesize: Some(37_500_000),
                    filesize_is_approx: false,
                    stream_type: StreamType::VideoOnly,
                    format_note: None,
                },
                FormatInfo {
                    format_id: "140".into(),
                    ext: "m4a".into(),
                    resolution: None,
                    height: None,
                    fps: None,
                    vcodec: None,
                    acodec: Some("mp4a".into()),
                    tbr: Some(128.0),
                    abr: Some(128.0),
                    filesize: Some(1_920_000),
                    filesize_is_approx: false,
                    stream_type: StreamType::AudioOnly,
                    format_note: None,
                },
            ],
            audio_tracks: vec![],
            subtitles: vec![],
            description: None,
        }
    }

    #[test]
    fn video_audio_selector_contains_bestvideo() {
        let media = sample_media();
        let opts = DownloadOptions::default();
        let compiled = FormatBuilder::compile(&media, &opts);
        assert!(compiled.format_selector.contains("bv*"));
        assert!(compiled.estimated_size.is_some());
    }

    #[test]
    fn audio_only_uses_extract_audio() {
        let media = sample_media();
        let mut opts = DownloadOptions::default();
        opts.mode = DownloadMode::AudioOnly;
        opts.audio_format = AudioFormat::Mp3;
        let compiled = FormatBuilder::compile(&media, &opts);
        assert!(compiled.extra_args.iter().any(|a| a == "--extract-audio"));
        assert_eq!(compiled.output_ext, "mp3");
    }

    #[test]
    fn format_size_human() {
        assert_eq!(FormatBuilder::format_size(None), "—");
        assert_eq!(FormatBuilder::format_size(Some(500)), "500 B");
        assert!(FormatBuilder::format_size(Some(5_000_000)).contains("MB"));
    }

    #[test]
    fn multi_audio_selection_generates_multistreams_and_mkv() {
        let media = sample_media();
        let mut opts = DownloadOptions::default();
        opts.selected_audio_langs = vec!["251".to_string(), "140".to_string()];
        let compiled = FormatBuilder::compile(&media, &opts);

        assert_eq!(compiled.output_ext, "mkv");
        assert!(compiled.format_selector.contains("251+140"));
        assert!(compiled.format_selector.contains("mergeall[vcodec=none]"));
        assert!(compiled.extra_args.iter().any(|a| a == "--audio-multistreams"));
    }

    #[test]
    fn subtitle_selection_generates_correct_flags_and_mkv() {
        let media = sample_media();
        let mut opts = DownloadOptions::default();
        opts.subtitle_mode = SubtitleMode::Custom;
        opts.subtitle_langs = vec!["en".to_string(), "es".to_string()];
        let compiled = FormatBuilder::compile(&media, &opts);

        assert_eq!(compiled.output_ext, "mkv");
        // Custom mode: embed-subs + exact sub-langs, but NO write-auto-subs or write-subs
        // (those trigger HTTP 429 rate-limit errors from YouTube)
        assert!(compiled.extra_args.iter().any(|a| a == "--embed-subs"));
        assert!(!compiled.extra_args.iter().any(|a| a == "--write-subs"));
        assert!(!compiled.extra_args.iter().any(|a| a == "--write-auto-subs"));

        let sub_langs_idx = compiled.extra_args.iter().position(|a| a == "--sub-langs");
        assert!(sub_langs_idx.is_some());
        assert_eq!(compiled.extra_args[sub_langs_idx.unwrap() + 1], "en,es");
    }

    #[test]
    fn subtitle_all_mode_includes_auto_subs_and_sleep() {
        let media = sample_media();
        let mut opts = DownloadOptions::default();
        opts.subtitle_mode = SubtitleMode::All;
        opts.subtitle_langs = vec!["all".to_string()];
        let compiled = FormatBuilder::compile(&media, &opts);

        assert_eq!(compiled.output_ext, "mkv");
        assert!(compiled.extra_args.iter().any(|a| a == "--embed-subs"));
        assert!(compiled.extra_args.iter().any(|a| a == "--write-auto-subs"));
        assert!(compiled.extra_args.iter().any(|a| a == "--sleep-subtitles"));
        assert!(!compiled.extra_args.iter().any(|a| a == "--write-subs"));
    }
}
