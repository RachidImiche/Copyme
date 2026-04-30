use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState, Code, Modifiers};
use std::str::FromStr;

pub fn setup_shortcuts(app: &mut tauri::App) {
    let ctrl_v = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV);
    
    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    if shortcut == &Shortcut::from_str("Super+V").unwrap() || shortcut == &Shortcut::from_str("Ctrl+Shift+V").unwrap() {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            })
            .build(),
    ).unwrap();
    
    app.global_shortcut().register(Shortcut::from_str("Super+V").unwrap()).unwrap();
    app.global_shortcut().register(Shortcut::from_str("Ctrl+Shift+V").unwrap()).unwrap();
}
