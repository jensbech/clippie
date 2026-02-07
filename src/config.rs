use crate::error::{CliError, Result};
use std::path::PathBuf;

pub struct ConfigManager;

impl ConfigManager {
    pub fn new() -> Result<Self> {
        Ok(ConfigManager)
    }

    fn get_clippie_dir(&self) -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or(CliError::ConfigError("Could not determine home directory".to_string()))?;
        Ok(home.join(".clippie"))
    }

    pub fn get_db_path(&self) -> Result<PathBuf> {
        Ok(self.get_clippie_dir()?.join("clipboard.db"))
    }

    pub fn exists(&self) -> bool {
        self.get_db_path().map(|p| p.exists()).unwrap_or(false)
    }

    pub fn is_paused(&self) -> bool {
        self.get_clippie_dir()
            .map(|p| p.join("paused").exists())
            .unwrap_or(false)
    }

    pub fn set_paused(&self, paused: bool) -> Result<()> {
        let path = self.get_clippie_dir()?.join("paused");
        if paused {
            std::fs::File::create(&path)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
            }
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
        assert!(ConfigManager::new().is_ok());
    }
}
