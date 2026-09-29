//! Typed application events (progress, state changes, errors).
//!
//! UI never touches worker threads. Services publish events; views subscribe.

use crate::models::{DownloadId, DownloadStatus, MediaInfo, PlaylistInfo};

/// High-level events that flow from services → UI.
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Media or playlist analysis finished successfully.
    AnalysisComplete {
        url: String,
        result: AnalysisResult,
    },
    /// Analysis failed.
    AnalysisFailed {
        url: String,
        error: String,
    },
    /// Download task status changed.
    DownloadUpdated {
        id: DownloadId,
        status: DownloadStatus,
        progress: f64,
        speed: Option<String>,
        eta: Option<String>,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
    },
    /// Generic toast / notification request.
    Toast {
        message: String,
        kind: ToastKind,
    },
}

#[derive(Debug, Clone)]
pub enum AnalysisResult {
    Single(MediaInfo),
    Playlist(PlaylistInfo),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

/// Simple in-process event bus (Phase 1).
/// Later phases can replace with GPUI entity observation where appropriate.
#[derive(Default)]
pub struct EventBus {
    // Placeholder for subscription list; real wiring uses cx.subscribe / Entity.
}
