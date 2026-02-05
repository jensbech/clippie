use crate::config::{Config, ConfigManager};
use crate::db::Database;
use crate::error::Result;
use std::io::{self, Write};
use std::path::PathBuf;

pub async fn run_setup() -> Result<()> {
    println!("\n🔧 Clippie Setup Wizard\n");

    let config_manager = ConfigManager::new()?;

    // Check if already configured
    if config_manager.exists() {
        print!("Configuration already exists at {}. Overwrite? [y/N]: ",
               config_manager.config_file().display());
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        if !response.trim().eq_ignore_ascii_case("y") {
            println!("Setup cancelled.");
            return Ok(());
        }
    }

    // Ask for database path with validation
    println!("\nWhere would you like to store the clipboard history?");
    println!("Default: ~/.clippie/clipboard.db\n");

    let default_db_path = {
        let home = dirs::home_dir().unwrap_or_default();
        home.join(".clippie").join("clipboard.db")
    };

    let db_path = loop {
        print!("Database path [{} ]: ", default_db_path.display());
        io::stdout().flush()?;

        let mut db_path_input = String::new();
        io::stdin().read_line(&mut db_path_input)?;
        let db_path_input = db_path_input.trim();

        let db_path = if db_path_input.is_empty() {
            default_db_path.clone()
        } else {
            let p = PathBuf::from(db_path_input);
            if p.is_absolute() {
                p
            } else {
                // Relative paths are relative to home
                let home = dirs::home_dir().unwrap_or_default();
                home.join(p)
            }
        };

        // Validate path
        if let Some(parent) = db_path.parent() {
            if parent.as_os_str().is_empty() {
                println!("✗ Invalid path. Please provide a valid database path.");
                continue;
            }
        } else {
            println!("✗ Invalid path. Please provide a valid database path.");
            continue;
        }

        // Check if database already exists
        if db_path.exists() {
            println!("\n⚠️  Database already exists at: {}", db_path.display());
            print!("Use existing database or create new? [use/new]: ");
            io::stdout().flush()?;

            let mut response = String::new();
            io::stdin().read_line(&mut response)?;
            let response = response.trim().to_lowercase();

            if response == "use" || response == "u" {
                println!("✓ Using existing database");
                break db_path;
            } else if response == "new" || response == "n" {
                println!("Creating new database at: {}", db_path.display());
                // Delete and recreate
                std::fs::remove_file(&db_path)?;
                break db_path;
            } else {
                println!("Invalid response. Please enter 'use' or 'new'.");
                continue;
            }
        }

        break db_path;
    };

    // Create database directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Create/verify database
    Database::open(&db_path)?;

    // Save configuration
    let config = Config {
        db_path: db_path.to_string_lossy().to_string(),
    };
    config_manager.save(&config)?;

    println!("\n✓ Configuration saved to {}", config_manager.config_file().display());
    println!("✓ Database created at {}", db_path.display());

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
