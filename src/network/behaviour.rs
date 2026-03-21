use libp2p::{
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

use crate::network::protocol::*;
use std::iter;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "RvcEvent")]
#[behaviour(prelude = "libp2p::swarm::derive_prelude")]
pub struct RvcBehaviour {
    pub mdns: mdns::tokio::Behaviour,
    pub ping: ping::Behaviour,
    pub req_res: RequestResponseBehaviour<RvcCodec>,
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

        Ok(Self { mdns, ping, req_res })
    }
}

pub enum RvcEvent {
    Mdns(mdns::Event),
    Ping(ping::Event),
    ReqRes(RequestResponseEvent<RvcRequest, RvcResponse>),
}

impl From<request_response::Event<RvcRequest, RvcResponse>> for RvcEvent {
    fn from(event: request_response::Event<RvcRequest, RvcResponse>) -> Self {
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