use super::*;
use cougr_core::circuits::{fog_of_war, test_fixtures, CircuitId};
use cougr_core::test::GameHarness;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

#[test]
fn init_map_configures_fog_circuit() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, FogExplorer);
    let client = FogExplorerClient::new(harness.env(), harness.contract_id());

    let config = client.init_map(&32, &32, &3);
    assert_eq!(config.width, 32);
    assert_eq!(config.visibility_radius, 3);

    let spec = fog_of_war(harness.env(), 32, 32, 3).unwrap();
    assert_eq!(spec.circuit_id, CircuitId::FogOfWar);
    assert_eq!(spec.layout.public_input_count(), 8);
}

#[test]
fn explore_accepts_pipeline_proof() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, FogExplorer);
    let client = FogExplorerClient::new(harness.env(), harness.contract_id());
    let player = Address::generate(harness.env());
    let public = test_fixtures::pipeline_public_inputs(harness.env(), CircuitId::FogOfWar);
    let proof = test_fixtures::pipeline_proof(harness.env(), CircuitId::FogOfWar);

    client.init_map(&32, &32, &3);
    let prior = BytesN::from_array(harness.env(), &public[1].bytes.to_array());
    client.register_explorer(&player, &prior);

    let map_root = BytesN::from_array(harness.env(), &public[0].bytes.to_array());
    let next = BytesN::from_array(harness.env(), &public[2].bytes.to_array());
    assert!(client.explore(&player, &map_root, &prior, &next, &0, &0, &1, &2, &proof,));
}

#[test]
fn explore_rejects_mismatched_map_root() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, FogExplorer);
    let client = FogExplorerClient::new(harness.env(), harness.contract_id());
    let player = Address::generate(harness.env());
    let public = test_fixtures::pipeline_public_inputs(harness.env(), CircuitId::FogOfWar);
    let proof = test_fixtures::pipeline_proof(harness.env(), CircuitId::FogOfWar);

    client.init_map(&32, &32, &3);
    let prior = BytesN::from_array(harness.env(), &public[1].bytes.to_array());
    client.register_explorer(&player, &prior);

    let bad_root = BytesN::from_array(harness.env(), &[0u8; 32]);
    let next = BytesN::from_array(harness.env(), &public[2].bytes.to_array());
    assert!(!client.explore(
        &player, &bad_root, &prior, &next, &0, &0, &1, &2, &proof,
    ));
}
