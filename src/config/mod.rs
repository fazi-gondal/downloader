//! Application settings and persistence.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::{AppError, Result};

/// Persistent application settings (mirrors the reference settings table).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub download_dir: PathBuf,
    pub max_concurrent: u32,
    pub concurrent_fragments: u32,
    pub proxy: String,
    pub cookies_browser: String,
    pub rate_limit_kbps: u32,
    pub keep_originals: bool,
    pub write_subtitles: bool,
    pub subtitle_langs: Vec<String>,
    pub multi_audio: bool,
    pub embed_thumbnail: bool,
    pub embed_metadata: bool,
    pub prefer_vp9_video: bool,
    pub theme_mode: ThemeMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeMode {
    #[default]
    System,
    Dark,
    Light,
}

impl ThemeMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Dark => "Dark",
            Self::Light => "Light",
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        let download_dir = dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("VideoDownloader");
        Self {
            download_dir,
            max_concurrent: 8,
            concurrent_fragments: 1,
            proxy: String::new(),
            cookies_browser: String::new(),
            rate_limit_kbps: 0,
            keep_originals: true,
            write_subtitles: false,
            subtitle_langs: Vec::new(),
            multi_audio: false,
            embed_thumbnail: true,
            embed_metadata: true,
            prefer_vp9_video: true,
            theme_mode: ThemeMode::System,
        }
    }
}

impl AppSettings {
    fn config_dir() -> Result<PathBuf> {
        let base = dirs::config_dir().ok_or_else(|| AppError::other("no config dir"))?;
        Ok(base.join("VideoDownloader"))
    }

    fn settings_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("settings.json"))
    }

    pub fn load() -> Self {
        match Self::try_load() {
            Ok(s) => s,
            Err(_) => {
                let s = Self::default();
                let _ = s.save();
                s
            }
        }
    }

    fn try_load() -> Result<Self> {
        let path = Self::settings_path()?;
        let data = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir()?;
        std::fs::create_dir_all(&dir)?;
        let path = Self::settings_path()?;
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }
}
