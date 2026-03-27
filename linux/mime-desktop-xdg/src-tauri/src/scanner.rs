use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopFile {
    pub absolute_path: String,
    pub filename: String,
    pub location_type: LocationType,
    pub is_shadowed: bool,
    pub shadows_paths: Vec<String>,
    pub parsed_name: Option<String>,
    pub parsed_exec: Option<String>,
    pub has_bad_pattern: bool,
    pub duplicate_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LocationType {
    UserLocal,
    System,
    SystemLocal,
    Flatpak,
    Other,
}

impl LocationType {
    pub fn priority(&self) -> u8 {
        match self {
            LocationType::UserLocal => 1,
            LocationType::SystemLocal => 2,
            LocationType::Flatpak => 3,
            LocationType::System => 4,
            LocationType::Other => 5,
        }
    }
}

#[tauri::command]
pub fn get_user_applications_dir() -> Result<String, String> {
    get_user_local_dir()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "Could not find user applications directory".to_string())
}

pub fn get_user_local_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|mut p| {
        p.push("applications");
        p
    })
}

#[derive(Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub loc_type: LocationType,
    pub name: Option<String>,
    pub exec: Option<String>,
    pub is_hidden: bool,
}

pub fn has_bad_pattern(filename: &str) -> bool {
    let stem = filename.trim_end_matches(".desktop");
    if let Some(idx) = stem.rfind('-') {
        let suffix = &stem[idx+1..];
        if !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
            return true;
        }
    }
    false
}

fn parse_basic_info(path: &Path) -> (Option<String>, Option<String>, bool) {
    let mut name = None;
    let mut exec = None;
    let mut is_hidden = false;
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if name.is_none() && trimmed.starts_with("Name=") {
                name = Some(trimmed.trim_start_matches("Name=").to_string());
            }
            if exec.is_none() && trimmed.starts_with("Exec=") {
                exec = Some(trimmed.trim_start_matches("Exec=").to_string());
            }
            if trimmed == "Hidden=true" || trimmed == "NoDisplay=true" {
                is_hidden = true;
            }
        }
    }
    (name, exec, is_hidden)
}

#[tauri::command]
pub async fn scan_desktop_files() -> Result<Vec<DesktopFile>, String> {
    let mut all_files: Vec<FileInfo> = Vec::new();

    let paths = vec![
        (get_user_local_dir(), LocationType::UserLocal),
        (Some(PathBuf::from("/usr/local/share/applications")), LocationType::SystemLocal),
        (Some(PathBuf::from("/usr/share/applications")), LocationType::System),
        (Some(PathBuf::from("/var/lib/flatpak/exports/share/applications")), LocationType::Flatpak),
    ];

    for (dir_opt, loc_type) in paths {
        if let Some(dir) = dir_opt {
            if dir.exists() && dir.is_dir() {
                for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                        let (name, exec, is_hidden) = parse_basic_info(path);
                        all_files.push(FileInfo {
                            path: path.to_path_buf(),
                            loc_type: loc_type.clone(),
                            name,
                            exec,
                            is_hidden,
                        });
                    }
                }
            }
        }
    }

    let mut by_filename: HashMap<String, Vec<FileInfo>> = HashMap::new();
    let mut by_exec: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for info in &all_files {
        if let Some(filename) = info.path.file_name().and_then(|n| n.to_str()) {
            by_filename.entry(filename.to_string()).or_default().push(info.clone());
        }
    }
    
    // Sort all grouped files by filename to determine shadowing
    for appearances in by_filename.values_mut() {
        appearances.sort_by_key(|info| info.loc_type.priority());
    }

    for appearances in by_filename.values() {
        if let Some(primary) = appearances.first() {
            if !primary.is_hidden {
                for info in appearances {
                    if let Some(exec) = &info.exec {
                        // Only add to by_exec if the primary file isn't hiding this whole chain
                        by_exec.entry(exec.clone()).or_default().push(info.path.clone());
                    }
                }
            }
        }
    }

    let mut result = Vec::new();

    for (filename, mut appearances) in by_filename {
        appearances.sort_by_key(|info| info.loc_type.priority());

        for (i, info) in appearances.iter().enumerate() {
            let is_shadowed = i > 0;
            let mut shadows_paths = Vec::new();
            
            if i == 0 && appearances.len() > 1 {
                for other_info in appearances.iter().skip(1) {
                    shadows_paths.push(other_info.path.to_string_lossy().to_string());
                }
            }
            
            let mut duplicate_paths = Vec::new();
            if let Some(exec) = &info.exec {
                if let Some(dupes) = by_exec.get(exec) {
                    for p in dupes {
                        if p != &info.path && p.file_name() != info.path.file_name() {
                            duplicate_paths.push(p.to_string_lossy().to_string());
                        }
                    }
                }
            }

            result.push(DesktopFile {
                absolute_path: info.path.to_string_lossy().to_string(),
                filename: filename.clone(),
                location_type: info.loc_type.clone(),
                is_shadowed,
                shadows_paths,
                parsed_name: info.name.clone(),
                parsed_exec: info.exec.clone(),
                has_bad_pattern: has_bad_pattern(&filename),
                duplicate_paths,
            });
        }
    }

    Ok(result)
}
