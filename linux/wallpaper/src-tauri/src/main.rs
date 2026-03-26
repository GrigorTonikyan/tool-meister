// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Builder, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};

// This is a command we will call from our React frontend
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn main() {
    // --- System Tray Menu ---
    // This is our primary interface for the user
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new(
            "next_wallpaper".to_string(),
            "Next Wallpaper",
        ))
        .add_item(CustomMenuItem::new(
            "approve_wallpaper".to_string(),
            "Approve Current",
        ))
        .add_item(CustomMenuItem::new(
            "reject_wallpaper".to_string(),
            "Reject Current",
        ))
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit".to_string(), "Quit"));

    let system_tray = SystemTray::new().with_menu(tray_menu);

    Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => {
                // We emit an event to the frontend when a tray item is clicked
                // The frontend will handle the logic.
                app.emit_all(&id, ()).unwrap();
                match id.as_str() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            // We will add more commands here like `approve_current`, `fetch_new`, etc.
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
