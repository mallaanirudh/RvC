use std::collections::{HashMap, HashSet};
use crate::core::store::FsObjectStore;
use crate::core::types::{Object, Oid};
use std::path::{Path, PathBuf};

pub fn find_missing_objects(
    repo: &Path,
    _local_heads: HashMap<String, String>,
    remote_heads: HashMap<String, String>,
) -> Vec<String> {
    let store = FsObjectStore::new(repo);
    let mut missing = Vec::new();
    let mut queue: Vec<String> = remote_heads.values().cloned().collect();
    let mut visited = HashSet::new();

    while let Some(hash_str) = queue.pop() {
        if visited.contains(&hash_str) { continue; }
        visited.insert(hash_str.clone());

        let oid = match Oid::from_hex(&hash_str) {
            Ok(o) => o,
            Err(_) => continue,
        };

        match store.get(&oid) {
            Ok(Some(obj)) => {
                // Object exists locally, traverse its children/parents to find further potential missing objects
                match obj {
                    Object::Commit(c) => {
                        for p in c.parents {
                            queue.push(p);
                        }
                        queue.push(c.tree);
                    }
                    Object::Tree(entries) => {
                        for e in entries {
                            queue.push(e.oid.to_hex());
                        }
                    }
                    Object::Blob(_) => {}
                }
            }
            _ => {
                // Object is missing locally! Add it to the list to be fetched.
                missing.push(hash_str.clone());
            }
        }
    }
    missing
}

pub fn get_objects(repo: &Path, hashes: Vec<String>) -> Vec<(String, Vec<u8>)> {
    println!("Serving {} requested objects", hashes.len());
    let mut res = Vec::new();
    for hash_str in hashes {
        let path = crate::core::types::objects_dir(repo).join(&hash_str);
        match std::fs::read(&path) {
            Ok(data) => {
                res.push((hash_str, data));
            }
            Err(e) => {
                println!("Warning: Failed to read object {}: {:?}", hash_str, e);
            }
        }
    }
    res
}

pub fn store_objects(repo: &Path, objects: Vec<(String, Vec<u8>)>) {
    println!("Storing {} objects...", objects.len());
    for (hash_str, data) in objects {
        let path = crate::core::types::objects_dir(repo).join(&hash_str);
        if !path.exists() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(e) = std::fs::write(&path, data) {
                println!("Error storing object {}: {:?}", hash_str, e);
            } else {
                println!("Stored object {}", hash_str);
            }
        }
    }
}

pub fn get_local_refs(repo: &Path) -> HashMap<String, String> {
    let mut refs = HashMap::new();
    if let Ok(head) = std::fs::read_to_string(crate::core::types::head_file(repo)) {
        if !head.trim().is_empty() {
             refs.insert("HEAD".to_string(), head.trim().to_string());
        }
    }
    let heads_dir = crate::core::types::refs_heads_dir(repo);
    if let Ok(entries) = std::fs::read_dir(heads_dir) {
        for entry in entries.filter_map(Result::ok) {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let name = entry.file_name().to_string_lossy().to_string();
                refs.insert(name, content.trim().to_string());
            }
        }
    }
    refs
}

pub fn update_refs(repo: &Path, refs: &HashMap<String, String>) {
    let heads_dir = crate::core::types::refs_heads_dir(repo);
    let _ = std::fs::create_dir_all(&heads_dir);
    for (name, hash) in refs {
        if name == "HEAD" {
            let _ = std::fs::write(crate::core::types::head_file(repo), hash);
        } else {
            let _ = std::fs::write(heads_dir.join(name), hash);
        }
    }
}

pub fn is_descendant(repo: &Path, descendant_hash: &str, ancestor_hash: &str) -> bool {
    if descendant_hash == ancestor_hash {
        return true;
    }
    let store = FsObjectStore::new(repo);
    let mut queue = vec![descendant_hash.to_string()];
    let mut visited = HashSet::new();

    while let Some(hash_str) = queue.pop() {
        if hash_str == ancestor_hash {
            return true;
        }
        if visited.contains(&hash_str) {
            continue;
        }
        visited.insert(hash_str.clone());

        if let Ok(oid) = Oid::from_hex(&hash_str) {
            if let Ok(Some(Object::Commit(c))) = store.get(&oid) {
                for p in c.parents {
                    queue.push(p);
                }
            }
        }
    }
    false
}

pub fn create_merge_commit(
    repo: &Path,
    local_hash: &str,
    remote_hash: &str,
) -> Option<String> {
    let store = FsObjectStore::new(repo);
    let local_oid = Oid::from_hex(local_hash).ok()?;
    let remote_oid = Oid::from_hex(remote_hash).ok()?;

    let local_commit = match store.get(&local_oid).ok()? {
        Some(Object::Commit(c)) => c,
        _ => return None,
    };
    let remote_commit = match store.get(&remote_oid).ok()? {
        Some(Object::Commit(c)) => c,
        _ => return None,
    };

    let mut merged_entries = std::collections::HashMap::new();
    
    if let Ok(remote_tree_oid) = Oid::from_hex(&remote_commit.tree) {
        if let Ok(Some(Object::Tree(entries))) = store.get(&remote_tree_oid) {
            for e in entries {
                merged_entries.insert(e.name.clone(), e);
            }
        }
    }
    
    if let Ok(local_tree_oid) = Oid::from_hex(&local_commit.tree) {
        if let Ok(Some(Object::Tree(entries))) = store.get(&local_tree_oid) {
            for e in entries {
                merged_entries.insert(e.name.clone(), e);
            }
        }
    }

    let mut new_entries: Vec<crate::core::types::TreeEntry> = merged_entries.into_values().collect();
    new_entries.sort_by(|a, b| a.name.cmp(&b.name));

    let tree_oid = store.put(&Object::Tree(new_entries)).ok()?;

    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    
    let merge_commit = crate::core::types::Commit {
        tree: tree_oid.to_hex(),
        parents: vec![local_hash.to_string(), remote_hash.to_string()],
        author: "minigit-sync <sync@example.com>".to_string(),
        message: format!("Merge remote {} into {}", remote_hash, local_hash),
        timestamp: ts,
    };

    let commit_oid = store.put(&Object::Commit(merge_commit)).ok()?;
    Some(commit_oid.to_hex())
}
