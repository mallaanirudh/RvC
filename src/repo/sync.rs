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

        if let Ok(Some(obj)) = store.get(&oid) {
            // Traverse parents
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
        } else {
            // Object is missing! Collect it.
            missing.push(hash_str);
        }
    }
    missing
}

pub fn get_objects(repo: &Path, hashes: Vec<String>) -> Vec<(String, Vec<u8>)> {
    let mut res = Vec::new();
    for hash_str in hashes {
        let path = crate::core::types::objects_dir(repo).join(&hash_str);
        if let Ok(data) = std::fs::read(path) {
            res.push((hash_str, data));
        }
    }
    res
}

pub fn store_objects(repo: &Path, objects: Vec<(String, Vec<u8>)>) {
    for (hash_str, data) in objects {
        let path = crate::core::types::objects_dir(repo).join(&hash_str);
        if !path.exists() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(path, data);
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
