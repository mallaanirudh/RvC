use crate::network::{
    identity::load_or_generate_identity,
    transport::build_transport,
    behaviour::{RvcBehaviour, RvcEvent},
};

use futures::StreamExt;
use libp2p::{
    swarm::{Swarm, SwarmEvent, dial_opts::DialOpts},
    Multiaddr,
};
use libp2p::mdns::Event as MdnsEvent;
use std::collections::HashSet;
use crate::network::protocol::{RvcRequest, RvcResponse};
use libp2p::request_response::{Event as RequestResponseEvent, Message as RequestResponseMessage};

pub async fn run_node(port: Option<u16>) -> Result<(), Box<dyn std::error::Error>> {
    let port = port.unwrap_or(4001);
    let identity = load_or_generate_identity(port);

    println!("Local peer id: {}", identity.peer_id);

    let transport = build_transport(&identity.keypair)?;
    let behaviour = RvcBehaviour::new(identity.peer_id).await?;

    let mut swarm = Swarm::new(
        transport,
        behaviour,
        identity.peer_id,
        libp2p::swarm::Config::with_tokio_executor(),
    );
    let addr: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse()?;
    swarm.listen_on(addr)?;
    let mut connected_peers = HashSet::new();

    loop {
        match swarm.select_next_some().await {

            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {address}");
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("Connected to {peer_id}");
                connected_peers.insert(peer_id);
            }

            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                println!("Disconnected from {peer_id}");
                connected_peers.remove(&peer_id);
            }

            SwarmEvent::Behaviour(event) => {
                match event {

                    RvcEvent::Mdns(mdns_event) => match mdns_event {

                        MdnsEvent::Discovered(list) => {
                            
                            for (peer, addr) in list {
                                if peer == identity.peer_id {
                                    continue;
                                }
                                if connected_peers.contains(&peer) {
                                    continue;
                                }
                                println!("Discovered {peer} at {addr}");
                                let opts = DialOpts::peer_id(peer)
                                    .addresses(vec![addr.clone()])
                                    .build();
                                if let Err(e) = swarm.dial(opts) {
        println!("Dial failed for {peer}: {e}");
        continue;
    }
                                swarm
                                .behaviour_mut()
                                .req_res
                                .send_request(&peer, RvcRequest::GetHead);
                               
                            }
                        }

                        MdnsEvent::Expired(_) => {}
                    },

                    RvcEvent::Ping(ping_event) => {
                        println!("Ping: {:?}", ping_event);
                    }
                    RvcEvent::ReqRes(event) => {
    match event {

        RequestResponseEvent::Message { peer, message } => {
            match message {
                RequestResponseMessage::Request { request, channel, .. } => {
                    match request {
                        RvcRequest::GetHead => {
                            println!("Peer {peer} requested HEAD");

                            let response = RvcResponse::Head("dummy_hash".into());

                            swarm.behaviour_mut()
                                .req_res
                                .send_response(channel, response)
                                .unwrap();
                        }
                    }
                }

            
                RequestResponseMessage::Response { response, .. } => {
                    println!("Received from {peer}: {:?}", response);
                }
            }
        }

        _ => {}
    }
}
                }
                
            }

            _ => {}
        }
        
    }
}