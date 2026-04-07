use libp2p::{
    kad::{self, Record, store::MemoryStore},
    mdns,
    ping,
    request_response,
    swarm::NetworkBehaviour,
};

use libp2p::request_response::{
    Behaviour as RequestResponseBehaviour,
    Event as RequestResponseEvent,
    ProtocolSupport,
};

use crate::sync::protocol::{RvcProtocol, RvcCodec};
use crate::sync::messages::{SyncRequest, SyncResponse};
use std::iter;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "RvcEvent")]
#[behaviour(prelude = "libp2p::swarm::derive_prelude")]
pub struct RvcBehaviour {
    pub mdns: mdns::tokio::Behaviour,
    pub ping: ping::Behaviour,
    pub req_res: RequestResponseBehaviour<RvcCodec>,
    pub kad: kad::Behaviour<MemoryStore>,
}

impl RvcBehaviour {
    pub async fn new(peer_id: libp2p::PeerId) -> Result<Self, Box<dyn std::error::Error>> {
        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(),
            peer_id,
        )?;

        let ping = ping::Behaviour::default();
        let req_res = RequestResponseBehaviour::with_codec(
       RvcCodec::default(), 
       iter::once((RvcProtocol, ProtocolSupport::Full)), 
      request_response::Config::default(),
    );
        
        let store = MemoryStore::new(peer_id);
        let kad = kad::Behaviour::new(peer_id, store);

        Ok(Self { mdns, ping, req_res, kad })
    }
}

pub fn repo_key(name: &str) -> libp2p::kad::RecordKey {
    libp2p::kad::RecordKey::new(&name.as_bytes())
}

pub enum RvcEvent {
    Mdns(mdns::Event),
    Ping(ping::Event),
    ReqRes(RequestResponseEvent<SyncRequest, SyncResponse>),
    Kad(kad::Event),
}

impl From<request_response::Event<SyncRequest, SyncResponse>> for RvcEvent {
    fn from(event: request_response::Event<SyncRequest, SyncResponse>) -> Self {
        RvcEvent::ReqRes(event)
    }
}

impl From<mdns::Event> for RvcEvent {
    fn from(event: mdns::Event) -> Self {
        RvcEvent::Mdns(event)
    }
}

impl From<ping::Event> for RvcEvent {
    fn from(event: ping::Event) -> Self {
        RvcEvent::Ping(event)
    }
}

impl From<kad::Event> for RvcEvent {
    fn from(event: kad::Event) -> Self {
        RvcEvent::Kad(event)
    }
}