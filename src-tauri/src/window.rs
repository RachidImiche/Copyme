use std::str::FromStr;
use tauri::{Manager, PhysicalPosition};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub fn setup_shortcuts(app: &mut tauri::App) {
    let super_v = Shortcut::from_str("Super+V").unwrap();
    let ctrl_shift_v = Shortcut::from_str("Ctrl+Shift+V").unwrap();

    let handler_super_v = super_v.clone();
    let handler_ctrl_shift_v = ctrl_shift_v.clone();

    app.handle()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    println!("Shortcut event: {:?}, state: {:?}", shortcut, event.state);
                    if event.state == ShortcutState::Pressed {
                        if shortcut == &handler_super_v || shortcut == &handler_ctrl_shift_v {
                            println!("Shortcut matched!");
                            if let Some(window) = app.get_webview_window("main") {
                                println!("Window found, showing...");
                                // Center window on primary monitor instead of following mouse
                                if let Ok(Some(monitor)) = app.primary_monitor() {
                                    let monitor_size = monitor.size();
                                    let monitor_pos = monitor.position();

                                    let window_width = 350.0;
                                    let window_height = 450.0;

                                    let center_x = monitor_pos.x as f64
                                        + (monitor_size.width as f64 - window_width) / 2.0;
                                    let center_y = monitor_pos.y as f64
                                        + (monitor_size.height as f64 - window_height) / 2.0;

                                    let center_pos =
                                        PhysicalPosition::new(center_x as i32, center_y as i32);
                                    let _ =
                                        window.set_position(tauri::Position::Physical(center_pos));
                                }
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.set_always_on_top(true);
                            } else {
                                println!("Window 'main' not found!");
                            }
                        }
                    }
                })
                .build(),
        )
        .expect("Failed to init global shortcut");

    if let Err(e) = app.global_shortcut().register(super_v) {
        println!("Failed to register Super+V: {}", e);
    }
    if let Err(e) = app.global_shortcut().register(ctrl_shift_v) {
        println!("Failed to register Ctrl+Shift+V: {}", e);
    }
}
