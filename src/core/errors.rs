//! Application error taxonomy.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("yt-dlp error: {0}")]
    YtDlp(String),

    #[error("FFmpeg error: {0}")]
    Ffmpeg(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("cancelled")]
    Cancelled,

    #[error("{0}")]
    Other(String),
}

impl AppError {
    pub fn ytdlp(msg: impl Into<String>) -> Self {
        Self::YtDlp(msg.into())
    }

    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}
