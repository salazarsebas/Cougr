#![cfg(test)]

use crate::{SpawnAndMoveClient, EAST, NORTH, SOUTH, WEST};
use soroban_sdk::{testutils::Events, Env};

fn setup() -> (Env, SpawnAndMoveClient<'static>) {
    let env = Env::default();
    let contract_id = env.register(crate::SpawnAndMove, ());
    let client = SpawnAndMoveClient::new(&env, &contract_id);
    (env, client)
}

// ─── Spawn ────────────────────────────────────────────────────────────────────

#[test]
fn spawn_returns_entity_id() {
    let (_, client) = setup();
    let id = client.spawn();
    assert_eq!(id, 1);
}

#[test]
fn spawn_multiple_entities_increment_ids() {
    let (_, client) = setup();
    let a = client.spawn();
    let b = client.spawn();
    let c = client.spawn();
    assert_eq!((a, b, c), (1, 2, 3));
}

#[test]
fn spawn_places_entity_at_origin() {
    let (_, client) = setup();
    let id = client.spawn();
    let pos = client.position(&id).unwrap();
    assert_eq!((pos.x, pos.y), (0, 0));
}

#[test]
fn spawn_emits_position_set_event() {
    let (env, client) = setup();
    client.spawn();
    // impl_component_observed! emits a (COUGR, set, position) event on every
    // set_typed_observed call. Contract invocations via the client populate events.
    let count = env.events().all().events().len();
    assert!(count > 0);
}

#[test]
fn entity_count_reflects_spawns() {
    let (_, client) = setup();
    assert_eq!(client.entity_count(), 0);
    client.spawn();
    assert_eq!(client.entity_count(), 1);
    client.spawn();
    assert_eq!(client.entity_count(), 2);
}

// ─── Movement ────────────────────────────────────────────────────────────────

#[test]
fn move_north_increments_y() {
    let (_, client) = setup();
    let id = client.spawn();
    client.move_entity(&id, &NORTH);
    let pos = client.position(&id).unwrap();
    assert_eq!((pos.x, pos.y), (0, 1));
}

#[test]
fn move_south_decrements_y() {
    let (_, client) = setup();
    let id = client.spawn();
    client.move_entity(&id, &SOUTH);
    let pos = client.position(&id).unwrap();
    assert_eq!((pos.x, pos.y), (0, -1));
}

#[test]
fn move_east_increments_x() {
    let (_, client) = setup();
    let id = client.spawn();
    client.move_entity(&id, &EAST);
    let pos = client.position(&id).unwrap();
    assert_eq!((pos.x, pos.y), (1, 0));
}

#[test]
fn move_west_decrements_x() {
    let (_, client) = setup();
    let id = client.spawn();
    client.move_entity(&id, &WEST);
    let pos = client.position(&id).unwrap();
    assert_eq!((pos.x, pos.y), (-1, 0));
}

#[test]
fn sequential_moves_accumulate() {
    let (_, client) = setup();
    let id = client.spawn();
    client.move_entity(&id, &NORTH);
    client.move_entity(&id, &NORTH);
    client.move_entity(&id, &EAST);
    let pos = client.position(&id).unwrap();
    assert_eq!((pos.x, pos.y), (1, 2));
}

#[test]
fn move_decrements_budget() {
    let (_, client) = setup();
    let id = client.spawn();
    let before = client.moves(&id).unwrap().remaining;
    client.move_entity(&id, &NORTH);
    let after = client.moves(&id).unwrap().remaining;
    assert_eq!(after, before - 1);
}

#[test]
fn move_records_last_direction() {
    let (_, client) = setup();
    let id = client.spawn();
    client.move_entity(&id, &EAST);
    let moves = client.moves(&id).unwrap();
    assert_eq!(moves.last_direction, EAST);
}

#[test]
fn multiple_entities_independent_positions() {
    let (_, client) = setup();
    let a = client.spawn();
    let b = client.spawn();
    client.move_entity(&a, &NORTH);
    client.move_entity(&b, &EAST);
    let pa = client.position(&a).unwrap();
    let pb = client.position(&b).unwrap();
    assert_eq!((pa.x, pa.y), (0, 1));
    assert_eq!((pb.x, pb.y), (1, 0));
}

#[test]
fn move_emits_position_event() {
    let (env, client) = setup();
    let id = client.spawn();
    // After a move, the most recent invocation should have emitted at least
    // one (COUGR, set, position) event.
    client.move_entity(&id, &NORTH);
    let count = env.events().all().events().len();
    assert!(count > 0);
}

// ─── Error paths ─────────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "entity not found")]
fn move_unknown_entity_panics() {
    let (_, client) = setup();
    client.move_entity(&999, &NORTH);
}

#[test]
#[should_panic(expected = "invalid direction")]
fn move_invalid_direction_panics() {
    let (_, client) = setup();
    let id = client.spawn();
    client.move_entity(&id, &99);
}

#[test]
fn position_unknown_entity_returns_none() {
    let (_, client) = setup();
    assert!(client.position(&999).is_none());
}

#[test]
fn moves_unknown_entity_returns_none() {
    let (_, client) = setup();
    assert!(client.moves(&999).is_none());
}
