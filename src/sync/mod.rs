pub mod messages;
pub mod protocol;
pub mod manager;

use std::path::Path;
use crate::repo::sync::{get_local_refs, get_objects};

pub fn handle_request(repo: &Path, req: messages::SyncRequest) -> messages::SyncResponse {
    println!("Sync request received: {:?}", req);
    match req {
        messages::SyncRequest::GetRefs => {
            let refs = get_local_refs(repo);
            println!("Reporting {} local refs.", refs.len());
            messages::SyncResponse::Refs(refs)
        }
        messages::SyncRequest::GetObjects(hashes) => {
            println!("GetObjects request for {} hashes: {:?}", hashes.len(), hashes);
            let objects = get_objects(repo, hashes);
            println!("Returning {} found objects.", objects.len());
            messages::SyncResponse::Objects(objects)
        }
    }
}
