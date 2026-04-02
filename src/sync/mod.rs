use crate::network::protocol::{SyncRequest, SyncResponse};
use crate::repo::sync::{get_local_refs, get_objects, update_refs, store_objects, find_missing_objects};
use libp2p::PeerId;
use std::path::{Path, PathBuf};
use crate::network::behaviour::RvcEvent;

pub fn handle_request(repo: &Path, req: SyncRequest) -> SyncResponse {
    match req {
        SyncRequest::GetRefs => {
            let refs = get_local_refs(repo);
            SyncResponse::Refs(refs)
        }
        SyncRequest::GetObjects(hashes) => {
            let objects = get_objects(repo, hashes);
            SyncResponse::Objects(objects)
        }
    }
}
