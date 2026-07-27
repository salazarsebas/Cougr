//! Integration tests for {{crate_name}}.
//!
//! These run through `cougr_core::test::GameHarness`, the sandbox the canonical
//! examples use: it registers the contract in a fresh `Env`, hands back the
//! contract ID for the generated client, and pairs with `Scenario` for
//! multi-turn play and `WorldFixture` for injecting pre-built world state.

use crate::components::{Moves, Position, EAST, NORTH, SOUTH};
use crate::systems::{step, MoveError};
use crate::{{ContractName}};
use crate::{{ContractName}}Client;
use cougr_core::test::{GameHarness, Scenario, SnapshotAssert, WorldFixture};
use soroban_sdk::Env;

fn harness() -> GameHarness {
    GameHarness::new(Env::default(), {{ContractName}})
}

#[test]
fn spawn_places_the_entity_at_the_origin() {
    let harness = harness();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let id = client.spawn();
    let position = client.position(&id).unwrap();

    assert_eq!((position.x, position.y), (0, 0));
    assert_eq!(client.entity_count(), 1);
}

#[test]
fn moving_updates_position_and_spends_the_budget() {
    let harness = harness();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let id = client.spawn();
    let budget = client.moves(&id).unwrap().remaining;

    client.move_entity(&id, &NORTH);
    client.move_entity(&id, &EAST);

    let position = client.position(&id).unwrap();
    assert_eq!((position.x, position.y), (1, 1));
    assert_eq!(client.moves(&id).unwrap().remaining, budget - 2);
    assert_eq!(client.moves(&id).unwrap().last_direction, EAST);
}

#[test]
#[should_panic(expected = "invalid direction")]
fn moving_in_an_unknown_direction_is_rejected() {
    let harness = harness();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let id = client.spawn();
    client.move_entity(&id, &99);
}

#[test]
#[should_panic(expected = "entity not found")]
fn moving_an_unspawned_entity_is_rejected() {
    let harness = harness();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    client.move_entity(&404, &NORTH);
}

#[test]
fn entities_move_independently() {
    let harness = harness();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let first = client.spawn();
    let second = client.spawn();
    client.move_entity(&first, &NORTH);

    assert_eq!(client.position(&first).unwrap(), Position { x: 0, y: 1 });
    assert_eq!(client.position(&second).unwrap(), Position { x: 0, y: 0 });
}

#[test]
fn scenario_runs_a_multi_turn_walk() {
    let harness = harness();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());
    let id = client.spawn();

    Scenario::new("walk a triangle")
        .turns(3)
        .run(&harness, |_player, turn, h| {
            let client = {{ContractName}}Client::new(h.env(), h.contract_id());
            match turn.0 {
                0 => client.move_entity(&id, &NORTH),
                1 => client.move_entity(&id, &EAST),
                _ => client.move_entity(&id, &SOUTH),
            }
        });

    assert_eq!(client.position(&id).unwrap(), Position { x: 1, y: 0 });
}

#[test]
fn fixture_injects_a_pre_built_world() {
    let harness = harness();

    let mut fixture = WorldFixture::empty(harness.env());
    fixture.spawn_entity();
    fixture.spawn_entity();
    fixture.inject::<{{ContractName}}>(&harness);

    SnapshotAssert::assert_entity_count(
        WorldFixture::read_from_contract::<{{ContractName}}>(&harness).world(),
        2,
    );
}

#[test]
fn step_reports_an_exhausted_move_budget() {
    let position = Position { x: 0, y: 0 };
    let spent = Moves {
        remaining: 0,
        last_direction: NORTH,
    };

    assert_eq!(
        step(&position, &spent, NORTH),
        Err(MoveError::NoMovesRemaining)
    );
}
