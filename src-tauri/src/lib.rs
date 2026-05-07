use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use image::ImageReader;
use std::fs;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::Manager;

mod clipboard;
mod db;
mod image_cache;
mod window;

static PASTE_LOCK: Mutex<()> = Mutex::new(());

fn open_clipboard_with_retry() -> Result<arboard::Clipboard, String> {
    let mut last_err = String::new();
    for _ in 0..12 {
        match arboard::Clipboard::new() {
            Ok(clipboard) => return Ok(clipboard),
            Err(e) => {
                last_err = e.to_string();
                thread::sleep(Duration::from_millis(15));
            }
        }
    }
    Err(format!(
        "Failed to access clipboard after retries: {last_err}"
    ))
}

fn set_text_with_retry(clipboard: &mut arboard::Clipboard, text: &str) -> Result<(), String> {
    let mut last_err = String::new();
    for _ in 0..8 {
        match clipboard.set_text(text.to_string()) {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_err = e.to_string();
                thread::sleep(Duration::from_millis(12));
            }
        }
    }
    Err(format!(
        "Failed to set text clipboard after retries: {last_err}"
    ))
}

fn set_image_with_retry(
    clipboard: &mut arboard::Clipboard,
    width: usize,
    height: usize,
    bytes: &[u8],
) -> Result<(), String> {
    let mut last_err = String::new();
    for _ in 0..8 {
        let image_data = arboard::ImageData {
            width,
            height,
            bytes: std::borrow::Cow::Borrowed(bytes),
        };
        match clipboard.set_image(image_data) {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_err = e.to_string();
                thread::sleep(Duration::from_millis(12));
            }
        }
    }
    Err(format!(
        "Failed to set image clipboard after retries: {last_err}"
    ))
}

#[tauri::command]
fn get_history(
    state: tauri::State<db::DbState>,
    search: Option<String>,
) -> Result<Vec<db::ClipboardItem>, String> {
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
fn update_text_content(
    state: tauri::State<db::DbState>,
    id: i64,
    content: String,
) -> Result<(), String> {
    let conn = state.conn.lock().unwrap();

    let item = db::get_item_by_id(&conn, id).map_err(|e| e.to_string())?;
    if item.content_type != "text" {
        return Err("Only text clipboard items can be edited".to_string());
    }

    db::update_item_content(&conn, id, &content).map_err(|e| e.to_string())
}

#[tauri::command]
fn hide_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn hide_for_paste(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn trigger_system_paste() -> Result<(), String> {
    thread::sleep(Duration::from_millis(40));

    use enigo::{Enigo, Key, Keyboard, Settings};

    let paste_modifier = if cfg!(target_os = "macos") {
        Key::Meta
    } else {
        Key::Control
    };

    let mut last_err = String::new();
    for _ in 0..3 {
        match Enigo::new(&Settings::default()) {
            Ok(mut enigo) => {
                enigo
                    .key(paste_modifier, enigo::Direction::Press)
                    .map_err(|e| e.to_string())?;
                thread::sleep(Duration::from_millis(8));

                enigo
                    .key(Key::Unicode('v'), enigo::Direction::Click)
                    .map_err(|e| e.to_string())?;

                thread::sleep(Duration::from_millis(8));
                enigo
                    .key(paste_modifier, enigo::Direction::Release)
                    .map_err(|e| e.to_string())?;
                return Ok(());
            }
            Err(e) => {
                last_err = e.to_string();
                thread::sleep(Duration::from_millis(20));
            }
        }
    }

    Err(format!(
        "Failed to initialize keyboard injector after retries: {last_err}"
    ))
}

fn load_cached_or_disk_image(item: &db::ClipboardItem) -> Result<image_cache::CachedImage, String> {
    let image_hash = &item.content;
    if let Some(cached) = image_cache::get_image(image_hash) {
        return Ok(cached);
    }

    let image_path = item
        .image_path
        .clone()
        .ok_or_else(|| "Image item missing image_path".to_string())?;

    let decoded = ImageReader::open(&image_path)
        .map_err(|e| format!("Failed to open image: {e}"))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {e}"))?
        .to_rgba8();

    let (width, height) = decoded.dimensions();
    image_cache::put_image_owned(
        image_hash,
        width as usize,
        height as usize,
        decoded.into_raw(),
    );

    image_cache::get_image(image_hash)
        .ok_or_else(|| "Failed to load image into memory cache".to_string())
}

#[tauri::command]
fn paste_item(
    app: tauri::AppHandle,
    state: tauri::State<db::DbState>,
    id: i64,
) -> Result<(), String> {
    let _guard = PASTE_LOCK
        .lock()
        .map_err(|_| "Paste lock poisoned".to_string())?;

    let item = {
        let conn = state.conn.lock().unwrap();
        db::get_item_by_id(&conn, id).map_err(|e| e.to_string())?
    };

    let mut clipboard = open_clipboard_with_retry()?;

    hide_for_paste(&app);

    if item.content_type == "image" {
        let cached = load_cached_or_disk_image(&item)?;
        let bytes_ref = cached.bytes.clone();
        set_image_with_retry(
            &mut clipboard,
            cached.width,
            cached.height,
            bytes_ref.as_slice(),
        )?;
    } else {
        set_text_with_retry(&mut clipboard, &item.content)?;
    }

    trigger_system_paste()?;
    Ok(())
}

#[tauri::command]
fn get_image_data(state: tauri::State<db::DbState>, id: i64) -> Result<String, String> {
    let item = {
        let conn = state.conn.lock().unwrap();
        db::get_item_by_id(&conn, id).map_err(|e| e.to_string())?
    };

    if item.content_type != "image" {
        return Err("Requested item is not an image".to_string());
    }

    let image_path = item
        .image_path
        .ok_or_else(|| "Image item missing image_path".to_string())?;

    let bytes = fs::read(&image_path).map_err(|e| format!("Failed to read image: {e}"))?;
    let encoded = STANDARD.encode(bytes);
    Ok(format!("data:image/png;base64,{encoded}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            fs::create_dir_all(&app_data_dir).expect("Failed to create app data dir");
            let db_path = app_data_dir.join("clipboard.db");

            let conn = db::init_db(db_path).expect("Failed to init db");
            app.manage(db::DbState {
                conn: Mutex::new(conn),
            });

            clipboard::start_listener(app.handle().clone());
            window::setup_shortcuts(app);

            use tauri_plugin_autostart::ManagerExt;
            let _ = app.autolaunch().enable();

            Ok(())
        })
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_history,
            toggle_pin,
            delete_item,
            update_text_content,
            paste_item,
            hide_window,
            get_image_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
