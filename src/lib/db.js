import Database from 'better-sqlite3';
import { existsSync } from 'fs';
import { getDbPath } from './config.js';

export function dbExists() {
  return existsSync(getDbPath());
}

let db = null;

function getDb() {
  if (!db) {
    db = new Database(getDbPath(), { readonly: true });
  }
  return db;
}

export function getEntries() {
  return getDb().prepare(`
    SELECT id, content, content_hash, first_copied, last_copied, copy_count
    FROM clipboard_history
    ORDER BY last_copied DESC
  `).all().map(row => ({
    id: row.id,
    content: row.content,
    lastCopied: row.last_copied,
    copyCount: row.copy_count,
  }));
}

export function closeDb() {
  if (db) {
    db.close();
    db = null;
  }
}
