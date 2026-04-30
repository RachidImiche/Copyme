use tauri::Manager;
use std::fs;
use std::sync::Mutex;

mod db;
mod clipboard;
mod window;

#[tauri::command]
fn get_history(state: tauri::State<db::DbState>, search: Option<String>) -> Result<Vec<db::ClipboardItem>, String> {
    let conn = state.conn.lock().unwrap();
    db::get_history(&conn, search).map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_pin(state: tauri::State<db::DbState>, id: i64) -> Result<(), String> {
    let conn = state.conn.lock().unwrap();
    db::toggle_pin(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_item(state: tauri::State<db::DbState>, id: i64) -> Result<(), String> {
    let conn = state.conn.lock().unwrap();
    db::delete_item(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn hide_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
fn paste_item(app: tauri::AppHandle, content: String) -> Result<(), String> {
    // 1. Set clipboard content
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let _ = clipboard.set_text(content);
    }
    
    // Hide the window first so focus returns to the previous app
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    
    // Small delay to ensure focus has shifted back to the previous window
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    // 2. Simulate Ctrl+V to paste into the previously focused window
    use enigo::{Enigo, Key, Keyboard, Settings};
    
    if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
        let _ = enigo.key(Key::Control, enigo::Direction::Press);
        let _ = enigo.key(Key::Unicode('v'), enigo::Direction::Click);
        let _ = enigo.key(Key::Control, enigo::Direction::Release);
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            fs::create_dir_all(&app_data_dir).expect("Failed to create app data dir");
            let db_path = app_data_dir.join("clipboard.db");
            
            let conn = db::init_db(db_path).expect("Failed to init db");
            app.manage(db::DbState { conn: Mutex::new(conn) });
            
            clipboard::start_listener(app.handle().clone());
            window::setup_shortcuts(app);
            
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_history, toggle_pin, delete_item, paste_item, hide_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
