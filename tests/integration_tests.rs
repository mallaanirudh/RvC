use minigit::core::{FsObjectStore, Object};
use minigit::index::Index;
use std::fs;
use tempfile;

#[test]
fn test_blob_oid_stability() {
    let data = b"hello world";
    let o1 = minigit::core::blob_oid(data);
    let o2 = minigit::core::blob_oid(data);
    assert_eq!(o1, o2);
}

#[test]
fn test_put_get_blob() -> anyhow::Result<()> {
    let td = tempfile::tempdir()?;
    let repo = td.path();
    minigit::core::init(repo)?;
    
    let store = FsObjectStore::new(repo);
    let oid = store.put(&Object::Blob(b"hi".to_vec()))?;
    
    let got = store.get(&oid)?.expect("exists");
    match got {
        Object::Blob(b) => assert_eq!(b, b"hi"),
        _ => panic!("expected blob"),
    }
    Ok(())
}

#[test]
fn test_commit_and_log() -> anyhow::Result<()> {
    let td = tempfile::tempdir()?;
    let repo = td.path();
    minigit::core::init(repo)?;
    
    let store = FsObjectStore::new(repo);
    fs::write(repo.join("f.txt"), b"a")?;
    
    let mut idx = Index::load(repo)?;
    idx.add_file(repo, std::path::Path::new("f.txt"), &store)?;
    idx.save(repo)?;
    
    let tree_oid = minigit::core::write_tree(repo, &idx, &store)?;
    assert!(!tree_oid.is_empty());
    
    Ok(())
}