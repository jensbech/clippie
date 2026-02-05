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
