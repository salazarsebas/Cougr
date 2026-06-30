// spawn_and_move — canonical Cougr "hello world" game.
//
// A player calls `spawn` to enter the world and receives an entity ID.
// They then call `move_entity` to walk in one of four directions.
// Every position change emits an indexed Soroban event so off-chain
// clients can track movement in real time.
//
// Demonstrates:
//   - impl_component_observed! — ECS component + indexer-friendly events
//   - impl_component!          — private ECS component (no events)
//   - SorobanGame              — standard load/save boilerplate
//   - impl_soroban_game!       — wires the trait to a Soroban contract

#![no_std]

use cougr_core::game::SorobanGame;
use cougr_core::{impl_component, impl_component_observed, impl_soroban_game};
use soroban_sdk::{contract, contractimpl, contracttype, Env};

// ─── Error ────────────────────────────────────────────────────────────────────

#[contracttype]
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum GameError {
    EntityNotFound = 1,
    NoMovesRemaining = 2,
    InvalidDirection = 3,
}

// ─── Components ───────────────────────────────────────────────────────────────

/// World-space position of an entity. Changes emit an indexed Soroban event
/// so off-chain clients can track movement without polling.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl_component_observed!(Position, "position", Table, { x: i32, y: i32 });

/// Remaining move budget and last direction taken. Not observed: clients
/// discover this through explicit queries rather than event streams.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Moves {
    pub remaining: u32,
    pub last_direction: u32,
}

impl_component!(Moves, "moves", Sparse, { remaining: u32, last_direction: u32 });

// ─── Direction constants ──────────────────────────────────────────────────────

pub const NORTH: u32 = 0; // +y
pub const EAST: u32 = 1; // +x
pub const SOUTH: u32 = 2; // −y
pub const WEST: u32 = 3; // −x

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
#[derive(Clone)]
pub struct SpawnAndMove;

impl_soroban_game!(SpawnAndMove, "world");

const DEFAULT_MOVE_BUDGET: u32 = 1_000;

#[contractimpl]
impl SpawnAndMove {
    /// Spawn a new player entity at the world origin.
    ///
    /// Returns the entity ID that the caller must pass to subsequent calls.
    /// Emits a `(COUGR, set, position)` event so indexers register the spawn.
    pub fn spawn(env: Env) -> u32 {
        let mut world = SpawnAndMove::load_world(&env);

        let entity = world.spawn_entity();
        world.set_typed_observed(&env, entity, &Position { x: 0, y: 0 });
        world.set_typed(
            &env,
            entity,
            &Moves {
                remaining: DEFAULT_MOVE_BUDGET,
                last_direction: NORTH,
            },
        );

        SpawnAndMove::save_world(&env, &world);
        entity
    }

    /// Move an entity one step in the given direction.
    ///
    /// Directions: `0` = North (+y), `1` = East (+x), `2` = South (−y),
    /// `3` = West (−x).
    ///
    /// Emits a `(COUGR, set, position)` event on success.
    /// Panics if the entity does not exist, has no moves remaining, or if
    /// an invalid direction is supplied.
    pub fn move_entity(env: Env, entity_id: u32, direction: u32) {
        if direction > 3 {
            panic!("invalid direction");
        }

        let mut world = SpawnAndMove::load_world(&env);

        let mut pos: Position = world
            .get_typed::<Position>(&env, entity_id)
            .unwrap_or_else(|| panic!("entity not found"));

        let mut moves: Moves = world
            .get_typed::<Moves>(&env, entity_id)
            .unwrap_or_else(|| panic!("entity not found"));

        if moves.remaining == 0 {
            panic!("no moves remaining");
        }

        match direction {
            NORTH => pos.y += 1,
            EAST => pos.x += 1,
            SOUTH => pos.y -= 1,
            WEST => pos.x -= 1,
            _ => panic!("invalid direction"),
        }
        moves.remaining -= 1;
        moves.last_direction = direction;

        world.set_typed_observed(&env, entity_id, &pos);
        world.set_typed(&env, entity_id, &moves);

        SpawnAndMove::save_world(&env, &world);
    }

    /// Return the current position of `entity_id`, or `None` if not spawned.
    pub fn position(env: Env, entity_id: u32) -> Option<Position> {
        let world = SpawnAndMove::load_world(&env);
        world.get_typed::<Position>(&env, entity_id)
    }

    /// Return the moves state of `entity_id`, or `None` if not spawned.
    pub fn moves(env: Env, entity_id: u32) -> Option<Moves> {
        let world = SpawnAndMove::load_world(&env);
        world.get_typed::<Moves>(&env, entity_id)
    }

    /// Return the total number of entities that have spawned.
    pub fn entity_count(env: Env) -> u32 {
        let world = SpawnAndMove::load_world(&env);
        world.next_entity_id().saturating_sub(1)
    }
}

#[cfg(test)]
mod sandbox_tests;
#[cfg(test)]
mod test;
