use crate::core::{FsObjectStore, Oid};
use crate::index::Index;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn execute(repo: &Path) -> Result<()> {
    let store = FsObjectStore::new(repo);
    let index = Index::load(repo)?;
    
    // Get the last commit to compare against
    let head_oid = get_head_commit_oid(repo)?;
    
    println!("Diff against: {}", head_oid.as_deref().unwrap_or("(no commits)"));
    println!();

    for (file_path, index_entry) in &index.entries {
        show_file_diff(repo, &store, file_path, &index_entry.oid, head_oid.as_deref())?;
    }
    
    Ok(())
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
fn show_file_diff(
    repo: &Path,
    store: &FsObjectStore,
    file_path: &str,
    _indexed_oid: &str,
    head_commit_oid: Option<&str>,
) -> Result<()> {
    let file_path_buf = repo.join(file_path);
    
    // Skip if file doesn't exist
    if !file_path_buf.exists() {
        return Ok(());
    }

    // Read file as bytes first to check if it's text
    let current_content_bytes = fs::read(&file_path_buf)?;
    
    // Try to read as UTF-8, if it fails, treat as binary
    let current_content = match String::from_utf8(current_content_bytes.clone()) {
        Ok(text) => text,
        Err(_) => {
            println!("--- a/{}", file_path);
            println!("+++ b/{}", file_path);
            println!("Binary files differ");
            println!();
            return Ok(());
        }
    };

    let current_lines: Vec<String> = current_content.lines().map(|s| s.to_string()).collect();
    
    // Get the committed version (if any)
    let committed_content_bytes = if let Some(commit_oid) = head_commit_oid {
        get_file_content_from_commit_bytes(commit_oid, file_path, store).ok()
    } else {
        None
    };
    
    let committed_lines: Vec<String> = if let Some(bytes) = committed_content_bytes {
        match String::from_utf8(bytes) {
            Ok(text) => text.lines().map(|s| s.to_string()).collect(),
            Err(_) => {
                println!("--- a/{}", file_path);
                println!("+++ b/{}", file_path);
                println!("Binary files differ");
                println!();
                return Ok(());
            }
        }
    } else {
        Vec::new() // File didn't exist in last commit
    };
    
    // Simple diff algorithm - compare line by line
    let diff = compute_diff(&committed_lines, &current_lines);
    
    if !diff.is_empty() {
        println!("--- a/{}", file_path);
        println!("+++ b/{}", file_path);
        
        for change in diff {
            match change {
                DiffLine::Added(line, num) => println!("+{}: {}", num, line),
                DiffLine::Removed(line, num) => println!("-{}: {}", num, line),
                DiffLine::Unchanged(line, num) => println!(" {}: {}", num, line),
            }
        }
        println!();
    }
    
    Ok(())
}
//Helper function
fn get_file_content_from_commit_bytes(commit_oid: &str, file_path: &str, store: &FsObjectStore) -> Result<Vec<u8>> {
    let commit_oid = Oid::from_hex(commit_oid)?;
    
    if let Some(crate::core::Object::Commit(commit)) = store.get(&commit_oid)? {
        let tree_oid = Oid::from_hex(&commit.tree)?;
        
        if let Some(crate::core::Object::Tree(entries)) = store.get(&tree_oid)? {
            for entry in entries {
                if entry.name == file_path {
                    if let Some(crate::core::Object::Blob(content)) = store.get(&entry.oid)? {
                        return Ok(content);
                    }
                }
            }
        }
    }
    
    Err(anyhow::anyhow!("File not found in commit: {}", file_path))
}


#[derive(Debug)]
enum DiffLine {
    Added(String, usize),    // Line added, line number in new file
    Removed(String, usize),  // Line removed, line number in old file  
    Unchanged(String, usize), // Line unchanged, line number
}

fn compute_diff(old_lines: &[String], new_lines: &[String]) -> Vec<DiffLine> {
    let mut diff = Vec::new();
    let mut i = 0;
    let mut j = 0;
    
    while i < old_lines.len() || j < new_lines.len() {
        if i < old_lines.len() && j < new_lines.len() && old_lines[i] == new_lines[j] {
            // Lines are the same
            diff.push(DiffLine::Unchanged(old_lines[i].clone(), i + 1));
            i += 1;
            j += 1;
        } else if j < new_lines.len() && (i >= old_lines.len() || !contains_line(&old_lines[i..], &new_lines[j])) {
            // Line added
            diff.push(DiffLine::Added(new_lines[j].clone(), j + 1));
            j += 1;
        } else if i < old_lines.len() && (j >= new_lines.len() || !contains_line(&new_lines[j..], &old_lines[i])) {
            // Line removed
            diff.push(DiffLine::Removed(old_lines[i].clone(), i + 1));
            i += 1;
        } else {
            // Context line (simplified - in real git this would be more complex)
            if i < old_lines.len() {
                diff.push(DiffLine::Unchanged(old_lines[i].clone(), i + 1));
                i += 1;
            }
            if j < new_lines.len() {
                diff.push(DiffLine::Unchanged(new_lines[j].clone(), j + 1));
                j += 1;
            }
        }
    }
    
    diff
}

fn contains_line(haystack: &[String], needle: &str) -> bool {
    haystack.iter().any(|line| line == needle)
}