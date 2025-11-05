use crate::core::{FsObjectStore, Oid};
use crate::index::Index;
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::collections::HashMap;

pub fn execute(repo: &Path) -> Result<()> {
    let store = FsObjectStore::new(repo);
    let status = get_status(repo, &store)?;
    print_status(&status);
    Ok(())
}

#[derive(Debug, Default)]
pub struct Status {
    pub staged_changes: Vec<String>,
    pub unstaged_changes: Vec<String>,
    pub untracked_files: Vec<String>,
}

fn get_status(repo: &Path, store: &FsObjectStore) -> Result<Status> {
    let mut status = Status::default();
    let index = Index::load(repo)?;
    
    // Get the last commit tree to compare against
    let head_oid = get_head_commit_oid(repo)?;
    let last_commit_tree = if let Some(oid) = head_oid {
        get_commit_tree(&oid, store)?
    } else {
        HashMap::new() // No commits yet
    };

    // Track all files in working directory
    let working_files = find_working_directory_files(repo)?;
    
    // Check staged changes (files in index)
    for indexed_file in index.entries.keys() {
        if let Some(commit_oid) = last_commit_tree.get(indexed_file) {
            let index_oid = &index.entries[indexed_file].oid;
            if index_oid != commit_oid {
                status.staged_changes.push(format!("modified: {}", indexed_file));
            }
        } else {
            status.staged_changes.push(format!("new file: {}", indexed_file));
        }
    }

    // Check unstaged changes and untracked files
    for file_path in working_files {
        if index.entries.contains_key(&file_path) {
            // File is tracked - check if modified
            let current_oid = compute_file_oid(repo, &file_path)?;
            let indexed_oid = &index.entries[&file_path].oid;
            
            if &current_oid != indexed_oid {
                status.unstaged_changes.push(format!("modified: {}", file_path));
            }
        } else {
            // File is untracked
            status.untracked_files.push(file_path);
        }
    }

    Ok(status)
}

fn get_head_commit_oid(repo: &Path) -> Result<Option<String>> {
    let head_file = crate::core::head_file(repo);
    if !head_file.exists() {
        return Ok(None);
    }
    
    let content = fs::read_to_string(head_file)?;
    if content.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(content.trim().to_string()))
    }
}

fn get_commit_tree(commit_oid: &str, store: &FsObjectStore) -> Result<HashMap<String, String>> {
    let oid = Oid::from_hex(commit_oid)?;
    let mut tree_map = HashMap::new();
    
    if let Some(crate::core::Object::Commit(commit)) = store.get(&oid)? {
        let tree_oid = Oid::from_hex(&commit.tree)?;
        if let Some(crate::core::Object::Tree(entries)) = store.get(&tree_oid)? {
            for entry in entries {
                tree_map.insert(entry.name, entry.oid.to_hex());
            }
        }
    }
    
    Ok(tree_map)
}

fn find_working_directory_files(repo: &Path) -> Result<Vec<String>> {
    let mut files = Vec::new();
    
    for entry in walkdir::WalkDir::new(repo)
        .min_depth(1)
        .max_depth(10) // Reasonable depth
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        // Skip .minigit directory and other ignores
        if path.starts_with(repo.join(".minigit")) {
            continue;
        }
        
        if path.is_file() {
            if let Ok(relative_path) = path.strip_prefix(repo) {
                files.push(relative_path.to_string_lossy().to_string());
            }
        }
    }
    
    Ok(files)
}

fn compute_file_oid(repo: &Path, file_path: &str) -> Result<String> {
    let abs_path = repo.join(file_path);
    let content = fs::read(&abs_path)?;
    let oid = crate::core::blob_oid(&content);
    Ok(oid.to_hex())
}

fn print_status(status: &Status) {
    if !status.staged_changes.is_empty() {
        println!("Changes to be committed:");
        println!("  (use 'minigit commit' to save changes)");
        for change in &status.staged_changes {
            println!("    {}", change);
        }
        println!();
    }

    if !status.unstaged_changes.is_empty() {
        println!("Changes not staged for commit:");
        println!("  (use 'minigit add <file>' to update what will be committed)");
        for change in &status.unstaged_changes {
            println!("    {}", change);
        }
        println!();
    }

    if !status.untracked_files.is_empty() {
        println!("Untracked files:");
        println!("  (use 'minigit add <file>' to include in what will be committed)");
        for file in &status.untracked_files {
            println!("    {}", file);
        }
        println!();
    }

    if status.staged_changes.is_empty() && status.unstaged_changes.is_empty() && status.untracked_files.is_empty() {
        println!("Nothing to commit, working tree clean");
    }
}