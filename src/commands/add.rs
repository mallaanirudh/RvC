use crate::core::FsObjectStore;
use crate::index::Index;
use anyhow::Result;
use std::path::Path;

// Remove: pub struct AddCommand { pub path: String }

pub fn execute(repo: &Path, path: &str) -> Result<()> {
    let store = FsObjectStore::new(repo);
    let mut idx = Index::load(repo)?;
    let oid = idx.add_file(repo, Path::new(path), &store)?;
    idx.save(repo)?;
    println!("added {} -> {}", path, oid);
    Ok(())
}