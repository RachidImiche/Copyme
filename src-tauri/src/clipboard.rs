use arboard::Clipboard;
use image::{ImageBuffer, Rgba};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

use crate::db;
use crate::image_cache;

fn hash_image_data(width: usize, height: usize, bytes: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    width.hash(&mut hasher);
    height.hash(&mut hasher);
    bytes.hash(&mut hasher);
    format!("img_{:x}", hasher.finish())
}

fn save_clipboard_image(
    app_handle: &AppHandle,
    hash: &str,
    width: usize,
    height: usize,
    bytes: &[u8],
) -> Result<String, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;
    let images_dir = app_data_dir.join("clipboard_images");

    fs::create_dir_all(&images_dir).map_err(|e| format!("Failed to create image dir: {e}"))?;

    let file_name = format!("{hash}.png");
    let image_path = images_dir.join(file_name);

    if !image_path.exists() {
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_raw(width as u32, height as u32, bytes.to_vec())
                .ok_or_else(|| "Invalid image buffer dimensions".to_string())?;

        img.save(&image_path)
            .map_err(|e| format!("Failed to save image: {e}"))?;
    }

    Ok(image_path.to_string_lossy().to_string())
}

pub fn start_listener(app_handle: AppHandle) {
    thread::spawn(move || {
        let mut clipboard = match Clipboard::new() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to create clipboard: {e}");
                return;
            }
        };

        let mut last_text = clipboard.get_text().unwrap_or_default();
        let mut last_image_hash = String::new();

        loop {
            let mut updated = false;

            if let Ok(content) = clipboard.get_text() {
                if !content.is_empty() && content != last_text {
                    last_text = content.clone();
                    if let Some(state) = app_handle.try_state::<db::DbState>() {
                        if let Ok(conn) = state.conn.lock() {
                            if db::insert_text_item(&conn, &content).is_ok() {
                                updated = true;
                            }
                        }
                    }
                }
            }

            if let Ok(image_data) = clipboard.get_image() {
                let image_hash = hash_image_data(
                    image_data.width,
                    image_data.height,
                    image_data.bytes.as_ref(),
                );

                if image_hash != last_image_hash {
                    image_cache::put_image(
                        &image_hash,
                        image_data.width,
                        image_data.height,
                        image_data.bytes.as_ref(),
                    );

                    if let Ok(image_path) = save_clipboard_image(
                        &app_handle,
                        &image_hash,
                        image_data.width,
                        image_data.height,
                        image_data.bytes.as_ref(),
                    ) {
                        if let Some(state) = app_handle.try_state::<db::DbState>() {
                            if let Ok(conn) = state.conn.lock() {
                                if db::insert_image_item(&conn, &image_hash, &image_path).is_ok() {
                                    last_image_hash = image_hash;
                                    updated = true;
                                }
                            }
                        }
                    }
                }
            }

            if updated {
                let _ = app_handle.emit("clipboard-update", ());
            }

            thread::sleep(Duration::from_millis(500));
        }
    });
}
