use super::types::{Commit, Oid, TreeEntry};
use blake3::Hasher;
use serde_json;

pub fn oid_for_bytes(kind: &str, data: &[u8]) -> Oid {
    let mut hasher = Hasher::new();
    hasher.update(kind.as_bytes());
    hasher.update(&[0u8]);
    hasher.update(data);
    let out = hasher.finalize();
    let mut b = [0u8; 32];
    b.copy_from_slice(out.as_bytes());
    Oid::from_bytes(b)
}

pub fn blob_oid(content: &[u8]) -> Oid {
    oid_for_bytes("blob", content)
}

pub fn tree_serialize(entries: &[TreeEntry]) -> Vec<u8> {
    let mut es = entries.to_vec();
    es.sort_by(|a, b| a.name.cmp(&b.name));
    
    let mut out = Vec::new();
    for e in es.iter() {
        out.extend(format!("{} {} {}\n", e.mode, e.name, e.oid.to_hex()).as_bytes());
    }
    out
}

pub fn tree_oid(entries: &[TreeEntry]) -> Oid {
    let body = tree_serialize(entries);
    oid_for_bytes("tree", &body)
}

pub fn commit_serialize(c: &Commit) -> Vec<u8> {
    serde_json::to_vec(c).expect("commit serialize")
}

pub fn commit_oid(c: &Commit) -> Oid {
    let body = commit_serialize(c);
    oid_for_bytes("commit", &body)
}