use serde::{Deserialize, Serialize};
use std::process::Command;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub exec_exists: bool,
    pub exec_command: Option<String>,
}

#[tauri::command]
pub async fn validate_desktop_file(absolute_path: String) -> Result<ValidationResult, String> {
    if !Path::new(&absolute_path).exists() {
        return Err("File does not exist".to_string());
    }

    // Run desktop-file-validate
    let output = Command::new("desktop-file-validate")
        .arg(&absolute_path)
        .output();

    let mut is_valid = false;
    let mut errors = Vec::new();

    match output {
        Ok(out) => {
            if out.status.success() {
                is_valid = true;
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                
                let all_output = format!("{}\n{}", stdout, stderr);
                errors = all_output
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| l.replace(&absolute_path, "file"))
                    .collect();
            }
        }
        Err(e) => {
            errors.push(format!("Failed to run desktop-file-validate: {}", e));
        }
    }

    let mut exec_exists = true;
    let mut exec_command = None;
    if let Ok(content) = std::fs::read_to_string(&absolute_path) {
        for line in content.lines() {
            if line.starts_with("Exec=") {
                let cmd = line.trim_start_matches("Exec=").trim();
                
                if let Some(args) = shlex::split(cmd) {
                    let mut bin_to_check = None;
                    for arg in args {
                        if arg == "env" {
                            continue;
                        }
                        if arg.contains('=') && !arg.starts_with('/') {
                            continue;
                        }
                        bin_to_check = Some(arg.clone());
                        break;
                    }
                    
                    if let Some(bin) = bin_to_check {
                        exec_command = Some(bin.clone());
                        exec_exists = which::which(&bin).is_ok();
                    } else {
                        exec_exists = false;
                    }
                } else {
                    exec_exists = false;
                }
                break;
            }
        }
    }

    Ok(ValidationResult {
        is_valid,
        errors,
        exec_exists,
        exec_command,
    })
}
