//! {{crate_name}} - a Cougr game contract generated from the `{{template_id}}` template.
//!
//! Two players take turns setting each other's scores. The `incremental`
//! template demonstrates `StorageWorld` dirty-tracking: only the entity
//! whose score changed is rewritten on `flush()`.
//!
//! Demonstrates:
//!   - `StorageWorld` - incremental per-entity persistent storage
//!   - `impl_soroban_game!` - standard world load/save
//!   - `test::GameHarness`, `Scenario`, `WorldFixture` - integration test sandbox

#![no_std]

pub mod components;
pub mod systems;
#[cfg(test)]
mod test;

use components::{Score1, Score2, DEFAULT_SCORE};
use systems::{validate_score, validate_turn};
use cougr_core::game::SorobanGame;
use cougr_core::impl_soroban_game;
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
#[derive(Clone)]
pub struct {{ContractName}};

//! Generates `load_world` / `save_world` against the "world" instance-storage key.
impl_soroban_game!({{ContractName}}, "world");

#[contractimpl]
impl {{ContractName}} {
    /// Create the first player entity at the world origin.
    ///
    /// Returns the entity ID the caller passes to every subsequent call, and
    /// emits a `(COUGR, set, score1)` event so indexers register the spawn.
    pub fn spawn_player1(env: Env) -> u32 {
        let mut world = {{ContractName}}::load_world(&env);

        let entity = world.spawn_entity();
        world.set_typed(&env, entity, &Score1 { score: DEFAULT_SCORE });

        {{ContractName}}::save_world(&env, &world);
        entity
    }

    /// Create the second player entity.
    ///
    /// Returns the entity ID the caller passes to every subsequent call.
    pub fn spawn_player2(env: Env) -> u32 {
        let mut world = {{ContractName}}::load_world(&env);

        let entity = world.spawn_entity();
        world.set_typed(&env, entity, &Score2 { score: DEFAULT_SCORE });

        {{ContractName}}::save_world(&env, &world);
        entity
    }

    /// Set player 1's score.
    ///
    /// Only player 1's entity is rewritten on `flush()`; player 2's entity
    /// is left untouched, demonstrating dirty-tracking savings.
    ///
    /// Panics if the score exceeds `u32::MAX` or the entity does not exist.
    pub fn set_score1(env: Env, entity_id: u32, score: u32) {
        let mut world = {{ContractName}}::load_world(&env);

        validate_score(&mut world, entity_id, score);

        world.set_typed(&env, entity_id, &Score1 { score });

        {{ContractName}}::save_world(&env, &world);
    }

    /// Set player 2's score.
    ///
    /// Only player 2's entity is rewritten on `flush()`; player 1's entity
    /// is left untouched, demonstrating dirty-tracking savings.
    ///
    /// Panics if the score exceeds `u32::MAX` or the entity does not exist.
    pub fn set_score2(env: Env, entity_id: u32, score: u32) {
        let mut world = {{ContractName}}::load_world(&env);

        validate_score(&mut world, entity_id, score);

        world.set_typed(&env, entity_id, &Score2 { score });

        {{ContractName}}::save_world(&env, &world);
    }

    /// Current score of player 1, or `None` if it never spawned.
    pub fn score1(env: Env, entity_id: u32) -> Option<u32> {
        let world = {{ContractName}}::load_world(&env);
        world.get_typed::<Score1>(&env, entity_id)
    }

    /// Current score of player 2, or `None` if it never spawned.
    pub fn score2(env: Env, entity_id: u32) -> Option<u32> {
        let world = {{ContractName}}::load_world(&env);
        world.get_typed::<Score2>(&env, entity_id)
    }

    /// Total number of entities spawned so far.
    pub fn entity_count(env: Env) -> u32 {
        let world = {{ContractName}}::load_world(&env);
        world.next_entity_id().saturating_sub(1)
    }
}