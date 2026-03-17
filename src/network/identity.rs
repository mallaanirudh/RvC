use libp2p::identity;
use libp2p::PeerId;
use std::fs;
use std::path::PathBuf;

pub struct NodeIdentity {
    pub keypair: identity::Keypair,
    pub peer_id: PeerId,
}

fn identity_path() -> PathBuf {
    let home = dirs::home_dir().expect("could not determine home directory");
    home.join(".rvc").join("peer_key")
}

pub fn load_or_generate_identity() -> NodeIdentity {
    let path = identity_path();

    let keypair = if path.exists() {
        let bytes = fs::read(&path).expect("failed to read peer key");
        identity::Keypair::from_protobuf_encoding(&bytes)
            .expect("invalid peer key encoding")
    } else {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create ~/.rvc directory");
        }

        let keypair = identity::Keypair::generate_ed25519();

        let encoded = keypair
            .to_protobuf_encoding()
            .expect("failed to encode keypair");

        fs::write(&path, encoded).expect("failed to write peer key");

        keypair
    };

    let peer_id = PeerId::from(keypair.public());

    NodeIdentity { keypair, peer_id }
}