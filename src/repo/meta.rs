use std::collections::HashMap;
use libp2p::PeerId;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct RepoMeta {
    pub peers: Vec<String>,
}

pub fn load_meta(repo: &Path) -> RepoMeta {
    let path = crate::core::types::repo_dir(repo).join("meta.json");
    if let Ok(data) = std::fs::read(path) {
        if let Ok(meta) = serde_json::from_slice(&data) {
            return meta;
        }
    }
    RepoMeta::default()
}

pub fn save_meta(repo: &Path, meta: &RepoMeta) {
    let path = crate::core::types::repo_dir(repo).join("meta.json");
    if let Ok(data) = serde_json::to_vec(meta) {
        let _ = std::fs::write(path, data);
    }
}

pub fn add_peer(repo: &Path, peer: PeerId) {
    let mut meta = load_meta(repo);
    let peer_str = peer.to_base58();
    if !meta.peers.contains(&peer_str) {
        meta.peers.push(peer_str);
        save_meta(repo, &meta);
    }
}
