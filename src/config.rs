use crate::error::{CliError, Result};
use std::path::PathBuf;

pub struct ConfigManager;

impl ConfigManager {
    /// Initialize config manager
    pub fn new() -> Result<Self> {
        Ok(ConfigManager)
    }

    /// Get default database path: ~/.clippie/clipboard.db
    pub fn get_db_path(&self) -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or(CliError::ConfigError("Could not determine home directory".to_string()))?;
        Ok(home.join(".clippie").join("clipboard.db"))
    }

    /// Check if setup has been run (database exists)
    pub fn exists(&self) -> bool {
        self.get_db_path()
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// Get the path to the pause flag file
    pub fn get_pause_flag_path(&self) -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or(CliError::ConfigError("Could not determine home directory".to_string()))?;
        Ok(home.join(".clippie").join("paused"))
    }

    /// Check if clipboard monitoring is paused
    pub fn is_paused(&self) -> bool {
        self.get_pause_flag_path()
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// Set the paused state
    pub fn set_paused(&self, paused: bool) -> Result<()> {
        let path = self.get_pause_flag_path()?;
        if paused {
            std::fs::File::create(&path)?;
        } else if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize config manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_manager_creation() {
        let cm = ConfigManager::new();
        assert!(cm.is_ok());
    }
}
