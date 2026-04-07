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
                    RvcEvent::ReqRes(RequestResponseEvent::Message { peer, message }) => {
                        match message {
                            RequestResponseMessage::Request { request, channel, .. } => {
                                let repo = std::env::current_dir().unwrap();
                                let response = crate::sync::handle_request(&repo, request);
                                let _ = swarm.behaviour_mut().req_res.send_response(channel, response);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

pub async fn announce_cmd(cwd: &Path, repo: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (mut swarm, _) = create_swarm(None, 4001).await?;
    let key = repo_key(repo);
    let record = Record {
        key,
        value: swarm.local_peer_id().to_bytes(),
        publisher: None,
        expires: None,
    };
    let _ = swarm.behaviour_mut().kad.put_record(record, libp2p::kad::Quorum::One);

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::Behaviour(RvcEvent::Kad(KadEvent::OutboundQueryProgressed { result: QueryResult::PutRecord(res), .. })) => {
                match res {
                    Ok(_) => { println!("Announced successfully"); break; }
                    Err(e) => { println!("Failed to announce: {:?}", e); break; }
                }
            }
            _ => {}
        }
    }
    Ok(())
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


pub async fn sync_cmd(cwd: &Path, repo: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (mut swarm, _) = create_swarm(None, 0).await?;
    let meta = crate::repo::meta::load_meta(cwd);
    
    // We can try to dial stored peers first if there are any that we have the address for...
    // But we don't store addresses, just PeerId. We need Kademlia to find them!
    
    let key = repo_key(repo);
    swarm.behaviour_mut().kad.get_record(key);

    let mut found_peer = None;

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::Behaviour(RvcEvent::Kad(KadEvent::OutboundQueryProgressed { result: QueryResult::GetRecord(res), .. })) => {
                match res {
                    Ok(GetRecordOk::FoundRecord(PeerRecord { record: Record { value, .. }, .. })) => {
                        if let Ok(peer) = PeerId::from_bytes(&value) {
                            found_peer = Some(peer);
                            println!("Discovered peer via DHT: {}", peer);
                            crate::repo::meta::add_peer(cwd, peer);
                        }
                        break;
                    }
                    _ => {}
                }
            }
            SwarmEvent::Behaviour(RvcEvent::Mdns(MdnsEvent::Discovered(list))) => {
                for (peer, addr) in list {
                    swarm.behaviour_mut().kad.add_address(&peer, addr);
                    println!("mDNS Discovered peer: {}", peer);
                    found_peer = Some(peer);
                    break;
                }
                if found_peer.is_some() { break; }
            }
            _ => {}
        }
    }
    
    // If still none, wait up to 3 seconds for mDNS discovery
    if found_peer.is_none() {
        println!("DHT lookup empty. Waiting a moment for local mDNS discovery...");
        let timeout = tokio::time::sleep(tokio::time::Duration::from_secs(3));
        tokio::pin!(timeout);
        
        loop {
            tokio::select! {
                _ = &mut timeout => {
                    break;
                }
                event = swarm.select_next_some() => {
                    if let SwarmEvent::Behaviour(RvcEvent::Mdns(MdnsEvent::Discovered(list))) = event {
                        for (peer, addr) in list {
                            swarm.behaviour_mut().kad.add_address(&peer, addr);
                            println!("mDNS Discovered peer: {}", peer);
                            found_peer = Some(peer);
                            break;
                        }
                        if found_peer.is_some() { break; }
                    }
                }
            }
        }
    }

    if found_peer.is_none() {
        if let Some(peer_str) = meta.peers.first() {
            if let Ok(peer) = peer_str.parse() {
                println!("DHT lookup failed, falling back to stored peer: {}", peer);
                found_peer = Some(peer);
            }
        }
    }

    if let Some(peer) = found_peer {
        // Dial them
        let _ = swarm.dial(peer);
        
        // Wait for connection
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    if peer_id == peer {
                        println!("Connected, starting sync...");
                        crate::sync::manager::sync_with_peer(peer, cwd, &mut swarm).await?;
                        break;
                    }
                }
                SwarmEvent::OutgoingConnectionError { peer_id: Some(pid), error, .. } if pid == peer => {
                    println!("Failed to connect to {}: {:?}", pid, error);
                    break;
                }
                _ => {}
            }
        }
    } else {
        println!("No peers found for repo {}", repo);
    }

    Ok(())
}