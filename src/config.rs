use crate::error::{CliError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub db_path: String,
}

pub struct ConfigManager {
    config_dir: PathBuf,
    config_file: PathBuf,
}

impl ConfigManager {
    /// Initialize config manager with standard XDG paths
    pub fn new() -> Result<Self> {
        let config_dir = if let Ok(path) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(path).join("clippy")
        } else {
            let home = dirs::home_dir().ok_or(CliError::ConfigError(
                "Could not determine home directory".to_string(),
            ))?;
            home.join(".config").join("clippy")
        };

        let config_file = config_dir.join("config.json");

        Ok(ConfigManager {
            config_dir,
            config_file,
        })
    }

    /// Get the configuration file path
    pub fn config_file(&self) -> &Path {
        &self.config_file
    }

    /// Load configuration from file
    pub fn load(&self) -> Result<Config> {
        if !self.config_file.exists() {
            return Err(CliError::ConfigNotFound);
        }

        let content = fs::read_to_string(&self.config_file)
            .map_err(|e| CliError::ConfigError(format!("Failed to read config: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| CliError::ConfigError(format!("Failed to parse config: {}", e)))
    }

    /// Save configuration to file
    pub fn save(&self, config: &Config) -> Result<()> {
        // Create config directory if it doesn't exist
        fs::create_dir_all(&self.config_dir).map_err(|e| {
            CliError::ConfigError(format!("Failed to create config directory: {}", e))
        })?;

        let content = serde_json::to_string_pretty(config)
            .map_err(|e| CliError::ConfigError(format!("Failed to serialize config: {}", e)))?;

        fs::write(&self.config_file, content).map_err(|e| {
            CliError::ConfigError(format!("Failed to write config: {}", e))
        })?;

        Ok(())
    }

    /// Check if configuration exists
    pub fn exists(&self) -> bool {
        self.config_file.exists()
    }

    /// Get database path with priority:
    /// 1. CLIPPY_DB_PATH environment variable
    /// 2. Value from config file
    /// 3. Default location
    pub fn get_db_path(&self) -> Result<PathBuf> {
        // Check environment variable first
        if let Ok(path) = std::env::var("CLIPPIE_DB_PATH") {
            return Ok(PathBuf::from(path));
        }

        // Load from config
        if let Ok(config) = self.load() {
            return Ok(PathBuf::from(&config.db_path));
        }

        // Default location: ~/.clippie/clipboard.db
        let home = dirs::home_dir()
            .ok_or(CliError::ConfigError("Could not determine home directory".to_string()))?;
        Ok(home.join(".clippie").join("clipboard.db"))
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
