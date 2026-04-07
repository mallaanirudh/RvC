use std::collections::HashMap;
use std::path::Path;
use libp2p::{PeerId, swarm::{Swarm, SwarmEvent}};
use libp2p::request_response::{Event as RequestResponseEvent, Message as RequestResponseMessage};
use futures::StreamExt;
use crate::network::behaviour::{RvcBehaviour, RvcEvent};
use super::messages::{SyncRequest, SyncResponse};
use crate::repo::sync::{get_local_refs, find_missing_objects, update_refs, store_objects, is_descendant, create_merge_commit};

pub fn sync_with_peer<'a>(
    peer: PeerId,
    cwd: &'a Path,
    swarm: &'a mut Swarm<RvcBehaviour>
) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<(), Box<dyn std::error::Error>>> + 'a>> {
    Box::pin(async move {
        let req_id = swarm.behaviour_mut().req_res.send_request(&peer, SyncRequest::GetRefs);
        
        let mut remote_refs = HashMap::new();
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::Behaviour(RvcEvent::ReqRes(RequestResponseEvent::Message { message: RequestResponseMessage::Response { request_id, response }, .. })) => {
                    if request_id == req_id {
                        if let SyncResponse::Refs(refs) = response {
                            remote_refs = refs;
                        }
                        break;
                    }
                }
                _ => {}
            }
        }

        let local_refs = get_local_refs(cwd);
        if remote_refs == local_refs {
            println!("Already up to date.");
            return Ok(());
        }

        let mut current_remote_heads = remote_refs.clone();
        
        loop {
            let missing = find_missing_objects(cwd, local_refs.clone(), current_remote_heads.clone());
            if missing.is_empty() {
                let mut final_refs = local_refs.clone();
                for (ref_name, remote_hash) in &remote_refs {
                    if let Some(local_hash) = local_refs.get(ref_name) {
                        if local_hash == remote_hash {
                            continue;
                        }
                        if is_descendant(cwd, remote_hash, local_hash) {
                            println!("Fast-forwarding {} to {}", ref_name, remote_hash);
                            final_refs.insert(ref_name.clone(), remote_hash.clone());
                        } else {
                            println!("Divergent branch detected on {}. Creating merge commit...", ref_name);
                            if let Some(merge_hash) = create_merge_commit(cwd, local_hash, remote_hash) {
                                println!("Created merge commit {} for {}", merge_hash, ref_name);
                                final_refs.insert(ref_name.clone(), merge_hash);
                            } else {
                                println!("Failed to create merge commit for {}", ref_name);
                            }
                        }
                    } else {
                        println!("Fetching new branch {} at {}", ref_name, remote_hash);
                        final_refs.insert(ref_name.clone(), remote_hash.clone());
                    }
                }
                update_refs(cwd, &final_refs);
                println!("Sync complete.");
                break;
            }

            let obj_req_id = swarm.behaviour_mut().req_res.send_request(&peer, SyncRequest::GetObjects(missing.clone()));
            let mut fetched_objects = Vec::new();
            
            loop {
                match swarm.select_next_some().await {
                    SwarmEvent::Behaviour(RvcEvent::ReqRes(RequestResponseEvent::Message { message: RequestResponseMessage::Response { request_id, response }, .. })) => {
                        if request_id == obj_req_id {
                            if let SyncResponse::Objects(objs) = response {
                                fetched_objects = objs;
                            }
                            break;
                        }
                    }
                    _ => {}
                }
            }

            store_objects(cwd, fetched_objects);
            let mut next_heads = HashMap::new();
            let store = crate::core::store::FsObjectStore::new(cwd);
            for hash in missing.iter() {
                if let Ok(oid) = crate::core::types::Oid::from_hex(&hash) {
                    if let Ok(Some(obj)) = store.get(&oid) {
                        match obj {
                            crate::core::types::Object::Commit(c) => {
                                for p in c.parents {
                                    next_heads.insert(p.clone(), p);
                                }
                                next_heads.insert(c.tree.clone(), c.tree.clone());
                            }
                            crate::core::types::Object::Tree(entries) => {
                                for e in entries {
                                    let hex = e.oid.to_hex();
                                    next_heads.insert(hex.clone(), hex);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            current_remote_heads = next_heads;
        }

        Ok(())
    })
}
