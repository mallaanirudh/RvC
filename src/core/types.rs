use serde::{Deserialize, Serialize};
use std::fmt;

/// 32-byte OID (blake3)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Oid([u8; 32]);

impl Oid {
    pub fn from_bytes(b: [u8; 32]) -> Self {
        Oid(b)
    }
    
    pub fn zero() -> Self {
        Oid([0u8; 32])
    }
    
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
    
    pub fn from_hex(hex_str: &str) -> anyhow::Result<Self> {
        let bytes = hex::decode(hex_str)?;
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(Oid(array))
    }
}

impl fmt::Display for Oid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<&[u8]> for Oid {
    fn from(slice: &[u8]) -> Self {
        let mut b = [0u8; 32];
        b.copy_from_slice(&slice[0..32]);
        Oid(b)
    }
}

#[derive(Debug, Clone)]
pub enum Object {
    Blob(Vec<u8>),
    Tree(Vec<TreeEntry>),
    Commit(Commit),
}

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub mode: u32,
    pub name: String,
    pub oid: Oid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub tree: String,
    pub parents: Vec<String>,
    pub author: String,
    pub message: String,
    pub timestamp: i64,
}

// Repository layout helpers
pub fn repo_dir<P: AsRef<std::path::Path>>(p: P) -> std::path::PathBuf {
    p.as_ref().join(".minigit")
}

pub fn objects_dir<P: AsRef<std::path::Path>>(p: P) -> std::path::PathBuf {
    repo_dir(p).join("objects")
}

pub fn refs_heads_dir<P: AsRef<std::path::Path>>(p: P) -> std::path::PathBuf {
    repo_dir(p).join("refs").join("heads")
}

pub fn head_file<P: AsRef<std::path::Path>>(p: P) -> std::path::PathBuf {
    repo_dir(p).join("HEAD")
}

pub fn index_file<P: AsRef<std::path::Path>>(p: P) -> std::path::PathBuf {
    repo_dir(p).join("index")
}