use crate::error::{CliError, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    pub id: i64,
    pub content: String,
    pub content_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_copied: DateTime<Utc>,
    pub copy_count: i32,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create a database at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let conn = Connection::open(path).map_err(|e| {
            CliError::DatabaseError(e)
        })?;

        let db = Database { conn };
        db.initialize_schema()?;

        Ok(db)
    }

    /// Initialize database schema if it doesn't exist
    fn initialize_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS clipboard_entries (
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
            PRAGMA synchronous = NORMAL;
            "
        )?;

        Ok(())
    }

    /// Get all clipboard entries ordered by last_copied (newest first)
    pub fn get_all_entries(&self) -> Result<Vec<ClipboardEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, content_hash, created_at, last_copied, copy_count
             FROM clipboard_entries
             ORDER BY last_copied DESC"
        )?;

        let entries = stmt.query_map([], |row| {
            let created_ts: i64 = row.get(3)?;
            let last_copied_ts: i64 = row.get(4)?;

            Ok(ClipboardEntry {
                id: row.get(0)?,
                content: row.get(1)?,
                content_hash: row.get(2)?,
                created_at: DateTime::<Utc>::from_timestamp(created_ts, 0)
                    .unwrap_or_else(|| Utc::now()),
                last_copied: DateTime::<Utc>::from_timestamp(last_copied_ts, 0)
                    .unwrap_or_else(|| Utc::now()),
                copy_count: row.get(5)?,
            })
        })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// Get a single entry by ID
    pub fn get_entry(&self, id: i64) -> Result<Option<ClipboardEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, content_hash, created_at, last_copied, copy_count
             FROM clipboard_entries
             WHERE id = ?1"
        )?;

        let entry = stmt.query_row(params![id], |row| {
            let created_ts: i64 = row.get(3)?;
            let last_copied_ts: i64 = row.get(4)?;

            Ok(ClipboardEntry {
                id: row.get(0)?,
                content: row.get(1)?,
                content_hash: row.get(2)?,
                created_at: DateTime::<Utc>::from_timestamp(created_ts, 0)
                    .unwrap_or_else(|| Utc::now()),
                last_copied: DateTime::<Utc>::from_timestamp(last_copied_ts, 0)
                    .unwrap_or_else(|| Utc::now()),
                copy_count: row.get(5)?,
            })
        })
            .optional()?;

        Ok(entry)
    }

    /// Insert or update a clipboard entry
    pub fn insert_entry(&self, content: &str, content_hash: &str) -> Result<i64> {
        let now = Utc::now().timestamp();

        // Try to insert, if it fails due to duplicate, update instead
        match self.conn.execute(
            "INSERT INTO clipboard_entries (content, content_hash, created_at, last_copied, copy_count)
             VALUES (?1, ?2, ?3, ?4, 1)",
            params![content, content_hash, now, now],
        ) {
            Ok(_) => {
                // Get the inserted ID
                Ok(self.conn.last_insert_rowid())
            }
            Err(rusqlite::Error::SqliteFailure(_, Some(msg))) if msg.contains("UNIQUE constraint failed") => {
                // Content already exists, update it
                self.conn.execute(
                    "UPDATE clipboard_entries SET last_copied = ?1, copy_count = copy_count + 1 WHERE content_hash = ?2",
                    params![now, content_hash],
                )?;

                // Return the ID of the updated entry
                let mut stmt = self.conn.prepare(
                    "SELECT id FROM clipboard_entries WHERE content_hash = ?1"
                )?;

                let id = stmt.query_row(params![content_hash], |row| row.get(0))?;
                Ok(id)
            }
            Err(e) => Err(CliError::DatabaseError(e)),
        }
    }

    /// Delete an entry by ID
    pub fn delete_entry(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM clipboard_entries WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Delete entries older than the given number of days
    pub fn delete_entries_older_than_days(&self, days: i64) -> Result<i64> {
        let cutoff_timestamp = (Utc::now().timestamp() - (days * 86400)) as i64;

        let rows_deleted = self.conn.execute(
            "DELETE FROM clipboard_entries WHERE created_at < ?1",
            params![cutoff_timestamp],
        )?;

        Ok(rows_deleted as i64)
    }

    /// Clear all entries
    pub fn clear_all(&self) -> Result<i64> {
        let rows_deleted = self.conn.execute("DELETE FROM clipboard_entries", [])?;
        Ok(rows_deleted as i64)
    }

    /// Get the total number of entries
    pub fn count_entries(&self) -> Result<i64> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM clipboard_entries")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }

    /// Get the database size in bytes
    pub fn get_size(&self) -> Result<u64> {
        let mut stmt = self.conn.prepare(
            "SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()"
        )?;
        let size: u64 = stmt.query_row([], |row| row.get(0))?;
        Ok(size)
    }

    /// Get the last clipboard entry (most recent)
    pub fn get_last_entry(&self) -> Result<Option<ClipboardEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, content_hash, created_at, last_copied, copy_count
             FROM clipboard_entries
             ORDER BY last_copied DESC
             LIMIT 1"
        )?;

        let entry = stmt.query_row([], |row| {
            let created_ts: i64 = row.get(3)?;
            let last_copied_ts: i64 = row.get(4)?;

            Ok(ClipboardEntry {
                id: row.get(0)?,
                content: row.get(1)?,
                content_hash: row.get(2)?,
                created_at: DateTime::<Utc>::from_timestamp(created_ts, 0)
                    .unwrap_or_else(|| Utc::now()),
                last_copied: DateTime::<Utc>::from_timestamp(last_copied_ts, 0)
                    .unwrap_or_else(|| Utc::now()),
                copy_count: row.get(5)?,
            })
        })
            .optional()?;

        Ok(entry)
    }

    /// Check if entry exists by content hash
    pub fn entry_exists(&self, content_hash: &str) -> Result<bool> {
        let mut stmt = self.conn.prepare(
            "SELECT 1 FROM clipboard_entries WHERE content_hash = ?1 LIMIT 1"
        )?;

        let exists = stmt.exists(params![content_hash])?;
        Ok(exists)
    }

    /// Get database path
    pub fn path(&self) -> PathBuf {
        // Try to get from the database connection
        // For now, we'll store it separately in the struct if needed
        PathBuf::new()
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

        // Should be the same entry
        assert_eq!(id1, id2);
        // Should still only have 1 entry
        assert_eq!(db.count_entries().unwrap(), 1);

        // Copy count should be incremented
        let entry = db.get_entry(id1).unwrap().unwrap();
        assert_eq!(entry.copy_count, 2);
    }
}
