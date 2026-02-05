use crate::config::ConfigManager;
use crate::db::Database;
use crate::error::Result;
use std::io::{self, Write};

pub async fn run_clear(all: bool) -> Result<()> {
    let config_manager = ConfigManager::new()?;

    if !config_manager.exists() {
        eprintln!("Error: Clippie not configured.");
        eprintln!("Run 'clippie setup' to configure the database location.");
        return Ok(());
    }

    let db_path = config_manager.get_db_path()?;

    if !db_path.exists() {
        eprintln!("Error: Database not found at {}", db_path.display());
        return Ok(());
    }

    let db = Database::open(&db_path)?;

    if all {
        print!("Are you sure you want to delete ALL clipboard history? This cannot be undone. [y/N]: ");
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        if !response.trim().eq_ignore_ascii_case("y") {
            println!("Cleared cancelled.");
            return Ok(());
        }

        let count = db.clear_all()?;
        println!("✓ Deleted {} clipboard entries", count);
    } else {
        // Clear entries older than 30 days
        let count = db.delete_entries_older_than_days(30)?;
        println!("✓ Deleted {} old clipboard entries", count);
    }

    println!();

    Ok(())
}
