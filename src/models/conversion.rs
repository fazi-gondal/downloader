//! Local media conversion task models.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type ConversionId = Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversionStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl ConversionStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Running => "Running",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

/// Target container / codec profile for conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConversionProfile {
    /// Stream copy to MKV (fastest, zero quality loss).
    #[default]
    RemuxCopy,
    /// Stream copy into MP4 container.
    RemuxMp4,
    /// Stream copy into MKV container.
    RemuxMkv,
    /// Stream copy into WebM container.
    RemuxWebm,
    /// Re-encode to H.264 + AAC in MP4.
    H264AacMp4,
    /// Re-encode to H.265 + AAC in MKV.
    H265AacMkv,
    /// Re-encode to VP9 + Opus in WebM.
    Vp9OpusWebm,
    /// Extract audio to MP3 (320 kbps).
    AudioMp3,
    /// Extract audio to M4A (256 kbps AAC).
    AudioM4a,
    /// Extract audio to AAC.
    AudioAac,
    /// Extract audio to FLAC (lossless).
    AudioFlac,
    /// Extract audio to Opus.
    AudioOpus,
    /// Extract audio to WAV (lossless PCM).
    AudioWav,
}

impl ConversionProfile {
    pub const ALL_VIDEO: [ConversionProfile; 6] = [
        ConversionProfile::RemuxMp4,
        ConversionProfile::RemuxMkv,
        ConversionProfile::RemuxWebm,
        ConversionProfile::H264AacMp4,
        ConversionProfile::H265AacMkv,
        ConversionProfile::Vp9OpusWebm,
    ];

    pub const ALL_AUDIO: [ConversionProfile; 6] = [
        ConversionProfile::AudioMp3,
        ConversionProfile::AudioM4a,
        ConversionProfile::AudioAac,
        ConversionProfile::AudioFlac,
        ConversionProfile::AudioOpus,
        ConversionProfile::AudioWav,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::RemuxCopy => "Remux (copy)",
            Self::RemuxMp4 => "Remux → MP4 (copy)",
            Self::RemuxMkv => "Remux → MKV (copy)",
            Self::RemuxWebm => "Remux → WebM (copy)",
            Self::H264AacMp4 => "H.264 + AAC (MP4)",
            Self::H265AacMkv => "H.265 + AAC (MKV)",
            Self::Vp9OpusWebm => "VP9 + Opus (WebM)",
            Self::AudioMp3 => "Audio → MP3 (320k)",
            Self::AudioM4a => "Audio → M4A (256k)",
            Self::AudioAac => "Audio → AAC (256k)",
            Self::AudioFlac => "Audio → FLAC (Lossless)",
            Self::AudioOpus => "Audio → Opus (192k)",
            Self::AudioWav => "Audio → WAV (Lossless)",
        }
    }

    pub fn output_ext(self) -> &'static str {
        match self {
            Self::RemuxCopy | Self::RemuxMkv | Self::H265AacMkv => "mkv",
            Self::RemuxMp4 | Self::H264AacMp4 => "mp4",
            Self::RemuxWebm | Self::Vp9OpusWebm => "webm",
            Self::AudioMp3 => "mp3",
            Self::AudioM4a => "m4a",
            Self::AudioAac => "aac",
            Self::AudioFlac => "flac",
            Self::AudioOpus => "opus",
            Self::AudioWav => "wav",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionTask {
    pub id: ConversionId,
    pub input_path: String,
    pub output_path: Option<String>,
    pub profile: ConversionProfile,
    pub keep_original: bool,
    pub status: ConversionStatus,
    pub progress: f64,
    pub error: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ConversionTask {
    pub fn new(input_path: impl Into<String>, profile: ConversionProfile) -> Self {
        Self::new_with_options(input_path, profile, true)
    }

    pub fn new_with_options(
        input_path: impl Into<String>,
        profile: ConversionProfile,
        keep_original: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            input_path: input_path.into(),
            output_path: None,
            profile,
            keep_original,
            status: ConversionStatus::Queued,
            progress: 0.0,
            error: None,
            created_at: chrono::Utc::now(),
        }
    }
}
