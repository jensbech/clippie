use crate::clipboard::{get_clipboard_content, hash_content};
use crate::config::ConfigManager;
use crate::db::Database;
use crate::error::Result;
use std::time::Duration;
use tokio::time::sleep;

const CLIPBOARD_CHECK_INTERVAL: u64 = 500;
const STABILITY_CHECK_INTERVAL: Duration = Duration::from_millis(500);
const MIN_CONTENT_LENGTH: usize = 1;

pub struct DaemonState {
    db: Database,
    last_content_hash: Option<String>,
    config_manager: ConfigManager,
}

impl DaemonState {
    pub fn new(db: Database, config_manager: ConfigManager) -> Result<Self> {
        Ok(DaemonState {
            db,
            last_content_hash: None,
            config_manager,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            match self.check_clipboard().await {
                Ok(true) => {
                    if let Err(_) = self.check_stability().await {
                        // Silently continue on stability check errors
                    }
                }
                Ok(false) => {
                    // No change
                }
                Err(_) => {
                    // Silently continue on clipboard check errors
                }
            }

            sleep(Duration::from_millis(CLIPBOARD_CHECK_INTERVAL)).await;
        }
    }

    async fn check_clipboard(&mut self) -> Result<bool> {
        match get_clipboard_content()? {
            Some(content) => {
                let hash = hash_content(&content);
                if self.last_content_hash.as_ref() != Some(&hash) {
                    self.last_content_hash = Some(hash);
                    return Ok(true);
                }
                Ok(false)
            }
            None => Ok(false),
        }
    }

    async fn check_stability(&mut self) -> Result<()> {
        match get_clipboard_content() {
            Ok(Some(content)) => {
                if content.trim().len() < MIN_CONTENT_LENGTH {
                    return Ok(());
                }

                sleep(STABILITY_CHECK_INTERVAL).await;

                match get_clipboard_content() {
                    Ok(Some(new_content)) => {
                        if new_content == content {
                            // Skip writing if paused
                            if self.config_manager.is_paused() {
                                return Ok(());
                            }
                            let hash = hash_content(&content);
                            let _ = self.db.insert_entry(&content, &hash);
                        }
                    }
                    Ok(None) => {}
                    Err(_) => {}
                }
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(_) => Ok(()),
        }
    }
}

pub async fn start_daemon() -> Result<()> {
    let config_manager = ConfigManager::new()?;

    if !config_manager.exists() {
        eprintln!("Error: Clippie not configured.");
        eprintln!("Run 'clippie setup' to configure the database location.");
        return Ok(());
    }

    let db_path = config_manager.get_db_path()?;
    let db = Database::open(&db_path)?;
    let mut daemon = DaemonState::new(db, config_manager)?;

    daemon.run().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_daemon_state_creation() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::open(tmp.path()).unwrap();
        let config_manager = ConfigManager::new().unwrap();
        let state = DaemonState::new(db, config_manager);
        assert!(state.is_ok());
    }
}
