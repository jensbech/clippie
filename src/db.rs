use crate::error::{CliError, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub last_copied: DateTime<Utc>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700));
                }
            }
        }

        let conn = Connection::open(path).map_err(CliError::DatabaseError)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
        }
        let db = Database { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    fn initialize_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS clipboard_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL UNIQUE,
                content_hash TEXT NOT NULL UNIQUE,
                created_at INTEGER NOT NULL,
                last_copied INTEGER NOT NULL,
                copy_count INTEGER NOT NULL DEFAULT 1
            );
            CREATE INDEX IF NOT EXISTS idx_created_at ON clipboard_entries(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_last_copied ON clipboard_entries(last_copied DESC);
            CREATE INDEX IF NOT EXISTS idx_content_hash ON clipboard_entries(content_hash);
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = FULL;"
        )?;
        Ok(())
    }

    pub fn get_all_entries(&self) -> Result<Vec<ClipboardEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT content, created_at, last_copied FROM clipboard_entries ORDER BY last_copied DESC"
        )?;

        let entries = stmt.query_map([], |row| {
            let created_ts: i64 = row.get(1)?;
            let last_copied_ts: i64 = row.get(2)?;
            Ok(ClipboardEntry {
                content: row.get(0)?,
                created_at: DateTime::<Utc>::from_timestamp(created_ts, 0).unwrap_or_else(Utc::now),
                last_copied: DateTime::<Utc>::from_timestamp(last_copied_ts, 0).unwrap_or_else(Utc::now),
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    pub fn insert_entry(&self, content: &str, content_hash: &str) -> Result<i64> {
        let now = Utc::now().timestamp();

        match self.conn.execute(
            "INSERT INTO clipboard_entries (content, content_hash, created_at, last_copied, copy_count)
             VALUES (?1, ?2, ?3, ?4, 1)",
            params![content, content_hash, now, now],
        ) {
            Ok(_) => Ok(self.conn.last_insert_rowid()),
            Err(rusqlite::Error::SqliteFailure(_, Some(msg))) if msg.contains("UNIQUE constraint failed") => {
                self.conn.execute(
                    "UPDATE clipboard_entries SET last_copied = ?1, copy_count = copy_count + 1 WHERE content_hash = ?2",
                    params![now, content_hash],
                )?;
                let mut stmt = self.conn.prepare("SELECT id FROM clipboard_entries WHERE content_hash = ?1")?;
                let id = stmt.query_row(params![content_hash], |row| row.get(0))?;
                Ok(id)
            }
            Err(e) => Err(CliError::DatabaseError(e)),
        }
    }

    pub fn delete_entries_older_than_days(&self, days: i64) -> Result<i64> {
        let cutoff = Utc::now().timestamp() - (days * 86400);
        let rows = self.conn.execute(
            "DELETE FROM clipboard_entries WHERE created_at < ?1",
            params![cutoff],
        )?;
        Ok(rows as i64)
    }

    pub fn clear_all(&self) -> Result<i64> {
        let rows = self.conn.execute("DELETE FROM clipboard_entries", [])?;
        Ok(rows as i64)
    }

    pub fn count_entries(&self) -> Result<i64> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM clipboard_entries")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_size(&self) -> Result<u64> {
        let mut stmt = self.conn.prepare(
            "SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()"
        )?;
        let size: u64 = stmt.query_row([], |row| row.get(0))?;
        Ok(size)
    }

    pub fn delete_entry_by_content(&self, content: &str) -> Result<bool> {
        let hash = crate::clipboard::hash_content(content);
        let rows = self.conn.execute(
            "DELETE FROM clipboard_entries WHERE content_hash = ?1",
            params![hash],
        )?;
        Ok(rows > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_database_creation() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::open(tmp.path()).unwrap();
        assert_eq!(db.count_entries().unwrap(), 0);
    }

    #[test]
    fn test_insert_entry() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::open(tmp.path()).unwrap();
        let id = db.insert_entry("test content", "hash123").unwrap();
        assert!(id > 0);
        assert_eq!(db.count_entries().unwrap(), 1);
    }

    #[test]
    fn test_duplicate_entry_updates() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::open(tmp.path()).unwrap();

        let id1 = db.insert_entry("test content", "hash123").unwrap();
        let id2 = db.insert_entry("test content", "hash123").unwrap();

        assert_eq!(id1, id2);
        assert_eq!(db.count_entries().unwrap(), 1);
    }

    #[test]
    fn test_delete_entry() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::open(tmp.path()).unwrap();
        let hash = crate::clipboard::hash_content("test content");
        db.insert_entry("test content", &hash).unwrap();
        assert_eq!(db.count_entries().unwrap(), 1);

        let deleted = db.delete_entry_by_content("test content").unwrap();
        assert!(deleted);
        assert_eq!(db.count_entries().unwrap(), 0);
    }
}
