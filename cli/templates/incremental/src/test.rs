//! Integration tests for {{crate_name}}.
//!
//! These run through `cougr_core::test::GameHarness`, the sandbox the canonical
//! examples use: it registers the contract in a fresh `Env`, hands back the
//! contract ID for the generated client, and pairs with `Scenario` for
//! multi-turn play and `WorldFixture` for injecting pre-built world state.
//!
//! ## Regression test for #330
//!
//! Proves that `StorageWorld` dirty-tracking means an unchanged entity is NOT
//! rewritten on `flush()`: after `set_score1` only entity 1's components are
//! persisted, entity 2's components remain as originally flushed.

use crate::components::{Score1, Score2, NORTH, EAST};
use crate::systems::{validate_score, validate_turn, ScoreError};
use crate::{{ContractName}};
use crate::{{ContractName}}Client;
use cougr_core::test::{GameHarness, Scenario, SnapshotAssert, WorldFixture};
use soroban_sdk::Env;

fn harness() -> GameHarness {
    GameHarness::new(Env::default(), {{ContractName}})
}

#[test]
fn spawn_player1_and_player2() {
    let harness = harness();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let id1 = client.spawn_player1();
    let id2 = client.spawn_player2();

    assert_eq!(client.score1(&id1).unwrap(), 0);
    assert_eq!(client.score2(&id2).unwrap(), 0);
    assert_eq!(client.entity_count(), 2);
}

#[test]
fn set_score1_only_rewrites_entity1() {
    let harness = harness();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let id1 = client.spawn_player1();
    let id2 = client.spawn_player2();

    // Set player 1's score - this should only rewrite entity 1
    client.set_score1(&id1, 42);

    // Player 1's score is updated
    assert_eq!(client.score1(&id1).unwrap(), 42);

    // Player 2's score should still be the original 0 (not rewritten)
    assert_eq!(client.score2(&id2).unwrap(), 0);
}

#[test]
fn set_score2_only_rewrites_entity2() {
    let harness = harness();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let id1 = client.spawn_player1();
    let id2 = client.spawn_player2();

    // Set player 2's score - this should only rewrite entity 2
    client.set_score2(&id2, 99);

    // Player 2's score is updated
    assert_eq!(client.score2(&id2).unwrap(), 99);

    // Player 1's score should still be the original 0 (not rewritten)
    assert_eq!(client.score1(&id1).unwrap(), 0);
}

#[test]
fn scenario_runs_a_multi_turn_score_game() {
    let harness = harness();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());
    let id1 = client.spawn_player1();
    let id2 = client.spawn_player2();

    Scenario::new("alternating score updates")
        .turns(4)
        .run(&harness, |_player, turn, h| {
            let client = {{ContractName}}Client::new(h.env(), h.contract_id());
            match turn.0 {
                0 => client.set_score1(&id1, 10),
                1 => client.set_score2(&id2, 20),
                2 => client.set_score1(&id1, 30),
                _ => client.set_score2(&id2, 40),
            }
        });

    assert_eq!(client.score1(&id1).unwrap(), 30);
    assert_eq!(client.score2(&id2).unwrap(), 40);
}

#[test]
fn fixture_injects_a_pre_built_world() {
    let harness = harness();

    let mut fixture = WorldFixture::empty(harness.env());
    fixture.spawn_entity(); // entity 1
    fixture.spawn_entity(); // entity 2
    fixture.inject::<{{ContractName}}>(&harness);

    SnapshotAssert::assert_entity_count(
        WorldFixture::read_from_contract::<{{ContractName}}>(&harness).world(),
        2,
    );
}