use anyhow::Result;
use crate::core::store::FsObjectStore;
use crate::core::types::{Object, Oid};
use std::path::Path;
use std::fs;

pub fn execute(repo: &Path, commit_hash: &str) -> Result<()> {
    let store = FsObjectStore::new(repo);
    let commit_oid = Oid::from_hex(commit_hash)?;
    
    if let Some(Object::Commit(commit)) = store.get(&commit_oid)? {
        let tree_oid = Oid::from_hex(&commit.tree)?;
        checkout_tree(repo, &store, &tree_oid)?;
    } else {
        return Err(anyhow::anyhow!("Commit not found: {}", commit_hash));
    }
    
    Ok(())
}

fn checkout_tree(repo: &Path, store: &FsObjectStore, tree_oid: &Oid) -> Result<()> {
    if let Some(Object::Tree(entries)) = store.get(tree_oid)? {
        for entry in entries {
            let path = repo.join(&entry.name);
            
            // For now, we only support blobs directly in the root or sub-folders
            // (Our tree model is simple: a list of all files in the commit)
            if let Some(Object::Blob(content)) = store.get(&entry.oid)? {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&path, content)?;
                println!("Updated: {}", entry.name);
            }
        }
    }
    Ok(())
}
