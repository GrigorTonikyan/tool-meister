// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

mod crud;
mod git_ops;
mod mime;
mod scanner;
mod validator;

#[tauri::command]
async fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_file(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if let Err(e) = std::fs::remove_file(p) {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            crud::delete_with_elevation(p)
        } else {
            Err(e.to_string())
        }
    } else {
        Ok(())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init());

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    builder
        .invoke_handler(tauri::generate_handler![
            greet,
            scanner::scan_desktop_files,
            validator::validate_desktop_file,
            git_ops::check_git_status,
            git_ops::git_init,
            git_ops::git_snapshot,
            git_ops::git_restore,
            git_ops::git_restore_to_commit,
            git_ops::list_snapshots,
            git_ops::get_snapshot_diff,
            scanner::get_user_applications_dir,
            crud::save_desktop_file,
            crud::rename_desktop_file,
            crud::auto_fix_desktop_file,
            mime::get_mime_associations,
            mime::set_mime_association,
            read_file,
            delete_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
