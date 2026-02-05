use crate::config::ConfigManager;
use crate::db::Database;
use crate::error::Result;
use std::io::{self, Write};

pub async fn run_setup() -> Result<()> {
    println!("\n🔧 Clippie Setup Wizard\n");

    let config_manager = ConfigManager::new()?;
    let db_path = config_manager.get_db_path()?;

    // Create database directory if needed
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Create/verify database
    Database::open(&db_path)?;

    println!("✓ Database configured at {}", db_path.display());

    // Ask about installing daemon
    print!("\nInstall the clipboard monitoring daemon? [y/N]: ");
    io::stdout().flush()?;

    let mut response = String::new();
    io::stdin().read_line(&mut response)?;
    if response.trim().eq_ignore_ascii_case("y") {
        crate::commands::install::run_install().await?;
    }

    println!("\nSetup complete! 🎉");
    println!("\nNext steps:");
    println!("  1. Run 'clippie start' to start the daemon");
    println!("  2. Run 'clippie' to launch the browser\n");

    Ok(())
}
