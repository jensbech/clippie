use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "clippie",
    about = "A fast, keyboard-driven clipboard history manager for macOS",
    version = "1.0.0"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Launch the clipboard history browser
    #[command(about = "Launch the clipboard history browser")]
    Tui,

    /// Configure database location and settings
    #[command(about = "Configure database location")]
    Setup,

    /// Start the clipboard monitoring daemon
    #[command(about = "Start the clipboard monitoring daemon")]
    Start,

    /// Stop the clipboard monitoring daemon
    #[command(about = "Stop the clipboard monitoring daemon")]
    Stop,

    /// Show daemon status and statistics
    #[command(about = "Show daemon status")]
    Status,

    /// Switch to a different database
    #[command(about = "Switch to a different database")]
    Db {
        /// Path to the new database file
        path: String,
    },

    /// Clear clipboard history
    #[command(about = "Clear clipboard history")]
    Clear {
        /// Delete all entries instead of just old ones
        #[arg(long)]
        all: bool,
    },

    /// Install the launchd daemon
    #[command(about = "Install the launchd daemon")]
    Install,
}

impl Cli {
    pub fn parse_args() -> Self {
        Parser::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let cli: Cli = Cli::try_parse_from(&["clippie", "--help"])
            .unwrap_or_else(|e| {
                if e.kind == clap::error::ErrorKind::DisplayHelp {
                    panic!("Help requested");
                }
                panic!("Parse error: {}", e)
            });
    }
}
