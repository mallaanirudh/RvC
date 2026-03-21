use libp2p::{
    mdns,
    ping,
    swarm::NetworkBehaviour,
};

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "RvcEvent")]
pub struct RvcBehaviour {
    pub mdns: mdns::tokio::Behaviour,
    pub ping: ping::Behaviour,
}
impl RvcBehaviour {
    pub async fn new(peer_id: libp2p::PeerId) -> Result<Self, Box<dyn std::error::Error>> {
        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(),
            peer_id,
        )?;

        let ping = ping::Behaviour::default();

        Ok(Self { mdns, ping })
    }
}
pub enum RvcEvent {
    Mdns(mdns::Event),
    Ping(ping::Event),
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