//! Shared application state held as an Entity by AppShell.

use crate::config::AppSettings;
use crate::models::{DownloadOptions, MediaInfo, PlaylistInfo, SubtitleMode};
use crate::services::{
    ConversionService, DownloadManager, HistoryService, NetworkOptions, YtDlpService,
};

/// Result of the most recent successful analysis.
#[derive(Debug, Clone)]
pub enum CurrentMedia {
    None,
    Single(MediaInfo),
    Playlist(PlaylistInfo),
}

pub struct AppState {
    pub settings: AppSettings,
    pub ytdlp: YtDlpService,
    pub downloads: DownloadManager,
    pub history: HistoryService,
    pub conversions: ConversionService,
    pub current_media: CurrentMedia,
    pub current_options: DownloadOptions,
    pub last_error: Option<String>,
    pub analyzing: bool,
    pub active_route: crate::core::AppRoute,
}

impl AppState {
    pub fn new() -> Self {
        let settings = AppSettings::load();
        let mut current_options = DownloadOptions::default();
        if settings.write_subtitles {
            current_options.subtitle_mode = SubtitleMode::All;
            current_options.subtitle_langs = if settings.subtitle_langs.is_empty() {
                vec!["all".to_string()]
            } else {
                settings.subtitle_langs.clone()
            };
        } else {
            current_options.subtitle_mode = SubtitleMode::None;
            current_options.subtitle_langs = Vec::new();
        }
        current_options.multi_audio = settings.multi_audio;
        current_options.embed_thumbnail = settings.embed_thumbnail;
        current_options.embed_metadata = settings.embed_metadata;

        let network = NetworkOptions {
            proxy: settings.proxy.clone(),
            cookies_browser: settings.cookies_browser.clone(),
            rate_limit_kbps: settings.rate_limit_kbps,
            concurrent_fragments: settings.concurrent_fragments,
        };

        let mut downloads = DownloadManager::default();
        downloads.set_max_concurrent(settings.max_concurrent as usize);
        downloads.set_download_dir(settings.download_dir.clone());
        downloads.set_network(network.clone());

        let ytdlp = YtDlpService::default().with_network(network);

        Self {
            settings,
            ytdlp,
            downloads,
            history: HistoryService::load(),
            conversions: ConversionService::default(),
            current_media: CurrentMedia::None,
            current_options,
            last_error: None,
            analyzing: false,
            active_route: crate::core::AppRoute::Dashboard,
        }
    }

    /// Push current settings into download/ytdlp services.
    pub fn apply_settings_to_services(&mut self) {
        let network = NetworkOptions {
            proxy: self.settings.proxy.clone(),
            cookies_browser: self.settings.cookies_browser.clone(),
            rate_limit_kbps: self.settings.rate_limit_kbps,
            concurrent_fragments: self.settings.concurrent_fragments,
        };
        self.downloads
            .set_max_concurrent(self.settings.max_concurrent as usize);
        self.downloads
            .set_download_dir(self.settings.download_dir.clone());
        self.downloads.set_network(network.clone());
        self.ytdlp.set_network(network);
    }

    pub fn set_single(&mut self, media: MediaInfo) {
        self.current_media = CurrentMedia::Single(media);
        self.last_error = None;
        self.analyzing = false;
    }

    pub fn set_playlist(&mut self, playlist: PlaylistInfo) {
        self.current_media = CurrentMedia::Playlist(playlist);
        self.last_error = None;
        self.analyzing = false;
    }

    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.last_error = Some(msg.into());
        self.analyzing = false;
        self.current_media = CurrentMedia::None;
    }

    pub fn clear_error(&mut self) {
        self.last_error = None;
    }

    pub fn toggle_playlist_entry(&mut self, index: usize) {
        if let CurrentMedia::Playlist(ref mut pl) = self.current_media {
            if let Some(entry) = pl.entries.get_mut(index) {
                entry.selected = !entry.selected;
            }
        }
    }

    pub fn select_all_playlist(&mut self, selected: bool) {
        if let CurrentMedia::Playlist(ref mut pl) = self.current_media {
            for e in &mut pl.entries {
                e.selected = selected;
            }
        }
    }

    /// Move any newly completed/failed downloads into history (idempotent by download_id).
    pub fn sync_downloads_to_history(&mut self) {
        let existing: std::collections::HashSet<_> = self
            .history
            .list()
            .iter()
            .filter_map(|e| e.download_id)
            .collect();

        for task in self.downloads.list() {
            if !task.status.is_terminal() {
                continue;
            }
            if existing.contains(&task.id) {
                continue;
            }
            let success = task.status == crate::models::DownloadStatus::Completed;
            self.history
                .add(crate::models::HistoryEntry::from_completed_download(
                    task.id,
                    task.title.clone(),
                    task.url.clone(),
                    task.output_path.clone(),
                    task.total_bytes,
                    success,
                    task.error.clone(),
                ));
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
