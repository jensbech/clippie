use crate::clipboard::{get_clipboard_change_count, get_clipboard_content, hash_content};
use crate::config::ConfigManager;
use crate::db::Database;
use crate::error::Result;
use std::time::Duration;
use tokio::time::sleep;

const CLIPBOARD_CHECK_INTERVAL: u64 = 1000; // 1 second in milliseconds
const STABILITY_CHECK_INTERVAL: Duration = Duration::from_secs(2);
const MIN_CONTENT_LENGTH: usize = 1; // Minimum length to record

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

    /// Run the main daemon loop
    pub async fn run(&mut self) -> Result<()> {
        println!("Clipboard monitoring daemon started");

        loop {
            match self.check_clipboard().await {
                Ok(true) => {
                    // Content changed, check for stability
                    if let Err(e) = self.check_stability().await {
                        eprintln!("Error checking clipboard stability: {}", e);
                    }
                }
                Ok(false) => {
                    // No change
                }
                Err(e) => {
                    eprintln!("Error checking clipboard: {}", e);
                }
            }

            sleep(Duration::from_millis(CLIPBOARD_CHECK_INTERVAL)).await;
        }
    }

    /// Check if clipboard content has changed
    async fn check_clipboard(&mut self) -> Result<bool> {
        let change_count = get_clipboard_change_count()?;

        if change_count != self.last_change_count {
            self.last_change_count = change_count;
            return Ok(true);
        }

        Ok(false)
    }

    /// Check if clipboard content is stable and record it if appropriate
    async fn check_stability(&mut self) -> Result<()> {
        // Get current content
        if let Ok(Some(content)) = get_clipboard_content() {
            // Skip very small or whitespace-only content
            if content.trim().len() < MIN_CONTENT_LENGTH {
                return Ok(());
            }

            // Check if content is different from last recorded
            if self.last_content.as_ref() != Some(&content) {
                self.last_content = Some(content.clone());

                // Wait for stability window
                sleep(STABILITY_CHECK_INTERVAL).await;

                // Check if content is still the same
                if let Ok(Some(new_content)) = get_clipboard_content() {
                    if new_content == content {
                        // Content is stable, record it
                        let hash = hash_content(&content);
                        match self.db.insert_entry(&content, &hash) {
                            Ok(id) => {
                                println!("Recorded clipboard entry (ID: {})", id);
                            }
                            Err(e) => {
                                eprintln!("Error recording clipboard entry: {}", e);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub async fn start_daemon() -> Result<()> {
    // Load configuration
    let config_manager = ConfigManager::new()?;

    if !config_manager.exists() {
        eprintln!("Error: Clippie not configured.");
        eprintln!("Run 'clippie setup' to configure the database location.");
        return Ok(());
    }

    // Get database path
    let db_path = config_manager.get_db_path()?;

    // Open or create database
    let db = Database::open(&db_path)?;

    // Create daemon state
    let mut daemon = DaemonState::new(db)?;

    // Run daemon (for now, without signal handling - can be added later)
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
