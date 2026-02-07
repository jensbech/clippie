use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Clipboard error: {0}")]
    ClipboardError(String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[allow(dead_code)]
    #[error("Config not found. Run 'clippie setup' to configure the database location.")]
    ConfigNotFound,
}

pub type Result<T> = std::result::Result<T, CliError>;
