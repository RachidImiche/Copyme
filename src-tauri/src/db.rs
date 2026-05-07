use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClipboardItem {
    pub id: i64,
    pub content: String,
    pub content_type: String,
    pub image_path: Option<String>,
    pub timestamp: String,
    pub pinned: bool,
}

pub struct DbState {
    pub conn: Mutex<Connection>,
}

pub fn init_db(db_path: PathBuf) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS clipboard (
            id INTEGER PRIMARY KEY,
            content TEXT UNIQUE,
            content_type TEXT NOT NULL DEFAULT 'text',
            image_path TEXT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            pinned BOOLEAN DEFAULT 0
        )",
        [],
    )?;

    let columns = {
        let mut stmt = conn.prepare("PRAGMA table_info(clipboard)")?;
        let names = stmt
            .query_map([], |row| row.get::<usize, String>(1))?
            .collect::<std::result::Result<Vec<String>, _>>()?;
        names
    };

    if !columns.iter().any(|c| c == "content_type") {
        conn.execute(
            "ALTER TABLE clipboard ADD COLUMN content_type TEXT NOT NULL DEFAULT 'text'",
            [],
        )?;
    }

    if !columns.iter().any(|c| c == "image_path") {
        conn.execute("ALTER TABLE clipboard ADD COLUMN image_path TEXT", [])?;
    }

    Ok(conn)
}

pub fn insert_text_item(conn: &Connection, content: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO clipboard (content, content_type, image_path, timestamp, pinned)
         VALUES (?1, 'text', NULL, datetime('now', 'localtime'), 0)
         ON CONFLICT(content) DO UPDATE SET
            timestamp=datetime('now', 'localtime'),
            content_type='text',
            image_path=NULL",
        params![content],
    )?;

    enforce_limit(conn)
}

pub fn insert_image_item(conn: &Connection, hash: &str, image_path: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO clipboard (content, content_type, image_path, timestamp, pinned)
         VALUES (?1, 'image', ?2, datetime('now', 'localtime'), 0)
         ON CONFLICT(content) DO UPDATE SET
            timestamp=datetime('now', 'localtime'),
            content_type='image',
            image_path=excluded.image_path",
        params![hash, image_path],
    )?;

    enforce_limit(conn)
}

fn enforce_limit(conn: &Connection) -> Result<()> {
    conn.execute(
        "DELETE FROM clipboard WHERE id NOT IN (
            SELECT id FROM clipboard ORDER BY pinned DESC, timestamp DESC LIMIT 500
        )",
        [],
    )?;
    Ok(())
}

pub fn get_history(conn: &Connection, search: Option<String>) -> Result<Vec<ClipboardItem>> {
    let mut query =
        "SELECT id, content, content_type, image_path, timestamp, pinned FROM clipboard"
            .to_string();
    let has_search = search.as_ref().map(|s| !s.is_empty()).unwrap_or(false);

    if has_search {
        query.push_str(" WHERE content LIKE ?1");
    }

    query.push_str(" ORDER BY pinned DESC, timestamp DESC LIMIT 500");

    let mut stmt = conn.prepare(&query)?;
    let mut items = Vec::new();

    if has_search {
        let search_term = format!("%{}%", search.unwrap());
        let item_iter = stmt.query_map(params![search_term], |row| {
            Ok(ClipboardItem {
                id: row.get(0)?,
                content: row.get(1)?,
                content_type: row.get(2)?,
                image_path: row.get(3)?,
                timestamp: row.get(4)?,
                pinned: row.get(5)?,
            })
        })?;
        for item in item_iter {
            items.push(item?);
        }
    } else {
        let item_iter = stmt.query_map([], |row| {
            Ok(ClipboardItem {
                id: row.get(0)?,
                content: row.get(1)?,
                content_type: row.get(2)?,
                image_path: row.get(3)?,
                timestamp: row.get(4)?,
                pinned: row.get(5)?,
            })
        })?;
        for item in item_iter {
            items.push(item?);
        }
    }

    Ok(items)
}

pub fn get_item_by_id(conn: &Connection, id: i64) -> Result<ClipboardItem> {
    conn.query_row(
        "SELECT id, content, content_type, image_path, timestamp, pinned FROM clipboard WHERE id = ?1",
        params![id],
        |row| {
            Ok(ClipboardItem {
                id: row.get(0)?,
                content: row.get(1)?,
                content_type: row.get(2)?,
                image_path: row.get(3)?,
                timestamp: row.get(4)?,
                pinned: row.get(5)?,
            })
        },
    )
}

pub fn toggle_pin(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "UPDATE clipboard SET pinned = NOT pinned WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

pub fn delete_item(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM clipboard WHERE id = ?1", params![id])?;
    Ok(())
}
