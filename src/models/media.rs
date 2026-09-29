//! Media metadata models (translated from the reference Python dataclasses).

use serde::{Deserialize, Serialize};

/// Classification of a single format stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamType {
    VideoOnly,
    AudioOnly,
    Muxed,
}

/// One downloadable format reported by yt-dlp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInfo {
    pub format_id: String,
    pub ext: String,
    pub resolution: Option<String>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    pub tbr: Option<f64>,
    pub abr: Option<f64>,
    pub filesize: Option<u64>,
    pub filesize_is_approx: bool,
    pub stream_type: StreamType,
    pub format_note: Option<String>,
}

impl FormatInfo {
    /// Approximate size in bytes when exact size is unknown (uses tbr × duration).
    pub fn estimated_size(&self, duration_secs: Option<f64>) -> Option<u64> {
        if let Some(size) = self.filesize {
            return Some(size);
        }
        let tbr = self.tbr?;
        let duration = duration_secs?;
        // tbr is kbps → bytes ≈ tbr * 1000 / 8 * duration
        Some((tbr * 125.0 * duration) as u64)
    }
}

/// Audio track language / label for multi-audio selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTrack {
    pub language: Option<String>,
    pub title: Option<String>,
    pub format_id: String,
    pub abr: Option<f64>,
    pub ext: Option<String>,
    pub acodec: Option<String>,
    pub label: Option<String>,
    pub note: Option<String>,
}

impl AudioTrack {
    pub fn display_label(&self) -> String {
        if let Some(ref l) = self.label {
            if !l.is_empty() {
                return l.clone();
            }
        }
        let mut parts = Vec::new();
        if let Some(ref t) = self.title {
            parts.push(t.clone());
        } else if let Some(ref lang) = self.language {
            parts.push(lang.clone());
        } else {
            parts.push("Audio".to_string());
        }

        if let Some(ref ext) = self.ext {
            if let Some(ref codec) = self.acodec {
                let codec_base = codec.split('.').next().unwrap_or(codec).to_lowercase();
                if !codec_base.is_empty() && &codec_base != "none" && &codec_base != ext {
                    parts.push(format!("{ext} ({codec_base})"));
                } else {
                    parts.push(ext.clone());
                }
            } else {
                parts.push(ext.clone());
            }
        }

        if let Some(abr) = self.abr {
            if abr > 0.0 {
                parts.push(format!("{} kbps", abr.round() as u64));
            }
        }

        parts.push(format!("[id: {}]", self.format_id));
        parts.join(" · ")
    }
}

/// Subtitle language entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleTrack {
    pub language: String,
    pub name: Option<String>,
    pub is_automatic: bool,
}

impl SubtitleTrack {
    pub fn display_label(&self) -> String {
        if let Some(ref n) = self.name {
            if !n.is_empty() && n != &self.language {
                return format!("{} ({})", n, self.language);
            }
        }
        self.language.clone()
    }
}

/// Single video / media item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub id: String,
    pub title: String,
    pub uploader: Option<String>,
    pub duration: Option<f64>,
    pub thumbnail: Option<String>,
    pub webpage_url: Option<String>,
    pub formats: Vec<FormatInfo>,
    pub audio_tracks: Vec<AudioTrack>,
    pub subtitles: Vec<SubtitleTrack>,
    pub description: Option<String>,
}

/// One entry inside a playlist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistEntry {
    pub id: String,
    pub title: String,
    pub url: String,
    pub duration: Option<f64>,
    pub thumbnail: Option<String>,
    pub selected: bool,
}

/// Playlist or channel listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistInfo {
    pub id: String,
    pub title: String,
    pub uploader: Option<String>,
    pub entries: Vec<PlaylistEntry>,
    pub webpage_url: Option<String>,
}

/// Download mode selected by the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DownloadMode {
    #[default]
    VideoAudio,
    AudioOnly,
    VideoOnly,
}

impl DownloadMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::VideoAudio => "Video + Audio",
            Self::AudioOnly => "Audio Only",
            Self::VideoOnly => "Video Only",
        }
    }
}

/// Preferred resolution preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ResolutionPreset {
    Best,
    P4320,
    P2160,
    P1440,
    #[default]
    P1080,
    P720,
    P480,
    P360,
    P240,
    P144,
}

