use crate::clipboard::{get_clipboard_content, hash_content};
use crate::config::ConfigManager;
use crate::db::Database;
use crate::error::Result;
use std::time::Duration;
use tokio::time::sleep;

const CHECK_INTERVAL: Duration = Duration::from_millis(500);
const STABILITY_DELAY: Duration = Duration::from_millis(500);

pub struct DaemonState {
    db: Database,
    last_hash: Option<String>,
    config: ConfigManager,
}

impl DaemonState {
    pub fn new(db: Database, config: ConfigManager) -> Self {
        DaemonState { db, last_hash: None, config }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            if let Ok(Some(content)) = get_clipboard_content() {
                let hash = hash_content(&content);
                if self.last_hash.as_ref() != Some(&hash) {
                    self.last_hash = Some(hash);
                    self.try_save_content(&content).await;
                }
            }
            sleep(CHECK_INTERVAL).await;
        }
    }

    async fn try_save_content(&self, content: &str) {
        if content.trim().is_empty() || self.config.is_paused() {
            return;
        }

        sleep(STABILITY_DELAY).await;

        if let Ok(Some(new_content)) = get_clipboard_content() {
            if new_content == content {
                let hash = hash_content(content);
                let _ = self.db.insert_entry(content, &hash);
            }
        }
    }
}

pub async fn start_daemon() -> Result<()> {
    let config = ConfigManager::new()?;

    if !config.exists() {
        eprintln!("Error: Clippie not configured. Run 'clippie setup' first.");
        return Ok(());
    }

    let db_path = config.get_db_path()?;
    let db = Database::open(&db_path)?;
    let mut daemon = DaemonState::new(db, config);
    daemon.run().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_daemon_state_creation() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::open(tmp.path()).unwrap();
        let config = ConfigManager::new().unwrap();
        let _state = DaemonState::new(db, config);
    }
}
