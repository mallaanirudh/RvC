use rvc::network::identity::load_or_generate_identity;

#[test]
fn identity_is_created() {
    let identity = load_or_generate_identity();
    assert!(!identity.peer_id.to_string().is_empty());
}