use arboard::Clipboard;
use std::time::Duration;
use std::thread;
use tauri::{AppHandle, Emitter, Manager};
use crate::db;

pub fn start_listener(app_handle: AppHandle) {
    thread::spawn(move || {
        let mut clipboard = Clipboard::new().unwrap();
        let mut last_content = clipboard.get_text().unwrap_or_default();

        loop {
            if let Ok(content) = clipboard.get_text() {
                if !content.is_empty() && content != last_content {
                    last_content = content.clone();
                    if let Some(state) = app_handle.try_state::<db::DbState>() {
                        if let Ok(conn) = state.conn.lock() {
                            if db::insert_item(&conn, &content).is_ok() {
                                // Emit event to update frontend
                                let _ = app_handle.emit("clipboard-update", ());
                            }
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(500));
        }
    });
}
