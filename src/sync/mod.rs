pub mod messages;
pub mod protocol;
pub mod manager;

use std::path::Path;
use crate::repo::sync::{get_local_refs, get_objects};

pub fn handle_request(repo: &Path, req: messages::SyncRequest) -> messages::SyncResponse {
    match req {
        messages::SyncRequest::GetRefs => {
            let refs = get_local_refs(repo);
            messages::SyncResponse::Refs(refs)
        }
        messages::SyncRequest::GetObjects(hashes) => {
            let objects = get_objects(repo, hashes);
            messages::SyncResponse::Objects(objects)
        }
    }
}
