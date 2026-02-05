mod cli;
mod clipboard;
mod commands;
mod config;
mod daemon;
mod db;
mod error;
mod tui;

use cli::{Cli, Commands};
use config::ConfigManager;
use db::Database;
use error::Result;
use std::process;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse_args();

    match cli.command {
        None => {
            launch_tui().await?;
        }
        Some(Commands::Setup) => {
            cmd_setup().await?;
        }
        Some(Commands::Start) => {
            cmd_start().await?;
        }
        Some(Commands::Stop) => {
            cmd_stop().await?;
        }
        Some(Commands::Status) => {
            cmd_status().await?;
        }
        Some(Commands::Db { path }) => {
            cmd_db(path).await?;
        }
        Some(Commands::Clear { all }) => {
            cmd_clear(all).await?;
        }
        Some(Commands::Install) => {
            cmd_install().await?;
        }
        Some(Commands::Tui) => {
            launch_tui().await?;
        }
        Some(Commands::Daemon) => {
            daemon::start_daemon().await?;
        }
    }

    Ok(())
}

async fn launch_tui() -> Result<()> {
    let config_manager = ConfigManager::new()?;
    if !config_manager.exists() {
        eprintln!("Error: Clippie not configured.");
        eprintln!("Run 'clippie setup' to configure the database location.");
        process::exit(1);
    }

    let db_path = config_manager.get_db_path()?;
    if !db_path.exists() {
        eprintln!("Error: Clipboard history database not found.");
        eprintln!("Expected at: {}", db_path.display());
        eprintln!("Make sure the daemon is running or run 'clippie setup'.");
        process::exit(1);
    }

    let db = Database::open(&db_path)?;

    let entries = db.get_all_entries()?;
    let db_path_str = db_path.to_string_lossy().to_string();

    let mut stdout = std::io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;

    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let terminal = ratatui::Terminal::new(backend)?;

    let result = run_tui(terminal, entries, db_path_str).await;

    crossterm::terminal::disable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::LeaveAlternateScreen)?;

    result?;

    Ok(())
}

async fn run_tui(
    mut terminal: ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    entries: Vec<db::ClipboardEntry>,
    db_path: String,
) -> Result<()> {
    let (w, h) = crossterm::terminal::size()
        .map(|(w, h)| (w as usize, h as usize))
        .unwrap_or((80, 24));

    let mut app = tui::App::new(entries, db_path, w, h);
    let mut event_handler = tui::EventHandler::new();

    loop {
        terminal.draw(|f| {
            tui::draw(f, &app);
        })?;

        if let Some(event) = event_handler.next().await {
            if tui::handlers::EventHandler::handle(&event, &mut app) {
                break;
            }
        }
    }

    if let Some(content) = &app.selected_entry {
        clipboard::set_clipboard_content(content)?;
        println!("{}", content);
    }

    event_handler.stop();
    Ok(())
}

async fn cmd_setup() -> Result<()> {
    commands::run_setup().await
}

async fn cmd_start() -> Result<()> {
    use std::process::Command;

    println!("\nStarting the clipboard daemon...\n");

    let plist_path = dirs::home_dir()
        .unwrap_or_default()
        .join("Library/LaunchAgents/no.bechsor.clippie-daemon.plist");

    if !plist_path.exists() {
        eprintln!("Error: Daemon not installed. Run 'clippie setup' and choose to install the daemon.");
        return Ok(());
    }

    let output = Command::new("launchctl")
        .args(&["load", "-F"])
        .arg(&plist_path)
        .output()?;

    if output.status.success() {
        println!("✓ Daemon started\n");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Failed to start daemon: {}", stderr);
    }

    Ok(())
}

async fn cmd_stop() -> Result<()> {
    println!("\nStopping the clipboard daemon...\n");

    let output = std::process::Command::new("launchctl")
        .args(&["unload", "-F"])
        .arg(
            dirs::home_dir()
                .unwrap_or_default()
                .join("Library/LaunchAgents/no.bechsor.clippy-daemon.plist")
        )
        .output()?;

    if output.status.success() {
        println!("✓ Daemon stopped\n");
    } else {
        eprintln!("Failed to stop daemon");
    }

    Ok(())
}

async fn cmd_status() -> Result<()> {
    commands::run_status().await
}

async fn cmd_db(path: String) -> Result<()> {
    commands::run_db(path).await
}

async fn cmd_clear(all: bool) -> Result<()> {
    commands::run_clear(all).await
}

async fn cmd_install() -> Result<()> {
    commands::run_install().await
}