impl ResolutionPreset {
    pub fn label(self) -> &'static str {
        match self {
            Self::Best => "Best",
            Self::P4320 => "4320p (8K)",
            Self::P2160 => "2160p (4K)",
            Self::P1440 => "1440p (2K)",
            Self::P1080 => "1080p",
            Self::P720 => "720p",
            Self::P480 => "480p",
            Self::P360 => "360p",
            Self::P240 => "240p",
            Self::P144 => "144p",
        }
    }

    pub fn height(self) -> Option<u32> {
        match self {
            Self::Best => None,
            Self::P4320 => Some(4320),
            Self::P2160 => Some(2160),
            Self::P1440 => Some(1440),
            Self::P1080 => Some(1080),
            Self::P720 => Some(720),
            Self::P480 => Some(480),
            Self::P360 => Some(360),
            Self::P240 => Some(240),
            Self::P144 => Some(144),
        }
    }

    pub const ALL: [ResolutionPreset; 10] = [
        ResolutionPreset::Best,
        ResolutionPreset::P4320,
        ResolutionPreset::P2160,
        ResolutionPreset::P1440,
        ResolutionPreset::P1080,
        ResolutionPreset::P720,
        ResolutionPreset::P480,
        ResolutionPreset::P360,
        ResolutionPreset::P240,
        ResolutionPreset::P144,
    ];
}

/// Container format preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Container {
    #[default]
    Mp4,
    Mkv,
    Webm,
    Avi,
    Mov,
    Ts,
}

impl Container {
    pub const ALL_VIDEO: [Container; 3] = [Container::Mp4, Container::Mkv, Container::Webm];

    pub fn extension(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mkv => "mkv",
            Self::Webm => "webm",
            Self::Avi => "avi",
            Self::Mov => "mov",
            Self::Ts => "ts",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Mp4 => "MP4",
            Self::Mkv => "MKV",
            Self::Webm => "WebM",
            Self::Avi => "AVI",
            Self::Mov => "MOV",
            Self::Ts => "TS",
        }
    }
}

/// Audio extraction format (when mode is AudioOnly).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AudioFormat {
    #[default]
    Mp3,
    M4a,
    Aac,
    Flac,
    Opus,
    Wav,
}

impl AudioFormat {
    pub const ALL: [AudioFormat; 6] = [
        AudioFormat::Mp3,
        AudioFormat::M4a,
        AudioFormat::Aac,
        AudioFormat::Flac,
        AudioFormat::Opus,
        AudioFormat::Wav,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Mp3 => "MP3",
            Self::M4a => "M4A",
            Self::Aac => "AAC",
            Self::Flac => "FLAC",
            Self::Opus => "OPUS",
            Self::Wav => "WAV",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Mp3 => "mp3",
            Self::M4a => "m4a",
            Self::Aac => "aac",
            Self::Flac => "flac",
            Self::Opus => "opus",
            Self::Wav => "wav",
        }
    }
}

/// Audio quality / bitrate preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AudioQualityPreset {
    #[default]
    Best,
    K320,
    K256,
    K192,
    K128,
}

impl AudioQualityPreset {
    pub const ALL: [AudioQualityPreset; 5] = [
        AudioQualityPreset::Best,
        AudioQualityPreset::K320,
        AudioQualityPreset::K256,
        AudioQualityPreset::K192,
        AudioQualityPreset::K128,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Best => "Best (Auto)",
            Self::K320 => "320 kbps",
            Self::K256 => "256 kbps",
            Self::K192 => "192 kbps",
            Self::K128 => "128 kbps",
        }
    }

    pub fn value(self) -> &'static str {
        match self {
            Self::Best => "0",
            Self::K320 => "320K",
            Self::K256 => "256K",
            Self::K192 => "192K",
            Self::K128 => "128K",
        }
    }
}

/// Subtitle mode preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SubtitleMode {
    #[default]
    None,
    All,
    Custom,
}

impl SubtitleMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::All => "All",
            Self::Custom => "Custom",
        }
    }
}

/// Audio tracks mode preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AudioTracksMode {
    #[default]
    Default,
    All,
    Custom,
}

impl AudioTracksMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Default => "Default Audio",
            Self::All => "All Audio Tracks",
            Self::Custom => "Select Tracks",
        }
    }
}

/// User selections that feed FormatBuilder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadOptions {
    pub mode: DownloadMode,
    pub resolution: ResolutionPreset,
    pub container: Container,
    pub audio_format: AudioFormat,
    pub audio_quality: AudioQualityPreset,
    pub fps_filter: Option<u32>,
    pub multi_audio: bool,
    pub audio_tracks_mode: AudioTracksMode,
    pub selected_audio_langs: Vec<String>,
    pub subtitle_mode: SubtitleMode,
    pub subtitle_langs: Vec<String>,
    pub embed_thumbnail: bool,
    pub embed_metadata: bool,
    pub video_format_id: Option<String>,
    pub audio_format_id: Option<String>,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            mode: DownloadMode::VideoAudio,
            resolution: ResolutionPreset::P1080,
            container: Container::Mp4,
            audio_format: AudioFormat::Mp3,
            audio_quality: AudioQualityPreset::Best,
            fps_filter: None,
            multi_audio: false,
            audio_tracks_mode: AudioTracksMode::Default,
            selected_audio_langs: Vec::new(),
            subtitle_mode: SubtitleMode::None,
            subtitle_langs: Vec::new(),
            embed_thumbnail: true,
            embed_metadata: true,
            video_format_id: None,
            audio_format_id: None,
        }
    }
}
