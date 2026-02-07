use crate::config::ConfigManager;
use crate::db::Database;
use crate::error::Result;
use std::process::Command;

pub async fn run_status() -> Result<()> {
    let config = ConfigManager::new()?;

    if !config.exists() {
        println!("Clippie is not configured.");
        println!("Run 'clippie setup' to get started.\n");
        return Ok(());
    }

    let db_path = config.get_db_path()?;
    let daemon_running = check_daemon_running();

    println!("\nClipboard History Manager Status");
    println!("================================\n");
    println!("Daemon Status:   {} {}",
        if daemon_running { "✓" } else { "✗" },
        if daemon_running { "Running" } else { "Stopped" }
    );

    if db_path.exists() {
        if let Ok(db) = Database::open(&db_path) {
            if let Ok(count) = db.count_entries() {
                println!("Entries:         {}", count);
            }
            if let Ok(size) = db.get_size() {
                println!("Database Size:   {} KB", size / 1024);
            }
        }
    }

    println!("Database Path:   {}\n", db_path.display());
    Ok(())
}

fn check_daemon_running() -> bool {
    Command::new("launchctl")
        .args(["list"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("clippie-daemon"))
        .unwrap_or(false)
}
