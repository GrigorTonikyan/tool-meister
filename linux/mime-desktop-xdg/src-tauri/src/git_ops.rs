use git2::{Repository, Signature};
use std::path::PathBuf;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GitSnapshot {
    pub commit_id: String,
    pub message: String,
    pub timestamp: i64,
    pub author: String,
}

fn get_repo_path() -> Result<PathBuf, String> {
    crate::scanner::get_user_local_dir().ok_or("Could not find user local applications directory".to_string())
}

#[tauri::command]
pub async fn check_git_status() -> Result<bool, String> {
    let repo_path = get_repo_path()?;
    Ok(Repository::open(&repo_path).is_ok())
}

#[tauri::command]
pub async fn git_init() -> Result<(), String> {
    let repo_path = get_repo_path()?;
    
    if !repo_path.exists() {
        std::fs::create_dir_all(&repo_path).map_err(|e| e.to_string())?;
    }
    
    Repository::init(&repo_path).map_err(|e| e.message().to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn git_snapshot(message: String) -> Result<(), String> {
    let repo_path = get_repo_path()?;
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    
    let mut index = repo.index().map_err(|e| e.message().to_string())?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None).map_err(|e| e.message().to_string())?;
    index.write().map_err(|e| e.message().to_string())?;
    
    let oid = index.write_tree().map_err(|e| e.message().to_string())?;
    let tree = repo.find_tree(oid).map_err(|e| e.message().to_string())?;
    
    let sig = Signature::now("Desktop Linter", "linter@localhost").map_err(|e| e.message().to_string())?;
    
    let parent_commit = match repo.head() {
        Ok(head) => {
            let target = head.target().unwrap();
            Some(repo.find_commit(target).map_err(|e| e.message().to_string())?)
        },
        Err(_) => None,
    };
    
    if let Some(parent) = parent_commit {
        repo.commit(Some("HEAD"), &sig, &sig, &message, &tree, &[&parent]).map_err(|e| e.message().to_string())?;
    } else {
        repo.commit(Some("HEAD"), &sig, &sig, &message, &tree, &[]).map_err(|e| e.message().to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn git_restore() -> Result<(), String> {
    let repo_path = get_repo_path()?;
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    
    let head = repo.head().map_err(|e| e.message().to_string())?;
    let target = head.target().unwrap();
    let commit = repo.find_commit(target).map_err(|e| e.message().to_string())?;
    let tree = commit.tree().map_err(|e| e.message().to_string())?;
    
    let mut cb = git2::build::CheckoutBuilder::new();
    cb.force();
    repo.checkout_tree(tree.as_object(), Some(&mut cb)).map_err(|e| e.message().to_string())?;
    
    repo.reset(commit.as_object(), git2::ResetType::Hard, None).map_err(|e| e.message().to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn git_restore_to_commit(commit_id: String) -> Result<(), String> {
    let repo_path = get_repo_path()?;
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    
    let oid = git2::Oid::from_str(&commit_id).map_err(|e| e.message().to_string())?;
    let commit = repo.find_commit(oid).map_err(|e| e.message().to_string())?;
    let tree = commit.tree().map_err(|e| e.message().to_string())?;
    
    let mut cb = git2::build::CheckoutBuilder::new();
    cb.force();
    repo.checkout_tree(tree.as_object(), Some(&mut cb)).map_err(|e| e.message().to_string())?;
    
    repo.reset(commit.as_object(), git2::ResetType::Hard, None).map_err(|e| e.message().to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn list_snapshots() -> Result<Vec<GitSnapshot>, String> {
    let repo_path = get_repo_path()?;
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    
    let mut snapshots = Vec::new();
    let mut revwalk = repo.revwalk().map_err(|e| e.message().to_string())?;
    if revwalk.push_head().is_err() {
        return Ok(snapshots); // Empty or no commits yet
    }
    
    for id in revwalk {
        if let Ok(oid) = id {
            if let Ok(commit) = repo.find_commit(oid) {
                snapshots.push(GitSnapshot {
                    commit_id: commit.id().to_string(),
                    message: commit.message().unwrap_or("").to_string(),
                    timestamp: commit.time().seconds(),
                    author: commit.author().name().unwrap_or("Unknown").to_string(),
                });
            }
        }
    }
    
    Ok(snapshots)
}

#[tauri::command]
pub async fn get_snapshot_diff(commit_id: String) -> Result<String, String> {
    let repo_path = get_repo_path()?;
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    
    let oid = git2::Oid::from_str(&commit_id).map_err(|e| e.message().to_string())?;
    let commit = repo.find_commit(oid).map_err(|e| e.message().to_string())?;
    let tree = commit.tree().map_err(|e| e.message().to_string())?;
    
    let parent_tree = if commit.parent_count() > 0 {
        let parent = commit.parent(0).map_err(|e| e.message().to_string())?;
        Some(parent.tree().map_err(|e| e.message().to_string())?)
    } else {
        None
    };
    
    let mut diff_opts = git2::DiffOptions::new();
    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut diff_opts))
        .map_err(|e| e.message().to_string())?;
    
    let mut diff_str = String::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let origin = line.origin();
        match origin {
            '+' | '-' | ' ' => {
                diff_str.push(origin);
            }
            _ => {}
        }
        if let Ok(content) = std::str::from_utf8(line.content()) {
            diff_str.push_str(content);
        }
        true
    }).map_err(|e| e.message().to_string())?;
    
    Ok(diff_str)
}
