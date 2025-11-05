use crate::core::{write_tree, Commit, FsObjectStore, Object};
use crate::index::Index;
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

// Remove: pub struct CommitCommand { pub message: String }

pub fn execute(repo: &Path, message: &str) -> Result<()> {
    let store = FsObjectStore::new(repo);
    let idx = Index::load(repo)?;
    
    if idx.is_empty() {
        return Err(anyhow::anyhow!("nothing to commit"));
    }

    let tree_oid = write_tree(repo, &idx, &store)?;
    let parent = get_head_commit(repo)?;
    let parents = parent.into_iter().collect::<Vec<_>>();

    let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
    let commit = Commit {
        tree: tree_oid,
        parents,
        author: "minigit <minigit@example.com>".to_string(),
        message: message.to_string(),
        timestamp: ts,
    };

    let oid = store.put(&Object::Commit(commit))?;
    update_head(repo, &oid)?;

    println!("Committed {}", oid);
    Ok(())
}

fn get_head_commit(repo: &Path) -> Result<Option<String>> {
    let headp = crate::core::head_file(repo);
    if headp.exists() {
        let s = fs::read_to_string(&headp)?;
        if s.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(s.trim().to_string()))
        }
    } else {
        Ok(None)
    }
}

fn update_head(repo: &Path, oid: &crate::core::Oid) -> Result<()> {
    fs::create_dir_all(crate::core::refs_heads_dir(repo))?;
    fs::write(crate::core::head_file(repo), oid.to_hex())?;
    Ok(())
}