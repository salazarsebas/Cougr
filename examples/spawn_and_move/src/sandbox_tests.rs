#![cfg(test)]

use crate::{SpawnAndMove, SpawnAndMoveClient, EAST, NORTH, SOUTH};
use cougr_core::test::{GameHarness, Scenario, SnapshotAssert, WorldFixture};
use soroban_sdk::Env;

#[test]
fn sandbox_scenario_spawns_four_entities() {
    let env = Env::default();
    let harness = GameHarness::new(env, SpawnAndMove);

    Scenario::new("four spawns")
        .players(1)
        .turns(4)
        .run_and_assert(
            &harness,
            |_player, _turn, h| {
                SpawnAndMoveClient::new(h.env(), h.contract_id()).spawn();
            },
            |h| {
                let client = SpawnAndMoveClient::new(h.env(), h.contract_id());
                assert_eq!(client.entity_count(), 4);
            },
        );
}

#[test]
fn sandbox_fixture_injects_entity_count() {
    let env = Env::default();
    let harness = GameHarness::new(env, SpawnAndMove);

    let mut fixture = WorldFixture::empty(harness.env());
    fixture.spawn_entity();
    fixture.spawn_entity();
    fixture.spawn_entity();
    fixture.inject::<SpawnAndMove>(&harness);

    SnapshotAssert::assert_entity_count(
        WorldFixture::read_from_contract::<SpawnAndMove>(&harness).world(),
        3,
    );
}

#[test]
fn sandbox_multi_turn_movement_sequence() {
    let env = Env::default();
    let harness = GameHarness::new(env, SpawnAndMove);
    let client = SpawnAndMoveClient::new(harness.env(), harness.contract_id());
    let id = client.spawn();

    Scenario::new("north twice")
        .turns(2)
        .run(&harness, |_player, turn, h| {
            let c = SpawnAndMoveClient::new(h.env(), h.contract_id());
            c.move_entity(&id, &NORTH);
            if turn.0 == 1 {
                let pos = c.position(&id).unwrap();
                assert_eq!((pos.x, pos.y), (0, 2));
            }
        });
}

#[test]
fn sandbox_three_step_movement_scenario() {
    let env = Env::default();
    let harness = GameHarness::new(env, SpawnAndMove);
    let client = SpawnAndMoveClient::new(harness.env(), harness.contract_id());
    let id = client.spawn();

    Scenario::new("three step movement")
        .turns(3)
        .run(&harness, |_player, turn, h| {
            let c = SpawnAndMoveClient::new(h.env(), h.contract_id());
            match turn.0 {
                0 => {
                    c.move_entity(&id, &NORTH);
                    let pos = c.position(&id).unwrap();
                    assert_eq!((pos.x, pos.y), (0, 1));
                }
                1 => {
                    c.move_entity(&id, &EAST);
                    let pos = c.position(&id).unwrap();
                    assert_eq!((pos.x, pos.y), (1, 1));
                }
                2 => {
                    c.move_entity(&id, &SOUTH);
                    let pos = c.position(&id).unwrap();
                    assert_eq!((pos.x, pos.y), (1, 0));
                }
                _ => {}
            }
        });
}
