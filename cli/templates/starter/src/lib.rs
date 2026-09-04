//! {{crate_name}} - a Cougr game contract generated from the `{{template_id}}` template.
//!
//! A player calls `spawn` to enter the world and receives an entity ID, then
//! calls `move_entity` to walk one step in one of four directions. Every
//! position change emits an indexed Soroban event.
//!
//! Demonstrates:
//!   - `impl_component_observed!` - ECS component plus indexer-friendly events
//!   - `impl_component!` - private ECS component, no events
//!   - `SorobanGame` - standard world load/save
//!   - `impl_soroban_game!` - wires the trait to a `#[contract]` struct

#![no_std]

pub mod components;
pub mod systems;
#[cfg(test)]
mod test;

use components::{Moves, Position, DEFAULT_MOVE_BUDGET, NORTH};
use systems::{step, MoveError};

use cougr_core::game::SorobanGame;
use cougr_core::impl_soroban_game;
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
#[derive(Clone)]
pub struct {{ContractName}};

// Generates `load_world` / `save_world` against the "world" instance-storage key.
impl_soroban_game!({{ContractName}}, "world");

#[contractimpl]
impl {{ContractName}} {
    /// Spawn a new player entity at the world origin.
    ///
    /// Returns the entity ID the caller passes to every subsequent call, and
    /// emits a `(COUGR, set, position)` event so indexers register the spawn.
    pub fn spawn(env: Env) -> u32 {
        let mut world = {{ContractName}}::load_world(&env);

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

        {{ContractName}}::save_world(&env, &world);
        entity
    }

    /// Move an entity one step in `direction`.
    ///
    /// Directions: `0` = North (+y), `1` = East (+x), `2` = South (−y),
    /// `3` = West (−x).
    ///
    /// Panics if the entity does not exist, has no moves left, or the direction
    /// is out of range.
    pub fn move_entity(env: Env, entity_id: u32, direction: u32) {
        let mut world = {{ContractName}}::load_world(&env);

        let position = world
            .get_typed::<Position>(&env, entity_id)
            .unwrap_or_else(|| panic!("entity not found"));
        let moves = world
            .get_typed::<Moves>(&env, entity_id)
            .unwrap_or_else(|| panic!("entity not found"));

        let (position, moves) = match step(&position, &moves, direction) {
            Ok(next) => next,
            Err(MoveError::InvalidDirection) => panic!("invalid direction"),
            Err(MoveError::NoMovesRemaining) => panic!("no moves remaining"),
        };

        world.set_typed_observed(&env, entity_id, &position);
        world.set_typed(&env, entity_id, &moves);

        {{ContractName}}::save_world(&env, &world);
    }

    /// Current position of `entity_id`, or `None` if it never spawned.
    pub fn position(env: Env, entity_id: u32) -> Option<Position> {
        let world = {{ContractName}}::load_world(&env);
        world.get_typed::<Position>(&env, entity_id)
    }

    /// Remaining move budget of `entity_id`, or `None` if it never spawned.
    pub fn moves(env: Env, entity_id: u32) -> Option<Moves> {
        let world = {{ContractName}}::load_world(&env);
        world.get_typed::<Moves>(&env, entity_id)
    }

    /// Total number of entities spawned so far.
    pub fn entity_count(env: Env) -> u32 {
        let world = {{ContractName}}::load_world(&env);
        world.next_entity_id().saturating_sub(1)
    }
}
