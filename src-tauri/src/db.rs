use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClipboardItem {
    pub id: i64,
    pub content: String,
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
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            pinned BOOLEAN DEFAULT 0
        )",
        [],
    )?;
    Ok(conn)
}

pub fn insert_item(conn: &Connection, content: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO clipboard (content, timestamp, pinned) 
         VALUES (?1, datetime('now', 'localtime'), 0) 
         ON CONFLICT(content) DO UPDATE SET timestamp=datetime('now', 'localtime')",
        params![content],
    )?;
    
    // Enforce limit of 500
    conn.execute(
        "DELETE FROM clipboard WHERE id NOT IN (
            SELECT id FROM clipboard ORDER BY pinned DESC, timestamp DESC LIMIT 500
        )",
        [],
    )?;
    Ok(())
}

pub fn get_history(conn: &Connection, search: Option<String>) -> Result<Vec<ClipboardItem>> {
    let mut query = "SELECT id, content, timestamp, pinned FROM clipboard".to_string();
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
                timestamp: row.get(2)?,
                pinned: row.get(3)?,
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
                timestamp: row.get(2)?,
                pinned: row.get(3)?,
            })
        })?;
        for item in item_iter {
            items.push(item?);
        }
    }

    Ok(items)
}

pub fn toggle_pin(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "UPDATE clipboard SET pinned = NOT pinned WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

pub fn delete_item(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM clipboard WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}
