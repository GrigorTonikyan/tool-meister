use std::fs;
use std::path::Path;
use std::process::Command;

#[tauri::command]
pub async fn save_desktop_file(path: String, content: String) -> Result<(), String> {
    match fs::write(Path::new(&path), &content) {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                if let Some(mut local_dir) = crate::scanner::get_user_local_dir() {
                    if !local_dir.exists() {
                        let _ = fs::create_dir_all(&local_dir);
                    }
                    if let Some(file_name) = Path::new(&path).file_name() {
                        local_dir.push(file_name);
                        return fs::write(&local_dir, content)
                            .map_err(|inner_e| format!("Failed to shadow file: {}", inner_e));
                    }
                }
                Err(format!(
                    "Permission denied and couldn't create local shadow: {}",
                    e
                ))
            } else {
                Err(e.to_string())
            }
        }
    }
}

#[tauri::command]
pub async fn rename_desktop_file(old_path: String, new_path: String) -> Result<(), String> {
    fs::rename(&old_path, &new_path).map_err(|e| e.to_string())
}

pub fn delete_with_elevation(path: &Path) -> Result<(), String> {
    let output = Command::new("pkexec")
        .arg("rm")
        .arg(path)
        .output()
        .map_err(|e| format!("Failed to execute pkexec: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        if err.is_empty() {
            Err("Elevation failed or cancelled".to_string())
        } else {
            Err(err)
        }
    }
}

pub fn rename_with_elevation(old_path: &Path, new_path: &Path) -> Result<(), String> {
    let output = Command::new("pkexec")
        .arg("mv")
        .arg(old_path)
        .arg(new_path)
        .output()
        .map_err(|e| format!("Failed to execute pkexec: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        if err.is_empty() {
            Err("Elevation failed or cancelled".to_string())
        } else {
            Err(err)
        }
    }
}

#[tauri::command]
pub async fn auto_fix_desktop_file(
    absolute_path: String,
    filename: String,
) -> Result<String, String> {
    let path = Path::new(&absolute_path);
    if !path.exists() {
        return Err("File not found".into());
    }

    // Check for pure duplicates and see if we are the "loser" (redundant)
    let all_files = crate::scanner::scan_desktop_files()
        .await
        .unwrap_or_default();
    if let Some(this_info) = all_files.iter().find(|f| f.absolute_path == absolute_path) {
        if !this_info.duplicate_paths.is_empty() {
            let mut duplicate_group = Vec::new();
            for f in &all_files {
                if this_info.parsed_exec == f.parsed_exec && this_info.parsed_exec.is_some() {
                    duplicate_group.push(f.clone());
                }
            }

            if duplicate_group.len() > 1 {
                duplicate_group.sort_by(|a, b| {
                    let a_bad = crate::scanner::has_bad_pattern(&a.filename);
                    let b_bad = crate::scanner::has_bad_pattern(&b.filename);
                    if a_bad != b_bad {
                        return if a_bad {
                            std::cmp::Ordering::Greater
                        } else {
                            std::cmp::Ordering::Less
                        }; // Less is winner
                    }

                    let a_pri = a.location_type.priority();
                    let b_pri = b.location_type.priority();
                    if a_pri != b_pri {
                        return a_pri.cmp(&b_pri); // UserLocal=1 comes before System=4
                    }
                    if a.filename.len() != b.filename.len() {
                        return a.filename.len().cmp(&b.filename.len());
                    }
                    a.filename.cmp(&b.filename)
                });

                let winner = &duplicate_group[0];
                if winner.absolute_path != absolute_path {
                    // We are the loser! Delete.
                    if let Err(e) = fs::remove_file(path) {
                        if e.kind() == std::io::ErrorKind::PermissionDenied {
                            // Try elevated delete
                            delete_with_elevation(path)?;
                        } else {
                            return Err(format!("Failed to delete duplicate: {}", e));
                        }
                    }
                    return Ok(format!(
                        "Deleted duplicate: {} (kept {})",
                        filename, winner.filename
                    ));
                }
            }
        }
    }

    let mut new_path = path.to_path_buf();
    let mut renamed = false;

    // Fix filename if it has a trailing number bad pattern (e.g., app-1.desktop)
    let stem = filename.trim_end_matches(".desktop");
    if let Some(idx) = stem.rfind('-') {
        let suffix = &stem[idx + 1..];
        if !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
            let pure_name = &stem[..idx];
            let pure_filename = format!("{}.desktop", pure_name);
            let target_path = path.with_file_name(&pure_filename);

            if !target_path.exists() {
                // We want to rename it. Try standard rename first (for local files).
                if fs::rename(path, &target_path).is_ok() {
                    new_path = target_path;
                    renamed = true;
                } else {
                    // Rename failed (likely permission denied for system file).
                    rename_with_elevation(path, &target_path)?;
                    new_path = target_path;
                    renamed = true;
                }
            } else {
                // Target already exists, meaning this is a redundant duplicate (like app-1.desktop vs app.desktop)
                // We should clean it up by deleting it.
                if let Err(e) = fs::remove_file(path) {
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        delete_with_elevation(path)?;
                    } else {
                        return Err(format!("Failed to delete duplicate: {}", e));
                    }
                }
                return Ok(format!("Deleted duplicate file: {}", filename));
            }
        }
    }

    let content = fs::read_to_string(&new_path).map_err(|e| e.to_string())?;

    let bad_keys = vec!["Encoding=", "Actions=;", "TerminalOptions=", "OnlyShowIn=;"];

    let mut modified_lines = Vec::new();
    let mut content_changed = false;
    let mut has_header = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            content_changed = true;
            continue; // Strip empty lines
        }

        if trimmed == "[Desktop Entry]" {
            has_header = true;
        }

        let mut keep = true;
        for bk in &bad_keys {
            if trimmed.starts_with(bk) {
                keep = false;
                content_changed = true;
                break;
            }
        }
        if keep {
            modified_lines.push(line.to_string());
        }
    }

    // Ensure the header is at the top if missing or misaligned
    if !has_header {
        modified_lines.insert(0, "[Desktop Entry]".to_string());
        content_changed = true;
    } else if let Some(pos) = modified_lines
        .iter()
        .position(|l| l.trim() == "[Desktop Entry]")
    {
        if pos != 0 {
            let header = modified_lines.remove(pos);
            modified_lines.insert(0, header);
            content_changed = true;
        }
    }

    if content_changed || renamed {
        let new_content = modified_lines.join("\n") + "\n";

        match fs::write(&new_path, &new_content) {
            Ok(_) => {}
            Err(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    // Try to shadow the file to the user's local dir
                    if let Some(mut local_dir) = crate::scanner::get_user_local_dir() {
                        if !local_dir.exists() {
                            let _ = fs::create_dir_all(&local_dir);
                        }

                        let file_name = new_path.file_name().unwrap_or_default();
                        local_dir.push(file_name);

                        fs::write(&local_dir, &new_content)
                            .map_err(|inner_e| format!("Failed to shadow file: {}", inner_e))?;
                        new_path = local_dir;
                        renamed = true; // Count as a 'move' essentially since the user interacts with the new shadowed one
                    } else {
                        return Err(format!(
                            "Permission denied and could not find local app dir: {}",
                            e
                        ));
                    }
                } else {
                    return Err(e.to_string());
                }
            }
        }
    }

    if renamed || content_changed {
        Ok(new_path.to_string_lossy().to_string())
    } else {
        Ok("No changes were necessary".into())
    }
}
