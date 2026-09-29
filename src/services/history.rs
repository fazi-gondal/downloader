//! Persistent download history (JSON file in OS config dir).

use std::path::PathBuf;

use crate::core::{AppError, Result};
use crate::models::HistoryEntry;

const MAX_ENTRIES: usize = 500;

pub struct HistoryService {
    path: PathBuf,
    entries: Vec<HistoryEntry>,
}

impl HistoryService {
    pub fn load() -> Self {
        match Self::try_load() {
            Ok(s) => s,
            Err(_) => {
                let path = Self::history_path().unwrap_or_else(|_| {
                    PathBuf::from(".").join("history.json")
                });
                Self {
                    path,
                    entries: Vec::new(),
                }
            }
        }
    }

    fn config_dir() -> Result<PathBuf> {
        let base = dirs::config_dir().ok_or_else(|| AppError::other("no config dir"))?;
        Ok(base.join("VideoDownloader"))
    }

    fn history_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("history.json"))
    }

    fn try_load() -> Result<Self> {
        let path = Self::history_path()?;
        if !path.exists() {
            return Ok(Self {
                path,
                entries: Vec::new(),
            });
        }
        let data = std::fs::read_to_string(&path)?;
        let entries: Vec<HistoryEntry> = serde_json::from_str(&data)?;
        Ok(Self { path, entries })
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(&self.entries)?;
        std::fs::write(&self.path, data)?;
        Ok(())
    }

    pub fn list(&self) -> &[HistoryEntry] {
        &self.entries
    }

    pub fn add(&mut self, entry: HistoryEntry) {
        self.entries.insert(0, entry);
        if self.entries.len() > MAX_ENTRIES {
            self.entries.truncate(MAX_ENTRIES);
        }
        let _ = self.save();
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        let _ = self.save();
    }

    pub fn remove(&mut self, id: uuid::Uuid) {
        self.entries.retain(|e| e.id != id);
        let _ = self.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::HistoryEntry;
    use uuid::Uuid;

    #[test]
    fn add_and_list() {
        let mut svc = HistoryService {
            path: PathBuf::from("/tmp/vd-history-test.json"),
            entries: Vec::new(),
        };
        svc.add(HistoryEntry {
            id: Uuid::new_v4(),
            download_id: None,
            title: "Test".into(),
            url: "https://example.com".into(),
            output_path: None,
            file_size: Some(1024),
            completed_at: chrono::Utc::now(),
            success: true,
            error: None,
        });
        assert_eq!(svc.list().len(), 1);
        assert_eq!(svc.list()[0].title, "Test");
        let _ = std::fs::remove_file("/tmp/vd-history-test.json");
    }
}
