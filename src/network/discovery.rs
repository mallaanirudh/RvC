use libp2p::{
    mdns,
    swarm::NetworkBehaviour,
    PeerId,
};
use std::error::Error;

#[derive(NetworkBehaviour)]
pub struct DiscoveryBehaviour {
    pub mdns: mdns::tokio::Behaviour,
}

impl DiscoveryBehaviour {
    pub async fn new(peer_id: PeerId) -> Result<Self, Box<dyn Error>> {
        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(),
            peer_id,
        )?;

        Ok(Self { mdns })
    }
}