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
    #[command(about = "Launch the clipboard history browser")]
    Tui,

    #[command(about = "Configure database location")]
    Setup,

    #[command(about = "Start the clipboard monitoring daemon")]
    Start,

    #[command(about = "Stop the clipboard monitoring daemon")]
    Stop,

    #[command(about = "Show daemon status")]
    Status,

    #[command(about = "Clear clipboard history")]
    Clear {
        #[arg(long)]
        all: bool,
    },

    #[command(about = "Install the launchd daemon")]
    Install,

    #[command(about = "Pause clipboard monitoring")]
    Pause,

    #[command(about = "Resume clipboard monitoring")]
    Resume,

    #[command(about = "Run the daemon process", hide = true)]
    Daemon,
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
    fn test_cli_status_command() {
        let cli = Cli::try_parse_from(["clippie", "status"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Status)));
    }

    #[test]
    fn test_cli_clear_all() {
        let cli = Cli::try_parse_from(["clippie", "clear", "--all"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Clear { all: true })));
    }
}
