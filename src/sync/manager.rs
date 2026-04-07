use std::collections::HashMap;
use std::path::Path;
use libp2p::{PeerId, swarm::{Swarm, SwarmEvent}};
use libp2p::request_response::{Event as RequestResponseEvent, Message as RequestResponseMessage};
use futures::StreamExt;
use crate::network::behaviour::{RvcBehaviour, RvcEvent};
use super::messages::{SyncRequest, SyncResponse};
use crate::repo::sync::{get_local_refs, find_missing_objects, update_refs, store_objects, is_descendant, create_merge_commit};

/// Send a request and wait for its response, draining other swarm events meanwhile.
async fn send_and_wait(
    swarm: &mut Swarm<RvcBehaviour>,
    peer: &PeerId,
    request: SyncRequest,
) -> Result<SyncResponse, Box<dyn std::error::Error>> {
    let req_id = swarm.behaviour_mut().req_res.send_request(peer, request);
    let timeout = tokio::time::Duration::from_secs(15);

    match tokio::time::timeout(timeout, async {
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::Behaviour(RvcEvent::ReqRes(RequestResponseEvent::Message {
                    message: RequestResponseMessage::Response { request_id, response },
                    ..
                })) if request_id == req_id => {
                    return Ok(response);
                }
                SwarmEvent::Behaviour(RvcEvent::ReqRes(RequestResponseEvent::OutboundFailure {
                    request_id,
                    error,
                    ..
                })) if request_id == req_id => {
                    return Err(format!("Request failed: {:?}", error).into());
                }
                _ => {} // drain other events
            }
        }
    })
    .await
    {
        Ok(result) => result,
        Err(_) => Err("Request timed out after 15s".into()),
    }
}

pub fn sync_with_peer<'a>(
    peer: PeerId,
    cwd: &'a Path,
    swarm: &'a mut Swarm<RvcBehaviour>,
) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<(), Box<dyn std::error::Error>>> + 'a>> {
    Box::pin(async move {
        // --- Step 1: Get remote refs ---
        println!("Sending GetRefs request to {:?}", peer);
        let remote_refs = match send_and_wait(swarm, &peer, SyncRequest::GetRefs).await? {
            SyncResponse::Refs(refs) => refs,
            _ => return Err("Unexpected response to GetRefs".into()),
        };
        println!("Remote refs ({}):", remote_refs.len());
        for (k, v) in &remote_refs {
            println!("  {} -> {}", k, v);
        }

        let local_refs = get_local_refs(cwd);
        println!("Local refs ({}):", local_refs.len());
        for (k, v) in &local_refs {
            println!("  {} -> {}", k, v);
        }

        if remote_refs.is_empty() {
            println!("Remote has no commits. Nothing to sync.");
            return Ok(());
        }

        // --- Step 2: Iteratively fetch ALL missing objects ---
        // Round 1: discovers missing commit hashes
        // Round 2: after storing commits, discovers missing tree hashes  
        // Round 3: after storing trees, discovers missing blob hashes
        // Round N: no more missing → done
        let mut round = 0;
        loop {
            round += 1;
            let missing = find_missing_objects(cwd, local_refs.clone(), remote_refs.clone());
            println!("Round {}: {} missing objects", round, missing.len());
            
            if missing.is_empty() {
                println!("All objects fetched.");
                break;
            }

            if round > 20 {
                return Err("Too many fetch rounds — possible cycle in object graph".into());
            }

            for h in &missing {
                println!("  Need: {}", h);
            }

            let fetched = match send_and_wait(swarm, &peer, SyncRequest::GetObjects(missing.clone())).await? {
                SyncResponse::Objects(objs) => objs,
                _ => return Err("Unexpected response to GetObjects".into()),
            };
            println!("Received {} objects from remote.", fetched.len());

            if fetched.is_empty() {
                println!("Remote returned 0 objects — cannot make progress.");
                break;
            }

            store_objects(cwd, fetched);
        }

        // --- Step 3: Update refs (fast-forward or merge) ---
        let mut final_refs = local_refs.clone();
        for (ref_name, remote_hash) in &remote_refs {
            match local_refs.get(ref_name) {
                None => {
                    println!("New ref {}: {}", ref_name, remote_hash);
                    final_refs.insert(ref_name.clone(), remote_hash.clone());
                }
                Some(local_hash) if local_hash == remote_hash => {
                    println!("Ref {} already up to date.", ref_name);
                }
                Some(local_hash) => {
                    if is_descendant(cwd, remote_hash, local_hash) {
                        println!("Fast-forwarding {} to {}", ref_name, remote_hash);
                        final_refs.insert(ref_name.clone(), remote_hash.clone());
                    } else {
                        println!("Diverged on {}. Creating merge commit...", ref_name);
                        match create_merge_commit(cwd, local_hash, remote_hash) {
                            Some(merge_hash) => {
                                println!("Merge commit: {}", merge_hash);
                                final_refs.insert(ref_name.clone(), merge_hash);
                            }
                            None => println!("Could not create merge for {}", ref_name),
                        }
                    }
                }
            }
        }

        update_refs(cwd, &final_refs);
        println!("Refs updated.");

        // --- Step 4: Checkout the new HEAD ---
        if let Some(new_head) = final_refs.get("HEAD") {
            println!("Checking out HEAD: {}...", new_head);
            match crate::commands::checkout::execute(cwd, new_head) {
                Ok(_) => println!("Checkout successful."),
                Err(e) => println!("Checkout error: {:?}", e),
            }
        }

        println!("Sync complete.");
        Ok(())
    })
}
