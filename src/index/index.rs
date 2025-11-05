use crate::core::{FsObjectStore, Object};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path};
use std::time::{UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexEntry {
    pub path: String,
    pub oid: String,
    pub mode: u32,
    pub mtime: u64,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Index {
    pub entries: HashMap<String, IndexEntry>,
}

impl Index {
    pub fn load<P: AsRef<Path>>(repo: P) -> Result<Self> {
        let p = crate::core::index_file(repo);
        if p.exists() {
            let s = fs::read_to_string(p)?;
            let idx: Index = serde_json::from_str(&s)?;
            Ok(idx)
        } else {
            Ok(Index::default())
        }
    }

    pub fn save<P: AsRef<Path>>(&self, repo: P) -> Result<()> {
        let p = crate::core::index_file(repo);
        fs::create_dir_all(p.parent().unwrap())?;
        let tmp = p.with_extension("tmp");
        let mut f = File::create(&tmp)?;
        let s = serde_json::to_vec(self)?;
        f.write_all(&s)?;
        f.sync_all()?;
        fs::rename(tmp, p)?;
        Ok(())
    }

    pub fn add_file<P: AsRef<Path>>(
        &mut self,
        repo: P,
        relpath: &Path,
        store: &FsObjectStore,
    ) -> Result<String> {
        let abs = repo.as_ref().join(relpath);
        let mut f = File::open(&abs)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        let oid = store.put(&Object::Blob(buf))?;
        let meta = fs::metadata(&abs)?;

        let entry = IndexEntry {
            path: relpath.to_string_lossy().to_string(),
            oid: oid.to_hex(),
            mode: 0o100644,
            mtime: meta.modified()?.duration_since(UNIX_EPOCH)?.as_secs(),
            size: meta.len(),
        };

        self.entries.insert(entry.path.clone(), entry.clone());
        Ok(entry.oid)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}