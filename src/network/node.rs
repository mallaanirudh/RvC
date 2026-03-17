use crate::network::{
    discovery::DiscoveryBehaviour,
    identity::load_or_generate_identity,
    transport::build_transport,
};

use futures::StreamExt;
use libp2p::{
    swarm::{Swarm, SwarmEvent},
    Multiaddr,
};

pub async fn run_node() -> Result<(), Box<dyn std::error::Error>> {

    let identity = load_or_generate_identity();

    println!("Local peer id: {}", identity.peer_id);

    let transport = build_transport(&identity.keypair)?;

    let behaviour = DiscoveryBehaviour::new(identity.peer_id).await?;

    let mut swarm = Swarm::new(
        transport,
        behaviour,
        identity.peer_id,
        libp2p::swarm::Config::with_tokio_executor(),
    );

    let addr: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse()?;
    swarm.listen_on(addr)?;

    loop {
        match swarm.select_next_some().await {

            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {address}");
            }

            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("Connected to {peer_id}");
            }

            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                println!("Disconnected from {peer_id}");
            }

            _ => {}
        }
    }
}