use crate::config::{Config, ConfigManager};
use crate::db::Database;
use crate::error::Result;
use std::path::PathBuf;

pub async fn run_db(path: String) -> Result<()> {
    let config_manager = ConfigManager::new()?;

    // Parse the path
    let db_path = if path.starts_with('/') {
        // Absolute path
        PathBuf::from(&path)
    } else if path.starts_with("~/") {
        // Home-relative path
        let home = dirs::home_dir().ok_or_else(|| {
            crate::error::CliError::ConfigError("Could not determine home directory".to_string())
        })?;
        home.join(&path[2..])
    } else {
        // Relative to home
        let home = dirs::home_dir().ok_or_else(|| {
            crate::error::CliError::ConfigError("Could not determine home directory".to_string())
        })?;
        home.join(&path)
    };

    // Create directory if needed
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Verify database can be opened/created
    Database::open(&db_path)?;

    // Save configuration
    let config = Config {
        db_path: db_path.to_string_lossy().to_string(),
    };
    config_manager.save(&config)?;

    println!("✓ Database path switched to: {}", db_path.display());
    println!("\nYou may need to restart the daemon for changes to take effect.");
    println!("Run 'clippie stop' and then 'clippie start'.\n");

    Ok(())
}
