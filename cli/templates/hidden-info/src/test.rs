//! Integration tests for {{crate_name}}.
//!
//! These run through `cougr_core::test::GameHarness`, the sandbox the canonical
//! examples use. The proof and commitments come from
//! `cougr_core::circuits::test_fixtures`, which ships the artifacts produced by
//! the Circom pipeline - the fixture hand belongs to seat `2` of a 52-card,
//! 5-card-hand table, so the tests seat three players before proving.

use crate::components::TableConfig;
use crate::systems::{validate_config, TableError, MAX_DECK_SIZE};
use crate::{{ContractName}};
use crate::{{ContractName}}Client;
use cougr_core::circuits::{test_fixtures, CircuitId};
use cougr_core::test::{GameHarness, PlayerSlot};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env};

const DECK_SIZE: u32 = 52;
const HAND_SIZE: u32 = 5;
/// Seat the pipeline fixture proof was generated for.
const PROVING_SEAT: u32 = 2;

/// A harness with three seated players and an initialised table.
fn seated_table() -> GameHarness {
    let env = Env::default();
    let mut harness = GameHarness::new(env, {{ContractName}});
    harness.mock_players(3);
    harness.mock_all_auths();

    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());
    client.init_table(&DECK_SIZE, &HAND_SIZE);
    for slot in 0..3 {
        client.join_table(harness.player(PlayerSlot(slot)));
    }

    harness
}

/// The deck root and hand commitment the fixture proof was built from.
fn fixture_commitments(env: &Env) -> (BytesN<32>, BytesN<32>) {
    let public = test_fixtures::pipeline_public_inputs(env, CircuitId::HiddenCards);
    (
        BytesN::from_array(env, &public[0].bytes.to_array()),
        BytesN::from_array(env, &public[1].bytes.to_array()),
    )
}

#[test]
fn init_table_freezes_the_configuration() {
    let harness = seated_table();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let expected = TableConfig {
        deck_size: DECK_SIZE,
        hand_size: HAND_SIZE,
    };
    assert_eq!(client.table(), Some(expected));
}

#[test]
#[should_panic(expected = "hand size out of range")]
fn a_hand_larger_than_the_deck_is_rejected() {
    let env = Env::default();
    let harness = GameHarness::new(env, {{ContractName}});
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    client.init_table(&10, &11);
}

#[test]
#[should_panic(expected = "deck size out of range")]
fn an_oversized_deck_is_rejected() {
    let env = Env::default();
    let harness = GameHarness::new(env, {{ContractName}});
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    client.init_table(&(MAX_DECK_SIZE + 1), &5);
}

#[test]
fn seats_are_handed_out_in_join_order() {
    let harness = seated_table();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    for slot in 0..3u32 {
        let player = harness.player(PlayerSlot(slot));
        assert_eq!(client.seat_of(player), Some(slot));
    }
}

#[test]
fn joining_twice_keeps_the_same_seat() {
    let harness = seated_table();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());
    let player = harness.player(PlayerSlot(1));

    assert_eq!(client.join_table(player), 1);
    assert_eq!(client.seat_of(player), Some(1));
}

#[test]
fn a_valid_deal_proof_verifies_and_is_counted() {
    let harness = seated_table();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());
    let player = harness.player(PlayerSlot(PROVING_SEAT));
    let (deck_root, hand) = fixture_commitments(harness.env());
    let proof = test_fixtures::pipeline_proof(harness.env(), CircuitId::HiddenCards);

    assert!(client.verify_deal(player, &deck_root, &hand, &proof));
    assert_eq!(client.deals_verified(player), 1);
}

#[test]
fn a_tampered_hand_commitment_does_not_verify() {
    let harness = seated_table();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());
    let player = harness.player(PlayerSlot(PROVING_SEAT));
    let (deck_root, _) = fixture_commitments(harness.env());
    let proof = test_fixtures::pipeline_proof(harness.env(), CircuitId::HiddenCards);
    let forged = BytesN::from_array(harness.env(), &[0u8; 32]);

    assert!(!client.verify_deal(player, &deck_root, &forged, &proof));
    assert_eq!(client.deals_verified(player), 0);
}

#[test]
fn a_proof_for_another_seat_does_not_verify() {
    let harness = seated_table();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());
    let other_seat = harness.player(PlayerSlot(0));
    let (deck_root, hand) = fixture_commitments(harness.env());
    let proof = test_fixtures::pipeline_proof(harness.env(), CircuitId::HiddenCards);

    assert!(!client.verify_deal(other_seat, &deck_root, &hand, &proof));
}

#[test]
#[should_panic(expected = "player has not joined the table")]
fn an_unseated_player_cannot_submit_a_deal() {
    let harness = seated_table();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());
    let stranger = Address::generate(harness.env());
    let (deck_root, hand) = fixture_commitments(harness.env());
    let proof = test_fixtures::pipeline_proof(harness.env(), CircuitId::HiddenCards);

    client.verify_deal(&stranger, &deck_root, &hand, &proof);
}

#[test]
fn validate_config_rejects_an_empty_deck() {
    let config = TableConfig {
        deck_size: 0,
        hand_size: 1,
    };

    assert_eq!(
        validate_config(&config),
        Err(TableError::DeckSizeOutOfRange)
    );
}
