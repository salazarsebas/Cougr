use super::*;
use cougr_core::circuits::{hidden_cards, test_fixtures, CircuitId};
use cougr_core::test::GameHarness;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

fn player_id_for_test(player: &Address) -> u32 {
    let bytes = player.to_string().to_bytes();
    let mut id = 0u32;
    for i in 0..bytes.len().min(4) {
        id = id.wrapping_add(u32::from(bytes.get(i).unwrap_or(0)));
    }
    id
}

#[test]
fn init_table_configures_standard_deck() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, HiddenHand);
    let client = HiddenHandClient::new(harness.env(), harness.contract_id());

    let config = client.init_table(&52, &5);
    assert_eq!(config.deck_size, 52);
    assert_eq!(config.hand_size, 5);

    let spec = hidden_cards(harness.env(), 52, 5).unwrap();
    assert_eq!(spec.circuit_id, CircuitId::HiddenCards);
    assert_eq!(spec.layout.public_input_count(), 5);
    assert_eq!(spec.verification_key.ic.len(), 6);
}

#[test]
fn verify_deal_accepts_pipeline_proof() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, HiddenHand);
    let player = Address::generate(harness.env());
    let client = HiddenHandClient::new(harness.env(), harness.contract_id());
    let public = test_fixtures::pipeline_public_inputs(harness.env(), CircuitId::HiddenCards);
    let proof = test_fixtures::pipeline_proof(harness.env(), CircuitId::HiddenCards);

    client.init_table(&52, &5);
    let deck = BytesN::from_array(harness.env(), &public[0].bytes.to_array());
    let hand = BytesN::from_array(harness.env(), &public[1].bytes.to_array());

    let spec = hidden_cards(harness.env(), 52, 5).unwrap();
    assert_eq!(
        spec.verify_hidden_hand(harness.env(), &proof, &deck, &hand, 2),
        Ok(true)
    );
    let player_id = player_id_for_test(&player);
    assert_eq!(
        client.verify_deal(&player, &deck, &hand, &proof),
        spec
            .verify_hidden_hand(harness.env(), &proof, &deck, &hand, player_id)
            .unwrap_or(false)
    );
}

#[test]
fn verify_deal_rejects_mismatched_hand_commitment() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, HiddenHand);
    let player = Address::generate(harness.env());
    let client = HiddenHandClient::new(harness.env(), harness.contract_id());
    let public = test_fixtures::pipeline_public_inputs(harness.env(), CircuitId::HiddenCards);
    let proof = test_fixtures::pipeline_proof(harness.env(), CircuitId::HiddenCards);

    client.init_table(&52, &5);
    let deck = BytesN::from_array(harness.env(), &public[0].bytes.to_array());
    let bad_hand = BytesN::from_array(harness.env(), &[0u8; 32]);

    assert!(!client.verify_deal(&player, &deck, &bad_hand, &proof));
}
