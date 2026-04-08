use crate::network::{
    identity::load_or_generate_identity,
    transport::build_transport,
    behaviour::{RvcBehaviour, RvcEvent, repo_key},
};
use crate::sync::messages::{SyncRequest, SyncResponse};
use libp2p::{
    swarm::{Swarm, SwarmEvent, dial_opts::DialOpts},
    Multiaddr, PeerId,
};
use std::collections::{HashSet, HashMap};
use std::path::{Path, PathBuf};
use futures::StreamExt;
use libp2p::mdns::Event as MdnsEvent;
use libp2p::request_response::{Event as RequestResponseEvent, Message as RequestResponseMessage};
use libp2p::kad::{Record, store::MemoryStore, Event as KadEvent, QueryResult, GetRecordOk, PeerRecord};

pub async fn create_swarm(port: Option<u16>, identity_port: u16) -> Result<(Swarm<RvcBehaviour>, PeerId), Box<dyn std::error::Error>> {
    let identity = load_or_generate_identity(identity_port);
    let transport = build_transport(&identity.keypair)?;
    let behaviour = RvcBehaviour::new(identity.peer_id).await?;
    let mut swarm = Swarm::new(
        transport,
        behaviour,
        identity.peer_id,
        libp2p::swarm::Config::with_tokio_executor(),
    );
    let listen_port = port.unwrap_or(0);
    let addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", listen_port).parse()?;
    swarm.listen_on(addr)?;
    Ok((swarm, identity.peer_id))
}

