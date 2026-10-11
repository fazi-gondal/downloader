//! Pure functional compiler: UI selections → yt-dlp format string + post-processors.
//!
//! This module has no I/O and is fully unit-testable.

use crate::models::{
    DownloadMode, DownloadOptions, FormatInfo, MediaInfo, ResolutionPreset, StreamType,
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
        Self::compile_inner(Some(media), options)
    }

    /// Compile options without metadata, omitting only the best-effort size estimate.
    pub fn compile_without_media(options: &DownloadOptions) -> CompiledFormat {
        Self::compile_inner(None, options)
    }

    fn compile_inner(media: Option<&MediaInfo>, options: &DownloadOptions) -> CompiledFormat {
        match options.mode {
            DownloadMode::AudioOnly => Self::compile_audio(media, options),
            DownloadMode::VideoOnly => Self::compile_video_only(media, options),
            DownloadMode::VideoAudio => Self::compile_video_audio(media, options),
        }
    }

    fn compile_audio(media: Option<&MediaInfo>, options: &DownloadOptions) -> CompiledFormat {
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

        let estimated = media.and_then(|media| {
            media
                .formats
                .iter()
                .filter(|f| {
                    f.stream_type == StreamType::AudioOnly || f.stream_type == StreamType::Muxed
                })
                .filter_map(|f| f.estimated_size(media.duration))
                .max()
        });

        CompiledFormat {
            format_selector: selector,
            extra_args: extra,
            output_ext: ext,
            estimated_size: estimated,
        }
    }

    fn compile_video_only(media: Option<&MediaInfo>, options: &DownloadOptions) -> CompiledFormat {
        let selector = if let Some(ref vid) = options.video_format_id {
            vid.clone()
        } else {
            Self::video_selector(options.resolution, options.fps_filter, false)
        };
        let ext = options.container.extension().to_string();
        let extra = vec!["--remux-video".into(), ext.clone()];

        let estimated =
            media.and_then(|media| Self::estimate_video_size(media, options.resolution, false));

        CompiledFormat {
            format_selector: selector,
            extra_args: extra,
            output_ext: ext,
            estimated_size: estimated,
        }
    }

    fn compile_video_audio(media: Option<&MediaInfo>, options: &DownloadOptions) -> CompiledFormat {
        // ── Format selector ─────────────────────────────────────────────────────
        // Build the video part of the selector.
        // Codec-specific `bv*` candidates are ordered by preference, with
        // height/fps filters appended as attribute filters.
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
            let video_parts = Self::video_candidates(options, &filters);

            let audio_chain = options.selected_audio_langs.join("+");

            if options.selected_audio_langs.len() == 1 {
                // Exact selection: do not fall back to a different audio stream.
                let audio = &options.selected_audio_langs[0];
                Self::append_to_candidates(&video_parts, audio)
            } else {
                // Exact selection: chain only the requested IDs. If an ID is
                // stale, fail rather than silently downloading another track.
                Self::append_to_candidates(&video_parts, &audio_chain)
            }
        } else {
            match (&options.video_format_id, &options.audio_format_id) {
                (Some(v), Some(a)) => format!("{v}+{a}"),
                (Some(v), None) => format!("{v}+ba/b"),
                (None, Some(a)) => {
                    let video_parts = Self::video_candidates(options, &filters);
                    format!("{}/b", Self::append_to_candidates(&video_parts, a))
                }
                (None, None) => {
                    // "All Audio Tracks" mode: mergeall grabs every available audio stream.
                    let video_parts = Self::video_candidates(options, &filters);
                    if options.multi_audio
                        || options.audio_tracks_mode == crate::models::AudioTracksMode::All
                    {
                        format!(
                            "{}/b",
                            Self::append_to_candidates(&video_parts, "mergeall[vcodec=none]")
                        )
                    } else {
                        format!("{}/b", Self::append_to_candidates(&video_parts, "ba"))
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
        let uploaded_langs = Self::clean_subtitle_languages(&options.subtitle_langs);
        let automatic_langs = Self::clean_subtitle_languages(&options.automatic_subtitle_langs);
        let has_subs = match options.subtitle_mode {
            SubtitleMode::None => false,
            SubtitleMode::All => true,
            SubtitleMode::Custom => !uploaded_langs.is_empty() || !automatic_langs.is_empty(),
        };
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
        // Request only the source types selected by the user. yt-dlp accepts a
        // single language filter shared by uploaded and automatic subtitles.
        match options.subtitle_mode {
            SubtitleMode::None => {}
            SubtitleMode::All => {
                // All mode: grab uploaded + auto-generated for every language.
                // Add a 1-second delay between subtitle requests to avoid 429s.
                extra.push("--embed-subs".into());
                extra.push("--write-subs".into());
                extra.push("--write-auto-subs".into());
                extra.push("--sub-langs".into());
                extra.push("all".into());
                extra.push("--sleep-subtitles".into());
                extra.push("1".into());
            }
            SubtitleMode::Custom => {
                if has_subs {
                    extra.push("--embed-subs".into());
                    if !uploaded_langs.is_empty() {
                        extra.push("--write-subs".into());
                    }
                    if !automatic_langs.is_empty() {
                        extra.push("--write-auto-subs".into());
                    }
                    let mut languages = uploaded_langs;
                    for lang in automatic_langs {
                        if !languages.contains(&lang) {
                            languages.push(lang);
                        }
                    }
                    extra.push("--sub-langs".into());
                    extra.push(languages.join(","));
                }
            }
        }

        // ── Multi-audio streams ──────────────────────────────────────────────────
        // --audio-multistreams tells FFmpeg to keep every audio stream during merge.
        // Without it only the first audio stream survives.
        if is_multi_audio {
            extra.push("--audio-multistreams".into());
        }

        let estimated =
            media.and_then(|media| Self::estimate_video_size(media, options.resolution, true));

        CompiledFormat {
            format_selector: selector,
            extra_args: extra,
            output_ext: effective_ext,
            estimated_size: estimated,
        }
    }

    fn clean_subtitle_languages(langs: &[String]) -> Vec<String> {
        let mut clean = Vec::new();
        for lang in langs {
            let lang = lang.trim();
            if !lang.is_empty() && lang != "all" && !clean.iter().any(|item| item == lang) {
                clean.push(lang.to_string());
            }
        }
        clean
    }

    fn video_selector(res: ResolutionPreset, fps: Option<u32>, prefer_vp9: bool) -> String {
        let height_filter = match res.height() {
            Some(h) => format!("[height<=?{h}]"),
            None => String::new(),
        };
        let fps_filter = match fps {
            Some(f) => format!("[fps<=?{f}]"),
            None => String::new(),
        };

        Self::video_candidates_for_filters(&format!("{height_filter}{fps_filter}"), prefer_vp9)
            .join("/")
    }

    fn video_candidates(options: &DownloadOptions, filters: &str) -> Vec<String> {
        if let Some(ref video_id) = options.video_format_id {
            vec![video_id.clone()]
        } else {
            Self::video_candidates_for_filters(filters, options.prefer_vp9_video)
        }
    }

    fn video_candidates_for_filters(filters: &str, prefer_vp9: bool) -> Vec<String> {
        let codecs = if prefer_vp9 {
            ["[vcodec^=vp9]", "[vcodec^=vp09]", "[vcodec^=avc1]", ""]
        } else {
            ["[vcodec^=avc1]", "[vcodec^=vp9]", "[vcodec^=vp09]", ""]
        };

        codecs
            .into_iter()
            .map(|codec| format!("bv*{codec}{filters}"))
            .collect()
    }

    fn append_to_candidates(candidates: &[String], suffix: &str) -> String {
        candidates
            .iter()
            .map(|video| format!("{video}+{suffix}"))
            .collect::<Vec<_>>()
            .join("/")
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
        if include_audio && let Some(a) = best_audio {
            total += a.estimated_size(media.duration).unwrap_or(0);
        }
        if total > 0 { Some(total) } else { None }
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
    fn default_video_selection_prefers_vp9_then_h264() {
        let compiled = FormatBuilder::compile(&sample_media(), &DownloadOptions::default());
        let vp9 = compiled
            .format_selector
            .find("[vcodec^=vp9]")
            .expect("VP9 selector should be first");
        let vp09 = compiled
            .format_selector
            .find("[vcodec^=vp09]")
            .expect("VP09 selector should be available");
        let h264 = compiled
            .format_selector
            .find("[vcodec^=avc1]")
            .expect("H.264 fallback should be available");

        assert!(vp9 < vp09);
        assert!(vp09 < h264);
    }

    #[test]
    fn disabling_vp9_preference_starts_with_h264() {
        let opts = DownloadOptions {
            prefer_vp9_video: false,
            ..DownloadOptions::default()
        };
        let compiled = FormatBuilder::compile(&sample_media(), &opts);
        assert!(compiled.format_selector.starts_with("bv*[vcodec^=avc1]"));
    }

    #[test]
    fn audio_only_uses_extract_audio() {
        let media = sample_media();
        let opts = DownloadOptions {
            mode: DownloadMode::AudioOnly,
            audio_format: AudioFormat::Mp3,
            ..DownloadOptions::default()
        };
        let compiled = FormatBuilder::compile(&media, &opts);
        assert!(compiled.extra_args.iter().any(|a| a == "--extract-audio"));
        assert_eq!(compiled.output_ext, "mp3");
    }

    #[test]
    fn video_only_forces_selected_container() {
        let media = sample_media();
        let opts = DownloadOptions {
            mode: DownloadMode::VideoOnly,
            container: Container::Mkv,
            ..DownloadOptions::default()
        };
        let compiled = FormatBuilder::compile(&media, &opts);

        assert_eq!(compiled.output_ext, "mkv");
        assert_eq!(compiled.extra_args, vec!["--remux-video", "mkv"]);
    }

    #[test]
    fn compile_without_media_preserves_selector_and_post_processing_options() {
        let opts = DownloadOptions {
            video_format_id: Some("137".into()),
            audio_format_id: Some("140".into()),
            subtitle_mode: SubtitleMode::Custom,
            subtitle_langs: vec!["en".into()],
            ..DownloadOptions::default()
        };

        let compiled = FormatBuilder::compile_without_media(&opts);

        assert_eq!(compiled.format_selector, "137+140");
        assert!(compiled.extra_args.iter().any(|arg| arg == "--embed-subs"));
        assert!(compiled.extra_args.iter().any(|arg| arg == "en"));
        assert_eq!(compiled.estimated_size, None);
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
        let opts = DownloadOptions {
            selected_audio_langs: vec!["251".to_string(), "140".to_string()],
            ..DownloadOptions::default()
        };
        let compiled = FormatBuilder::compile(&media, &opts);

        assert_eq!(compiled.output_ext, "mkv");
        assert!(compiled.format_selector.contains("251+140"));
        assert!(compiled.format_selector.contains("mergeall[vcodec=none]"));
        assert!(
            compiled
                .extra_args
                .iter()
                .any(|a| a == "--audio-multistreams")
        );
    }

    #[test]
    fn subtitle_selection_generates_correct_flags_and_mkv() {
        let media = sample_media();
        let opts = DownloadOptions {
            subtitle_mode: SubtitleMode::Custom,
            subtitle_langs: vec!["en".to_string(), "es".to_string()],
            ..DownloadOptions::default()
        };
        let compiled = FormatBuilder::compile(&media, &opts);

        assert_eq!(compiled.output_ext, "mkv");
        assert!(compiled.extra_args.iter().any(|a| a == "--embed-subs"));
        assert!(compiled.extra_args.iter().any(|a| a == "--write-subs"));
        assert!(!compiled.extra_args.iter().any(|a| a == "--write-auto-subs"));

        let sub_langs_idx = compiled.extra_args.iter().position(|a| a == "--sub-langs");
        assert!(sub_langs_idx.is_some());
        assert_eq!(compiled.extra_args[sub_langs_idx.unwrap() + 1], "en,es");
    }

    #[test]
    fn automatic_only_requests_no_uploaded_subtitles() {
        let opts = DownloadOptions {
            subtitle_mode: SubtitleMode::Custom,
            automatic_subtitle_langs: vec!["en".into()],
            ..DownloadOptions::default()
        };
        let compiled = FormatBuilder::compile_without_media(&opts);
        assert_eq!(compiled.output_ext, "mkv");
        assert!(compiled.extra_args.contains(&"--embed-subs".into()));
        assert!(compiled.extra_args.contains(&"--write-auto-subs".into()));
        assert!(!compiled.extra_args.contains(&"--write-subs".into()));
        let index = compiled
            .extra_args
            .iter()
            .position(|arg| arg == "--sub-langs")
            .unwrap();
        assert_eq!(compiled.extra_args[index + 1], "en");
    }

    #[test]
    fn mixed_sources_deduplicate_languages_in_stable_order() {
        let opts = DownloadOptions {
            subtitle_mode: SubtitleMode::Custom,
            subtitle_langs: vec!["en".into(), "es".into(), "en".into()],
            automatic_subtitle_langs: vec!["en".into(), "fr".into(), "fr".into()],
            ..DownloadOptions::default()
        };
        let compiled = FormatBuilder::compile_without_media(&opts);
        assert_eq!(compiled.output_ext, "mkv");
        assert!(compiled.extra_args.contains(&"--embed-subs".into()));
        assert!(compiled.extra_args.contains(&"--write-subs".into()));
        assert!(compiled.extra_args.contains(&"--write-auto-subs".into()));
        assert_eq!(
            compiled
                .extra_args
                .iter()
                .filter(|arg| *arg == "--sub-langs")
                .count(),
            1
        );
        let index = compiled
            .extra_args
            .iter()
            .position(|arg| arg == "--sub-langs")
            .unwrap();
        assert_eq!(compiled.extra_args[index + 1], "en,es,fr");
    }

    #[test]
    fn no_subtitles_requests_neither_source_even_with_stale_languages() {
        let opts = DownloadOptions {
            subtitle_mode: SubtitleMode::None,
            subtitle_langs: vec!["en".into()],
            automatic_subtitle_langs: vec!["fr".into()],
            ..DownloadOptions::default()
        };
        let compiled = FormatBuilder::compile_without_media(&opts);
        assert_eq!(compiled.output_ext, "mp4");
        for flag in [
            "--embed-subs",
            "--write-subs",
            "--write-auto-subs",
            "--sub-langs",
        ] {
            assert!(!compiled.extra_args.iter().any(|arg| arg == flag));
        }
    }

    #[test]
    fn empty_custom_selection_does_not_force_mkv_or_embed() {
        let opts = DownloadOptions {
            subtitle_mode: SubtitleMode::Custom,
            subtitle_langs: vec!["all".into(), " ".into()],
            ..DownloadOptions::default()
        };
        let compiled = FormatBuilder::compile_without_media(&opts);
        assert_eq!(compiled.output_ext, "mp4");
        assert!(!compiled.extra_args.contains(&"--embed-subs".into()));
    }

    #[test]
    fn subtitle_all_mode_includes_auto_subs_and_sleep() {
        let media = sample_media();
        let opts = DownloadOptions {
            subtitle_mode: SubtitleMode::All,
            subtitle_langs: vec!["all".to_string()],
            ..DownloadOptions::default()
        };
        let compiled = FormatBuilder::compile(&media, &opts);

        assert_eq!(compiled.output_ext, "mkv");
        assert!(compiled.extra_args.iter().any(|a| a == "--embed-subs"));
        assert!(compiled.extra_args.iter().any(|a| a == "--write-subs"));
        assert!(compiled.extra_args.iter().any(|a| a == "--write-auto-subs"));
        assert!(compiled.extra_args.iter().any(|a| a == "--sleep-subtitles"));
        let index = compiled
            .extra_args
            .iter()
            .position(|arg| arg == "--sub-langs")
            .unwrap();
        assert_eq!(compiled.extra_args[index + 1], "all");
    }
}
