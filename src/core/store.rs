use super::hashing::{blob_oid, commit_oid, commit_serialize, tree_oid, tree_serialize};
use crate::core::types::{Commit, Object, Oid, TreeEntry, objects_dir, head_file, refs_heads_dir};
use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};


pub struct FsObjectStore {
    repo: PathBuf,
}

impl FsObjectStore {
    pub fn new<P: AsRef<Path>>(repo_root: P) -> Self {
        FsObjectStore {
            repo: repo_root.as_ref().to_path_buf(),
        }
    }

    fn object_path(&self, oid: &Oid) -> PathBuf {
        super::objects_dir(&self.repo).join(oid.to_hex())
    }

    pub fn put(&self, obj: &Object) -> Result<Oid> {
        match obj {
            Object::Blob(b) => {
                let oid = blob_oid(b);
                self.write_object("blob", &oid, b)?;
                Ok(oid)
            }
            Object::Tree(entries) => {
                let oid = tree_oid(entries);
                let data = tree_serialize(entries);
                self.write_object("tree", &oid, &data)?;
                Ok(oid)
            }
            Object::Commit(c) => {
                let oid = commit_oid(c);
                let data = commit_serialize(c);
                self.write_object("commit", &oid, &data)?;
                Ok(oid)
            }
        }
    }

    fn write_object(&self, kind: &str, oid: &Oid, data: &[u8]) -> Result<()> {
        let path = self.object_path(oid);
        if !path.exists() {
            fs::create_dir_all(path.parent().unwrap())?;
            let mut f = File::create(&path)?;
            f.write_all(kind.as_bytes())?;
            f.write_all(&[0u8])?;
            f.write_all(data)?;
            f.sync_all()?;
        }
        Ok(())
    }

    pub fn get(&self, oid: &Oid) -> Result<Option<Object>> {
        let path = self.object_path(oid);
        if !path.exists() {
            return Ok(None);
        }

        let mut buf = Vec::new();
        let mut f = File::open(path)?;
        f.read_to_end(&mut buf)?;

        if let Some(pos) = buf.iter().position(|&b| b == 0) {
            let kind = &buf[..pos];
            let body = &buf[pos + 1..];
            match std::str::from_utf8(kind).unwrap_or("") {
                "blob" => Ok(Some(Object::Blob(body.to_vec()))),
                "tree" => self.parse_tree_object(body),
                "commit" => self.parse_commit_object(body),
                other => Err(anyhow::anyhow!("unknown object kind: {}", other)),
            }
        } else {
            Err(anyhow::anyhow!("malformed object file"))
        }
    }

    fn parse_tree_object(&self, body: &[u8]) -> Result<Option<Object>> {
        let s = std::str::from_utf8(body).context("tree body utf8")?;
        let mut entries = Vec::new();

        for line in s.lines() {
            let parts: Vec<&str> = line.split(' ').collect();
            if parts.len() >= 3 {
                let mode: u32 = parts[0].parse().unwrap_or(0);
                let name = parts[1].to_string();
                let oid_hex = parts[2];
                let oid = Oid::from_hex(oid_hex)?;
                entries.push(TreeEntry { mode, name, oid });
            }
        }

        Ok(Some(Object::Tree(entries)))
    }

    fn parse_commit_object(&self, body: &[u8]) -> Result<Option<Object>> {
        let c: Commit = serde_json::from_slice(body)?;
        Ok(Some(Object::Commit(c)))
    }
}
pub fn write_tree<P: AsRef<Path>>(
   _repo: P,
    index: &crate::index::Index,
    store: &FsObjectStore,
) -> anyhow::Result<String> {
    let mut entries = Vec::new();
    
    for ent in index.entries.values() {
        let oid = Oid::from_hex(&ent.oid)?;
        entries.push(crate::core::TreeEntry {
            mode: ent.mode,
            name: ent.path.clone(),
            oid,
        });
    }
    
    let oid = store.put(&Object::Tree(entries))?;
    Ok(oid.to_hex())
}

pub fn init<P: AsRef<Path>>(repo: P) -> anyhow::Result<()> {
    fs::create_dir_all(objects_dir(&repo))?;
    fs::create_dir_all(refs_heads_dir(&repo))?;
    fs::write(head_file(&repo), "")?;
    Ok(())
}