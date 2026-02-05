use crate::clipboard::{get_clipboard_change_count, get_clipboard_content, hash_content};
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
    last_change_count: i64,
    last_content: Option<String>,
}

impl DaemonState {
    pub fn new(db: Database) -> Result<Self> {
        let last_change_count = get_clipboard_change_count().unwrap_or(0);

        Ok(DaemonState {
            db,
            last_change_count,
            last_content: None,
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
        let change_count = get_clipboard_change_count()?;

        if change_count != self.last_change_count {
            self.last_change_count = change_count;
            return Ok(true);
        }

        Ok(false)
    }

    async fn check_stability(&mut self) -> Result<()> {
        match get_clipboard_content() {
            Ok(Some(content)) => {
                if content.trim().len() < MIN_CONTENT_LENGTH {
                    return Ok(());
                }

                if self.last_content.as_ref() != Some(&content) {
                    self.last_content = Some(content.clone());

                    sleep(STABILITY_CHECK_INTERVAL).await;

                    match get_clipboard_content() {
                        Ok(Some(new_content)) => {
                            if new_content == content {
                                let hash = hash_content(&content);
                                let _ = self.db.insert_entry(&content, &hash);
                            }
                        }
                        Ok(None) => {}
                        Err(_) => {}
                    }
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
    let mut daemon = DaemonState::new(db)?;

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
        let state = DaemonState::new(db);
        assert!(state.is_ok());
    }
}
