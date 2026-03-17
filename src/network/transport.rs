use libp2p::{
    core::upgrade,
    identity::Keypair,
    noise,
    tcp,
    yamux,
    Transport,
};
use std::error::Error;

pub fn build_transport(keypair: &Keypair,) -> Result<
    libp2p::core::transport::Boxed<(libp2p::PeerId, libp2p::core::muxing::StreamMuxerBox)>,
    Box<dyn Error>,
> {
    let noise_config = noise::Config::new(keypair)?;

    let transport = tcp::tokio::Transport::new(tcp::Config::default())
        .upgrade(upgrade::Version::V1)
        .authenticate(noise_config)
        .multiplex(yamux::Config::default())
        .boxed();

    Ok(transport)
}