pub async fn run_node(port: Option<u16>, bootstrap: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let port = port.unwrap_or(4001);
    let (mut swarm, peer_id) = create_swarm(Some(port), port).await?;
    println!("Local peer id: {}", peer_id);

    if let Some(addr_str) = bootstrap {
        let addr: Multiaddr = addr_str.parse()?;
        swarm.dial(addr.clone())?;
    }

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {address}");
            }
            SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                swarm.behaviour_mut().kad.add_address(&peer_id, endpoint.get_remote_address().clone());
                let _ = swarm.behaviour_mut().kad.bootstrap();
            }
            SwarmEvent::Behaviour(event) => {
                match event {
                    RvcEvent::Mdns(MdnsEvent::Discovered(list)) => {
                        for (peer, addr) in list {
                            swarm.behaviour_mut().kad.add_address(&peer, addr.clone());
                        }
                    }
                    RvcEvent::ReqRes(RequestResponseEvent::Message { peer: _peer, message }) => {
                        match message {
                            RequestResponseMessage::Request { request, channel, .. } => {
                                if let Ok(repo) = std::env::current_dir() {
                                    println!("--- INCOMING REQUEST ---");
                                    println!("Repo Path: {:?}", repo.canonicalize().unwrap_or(repo.clone()));
                                    let response = crate::sync::handle_request(&repo, request);
                                    match swarm.behaviour_mut().req_res.send_response(channel, response) {
                                        Ok(_) => println!("Response successfully queued."),
                                        Err(_) => println!("Error: Response channel closed."),
                                    }
                                }
                            }
                            RequestResponseMessage::Response { .. } => {}
                        }
                    }
                    RvcEvent::ReqRes(RequestResponseEvent::InboundFailure { error, .. }) => {
                        println!("Inbound request failed: {:?}", error);
                    }
                    RvcEvent::ReqRes(RequestResponseEvent::OutboundFailure { error, .. }) => {
                        println!("Outbound request failed: {:?}", error);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

pub async fn announce_cmd(_cwd: &Path, repo: &str, port: Option<u16>) -> Result<(), Box<dyn std::error::Error>> {
    let identity_port = port.unwrap_or(4001);
    // Get the actual identity we want to announce for
    let target_identity = load_or_generate_identity(identity_port);
    let target_peer_id = target_identity.peer_id;

    // Create a temporary swarm with a DIFFERENT identity (port 0) 
    // so it can connect to the local node if it's running with the same port-based identity.
    let (mut swarm, _) = create_swarm(None, 0).await?;
    
    println!("--- ANNOUNCE START ---");
    println!("Target Repository: {}", repo);
    println!("Announcing Peer ID: {}", target_peer_id);
    println!("Searching for peers to join the network...");
    
    let key = repo_key(repo);
    let timeout = tokio::time::sleep(tokio::time::Duration::from_secs(30));
    tokio::pin!(timeout);
    
    let mut announced = false;

    loop {
        tokio::select! {
            _ = &mut timeout => {
                if !announced {
                   return Err(anyhow::anyhow!("Announcement timed out: No peers found within 30s. Make sure another node is running!").into());
                }
                break;
            }
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Discovery node listening on {}", address);
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("Connected to peer: {}. Publishing record...", peer_id);
                        let record = Record {
                            key: key.clone(),
                            value: target_peer_id.to_bytes(),
                            publisher: None,
                            expires: None,
                        };
                        let _ = swarm.behaviour_mut().kad.put_record(record, libp2p::kad::Quorum::One);
                    }
                    SwarmEvent::Behaviour(RvcEvent::Kad(KadEvent::OutboundQueryProgressed { result: QueryResult::PutRecord(res), .. })) => {
                        match res {
                            Ok(_) => { 
                                println!("Announced successfully! Other nodes can now find {} via {}", repo, target_peer_id);
                                announced = true;
                                break; 
                            }
                            Err(e) => {
                                println!("PutRecord attempted but failed: {:?}. Continuing to seek peers...", e);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    if announced {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to announce after exhausting discovery options.").into())
    }
}

pub async fn peers_cmd(cwd: &Path, repo: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (mut swarm, _) = create_swarm(None, 0).await?;
    let meta = crate::repo::meta::load_meta(cwd);
    println!("Stored peers: {:?}", meta.peers);

    let key = repo_key(repo);
    swarm.behaviour_mut().kad.get_record(key);

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::Behaviour(RvcEvent::Kad(KadEvent::OutboundQueryProgressed { result: QueryResult::GetRecord(res), .. })) => {
                match res {
                    Ok(GetRecordOk::FoundRecord(PeerRecord { record: Record { value, .. }, .. })) => {
                        if let Ok(peer) = PeerId::from_bytes(&value) {
                            println!("Found peer for repo: {}", peer);
                        }
                        break;
                    }
                    Err(e) => {
                        println!("Failed to find peers: {:?}", e);
                        break;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    Ok(())
}


pub async fn sync_cmd(cwd: &Path, repo: &str, port: Option<u16>) -> Result<(), Box<dyn std::error::Error>> {
    let identity_port = port.unwrap_or(0);
    let (mut swarm, _) = create_swarm(None, identity_port).await?;
    let meta = crate::repo::meta::load_meta(cwd);
    
    // We can try to dial stored peers first if there are any that we have the address for...
    // But we don't store addresses, just PeerId. We need Kademlia to find them!
    
    let key = repo_key(repo);
    swarm.behaviour_mut().kad.get_record(key);

    let mut target_peer = None;

    // Dial already known peers from meta
    for peer_str in &meta.peers {
        if let Ok(peer) = peer_str.parse() {
            println!("Dialing known peer from meta: {}", peer);
            let _ = swarm.dial(peer);
            target_peer = Some(peer);
        }
    }

    println!("Searching for peers and waiting for connection...");
    let timeout = tokio::time::sleep(tokio::time::Duration::from_secs(30));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            _ = &mut timeout => {
                return Err(anyhow::anyhow!("Sync timed out: No peers found for repo {}", repo).into());
            }
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::Behaviour(RvcEvent::Mdns(MdnsEvent::Discovered(list))) => {
                        for (peer, addr) in list {
                            println!("mDNS Discovered peer: {}", peer);
                            swarm.behaviour_mut().kad.add_address(&peer, addr);
                            target_peer = Some(peer);
                            
                            if swarm.is_connected(&peer) {
                                println!("Peer {} already connected. Starting sync...", peer);
                                if let Err(e) = crate::sync::manager::sync_with_peer(peer, cwd, &mut swarm).await {
                                    println!("Sync with peer {} failed: {}. Continuing discovery...", peer, e);
                                } else {
                                    println!("Sync with peer {} successful!", peer);
                                    return Ok(());
                                }
                            } else {
                                println!("Dialing discovered peer: {}", peer);
                                let _ = swarm.dial(peer);
                            }
                        }
                    }
                    SwarmEvent::Behaviour(RvcEvent::Kad(KadEvent::OutboundQueryProgressed { result: QueryResult::GetRecord(res), .. })) => {
                        if let Ok(GetRecordOk::FoundRecord(PeerRecord { record: Record { value, .. }, .. })) = res {
                            if let Ok(peer_id) = PeerId::from_bytes(&value) {
                                println!("DHT Discovered peer: {}", peer_id);
                                target_peer = Some(peer_id);
                                
                                if swarm.is_connected(&peer_id) {
                                    println!("Peer {} already connected (DHT). Starting sync...", peer_id);
                                    if let Err(e) = crate::sync::manager::sync_with_peer(peer_id, cwd, &mut swarm).await {
                                        println!("Sync with peer {} failed: {}. Continuing discovery...", peer_id, e);
                                    } else {
                                        println!("Sync with peer {} successful!", peer_id);
                                        return Ok(());
                                    }
                                } else {
                                    println!("Dialing peer from DHT: {}", peer_id);
                                    let _ = swarm.dial(peer_id);
                                }
                            }
                        }
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        if Some(peer_id) == target_peer {
                            println!("Connection established to {}. Starting sync...", peer_id);
                            if let Err(e) = crate::sync::manager::sync_with_peer(peer_id, cwd, &mut swarm).await {
                                println!("Sync with peer {} failed: {}. Continuing discovery...", peer_id, e);
                            } else {
                                println!("Sync with peer {} successful!", peer_id);
                                return Ok(());
                            }
                        }
                    }
                    SwarmEvent::OutgoingConnectionError { peer_id: Some(peer_id), error, .. } => {
                        if Some(peer_id) == target_peer {
                            println!("Failed to connect to {}: {:?}", peer_id, error);
                            // Keep trying discovery
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}