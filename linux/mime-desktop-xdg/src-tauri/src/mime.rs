use ini::Ini;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MimeAssociation {
    pub mime_type: String,
    pub default_apps: Vec<String>,
}

fn get_mimeapps_list_path() -> Option<PathBuf> {
    dirs::config_dir().map(|mut p| {
        p.push("mimeapps.list");
        p
    })
}

#[tauri::command]
pub async fn get_mime_associations() -> Result<Vec<MimeAssociation>, String> {
    let path = match get_mimeapps_list_path() {
        Some(p) => p,
        None => return Err("Could not find config dir".into())
    };
    
    if !path.exists() {
        return Ok(Vec::new());
    }

    let i = Ini::load_from_file(&path).map_err(|e| e.to_string())?;
    
    let mut mapping = HashMap::new();
    
    if let Some(section) = i.section(Some("Default Applications")) {
        for (k, v) in section.iter() {
            let apps: Vec<String> = v.split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            mapping.insert(k.to_string(), apps);
        }
    }
    
    // Also merge Added Associations if they are not in Default
    if let Some(section) = i.section(Some("Added Associations")) {
        for (k, v) in section.iter() {
            if !mapping.contains_key(k) {
                let apps: Vec<String> = v.split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                mapping.insert(k.to_string(), apps);
            }
        }
    }
    
    let mut results = Vec::new();
    for (k, v) in mapping {
        results.push(MimeAssociation {
            mime_type: k,
            default_apps: v,
        });
    }
    
    results.sort_by(|a, b| a.mime_type.cmp(&b.mime_type));
    Ok(results)
}

#[tauri::command]
pub async fn set_mime_association(mime_type: String, desktop_file: String) -> Result<(), String> {
    let path = match get_mimeapps_list_path() {
        Some(p) => p,
        None => return Err("Could not find config dir".into())
    };
    
    let mut i = match Ini::load_from_file(&path) {
        Ok(ini) => ini,
        Err(_) => Ini::new()
    };
    
    // Ini API: section can be updated or created
    let val = format!("{};", desktop_file);
    i.with_section(Some("Default Applications")).set(&mime_type, &val);
    i.with_section(Some("Added Associations")).set(&mime_type, &val);
        
    i.write_to_file(&path).map_err(|e| e.to_string())?;
    
    Ok(())
}
