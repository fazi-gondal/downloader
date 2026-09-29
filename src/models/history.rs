//! Completed download history entries.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::DownloadId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: Uuid,
    pub download_id: Option<DownloadId>,
    pub title: String,
    pub url: String,
    pub output_path: Option<String>,
    pub file_size: Option<u64>,
    pub completed_at: chrono::DateTime<chrono::Utc>,
    pub success: bool,
    pub error: Option<String>,
}

impl HistoryEntry {
    pub fn from_completed_download(
        download_id: DownloadId,
        title: String,
        url: String,
        output_path: Option<String>,
        file_size: Option<u64>,
        success: bool,
        error: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            download_id: Some(download_id),
            title,
            url,
            output_path,
            file_size,
            completed_at: chrono::Utc::now(),
            success,
            error,
        }
    }
}
