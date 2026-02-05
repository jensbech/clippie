use crate::config::ConfigManager;
use crate::db::Database;
use crate::error::Result;
use std::process::Command;

pub async fn run_status() -> Result<()> {
    let config_manager = ConfigManager::new()?;

    if !config_manager.exists() {
        println!("Clippie is not configured.");
        println!("Run 'clippie setup' to get started.\n");
        return Ok(());
    }

    let db_path = config_manager.get_db_path()?;

    // Check daemon status
    let daemon_status = check_daemon_status();
    let status_symbol = if daemon_status.is_running { "✓" } else { "✗" };

    println!("\nClipboard History Manager Status");
    println!("================================\n");

    println!("Daemon Status:   {} {}", status_symbol,
             if daemon_status.is_running { "Running" } else { "Stopped" });

    if let Some(pid) = daemon_status.pid {
        println!("Daemon PID:      {}", pid);
    }

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

struct DaemonStatus {
    is_running: bool,
    pid: Option<i32>,
}

fn check_daemon_status() -> DaemonStatus {
    // On macOS, check if the LaunchAgent is loaded
    let output = Command::new("launchctl")
        .args(&["list"])
        .output();

    let is_running = match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains("clippy-daemon")
        }
        Err(_) => false,
    };

    DaemonStatus {
        is_running,
        pid: None,
    }
}
