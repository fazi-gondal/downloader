//! Download task models.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{DownloadOptions, MediaInfo};

pub type DownloadId = Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadStatus {
    Queued,
    Analyzing,
    Downloading,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl DownloadStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Analyzing => "Analyzing",
            Self::Downloading => "Downloading",
            Self::Processing => "Processing",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: DownloadId,
    pub url: String,
    pub title: String,
    pub playlist_title: Option<String>,
    pub status: DownloadStatus,
    pub progress: f64,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub output_path: Option<String>,
    pub error: Option<String>,
    pub options: DownloadOptions,
    pub media: Option<MediaInfo>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl DownloadTask {
    pub fn new(url: impl Into<String>, options: DownloadOptions) -> Self {
        Self {
            id: Uuid::new_v4(),
            url: url.into(),
            title: "Pending…".into(),
            playlist_title: None,
            status: DownloadStatus::Queued,
            progress: 0.0,
            speed: None,
            eta: None,
            downloaded_bytes: 0,
            total_bytes: None,
            output_path: None,
            error: None,
            options,
            media: None,
            created_at: chrono::Utc::now(),
        }
    }
}
