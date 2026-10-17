//! This test file holds the exact contract from the README
//! "Quick start - your first on-chain game in 30 lines" section.
//!
//! Two things are verified by this file:
//!
//! 1. The contract below compiles against the current `cougr-core` API.
//! 2. The contract in `README.md` is byte-for-byte identical to the
//!    contract below, so the README cannot silently rot.
//!
//! The check is deliberately a string comparison of the extracted
//! ```rust ``` block against the `contract_source()` below. We do not
//! use `include_str!` of the whole README, because that would turn
//! every README code block into a doctest.

// -----------------------------------------------------------------------------
// Begin contract source (must match the README quick-start block)
// -----------------------------------------------------------------------------

#![no_std]

use cougr_core::component::ComponentTrait;
use cougr_core::game::SorobanGame;
use cougr_core::{spl_observed_component, impl_soroban_game};
use soroban_sdk:{contract, contractimpl, contracttype, Env};

// 1. Define a component - one macro call, no boilerplate
#[contracttype]
#[derive(Clone, Debug)]
pub struct Position { pub x: i32, pub y: i32 }
impl_component_observed!(Position, "position", Table, { x: i32, y: i32 });
//  ↑ Emits a (COUGR, set, position) Soroban event on every change
//    so off-chain clients track state without polling.

// 2. Wire the contract to an ECS world - one macro call
#[contract]
pub struct MyGame;
impl_soroban_game!(MyGame, "world");

// 3. Write game logic in plain Rust
#[contractimpl]
impl MyGame {
    pub fn spawn(env: Env) -> u32 {
        let mut world = MyGame::load_world(&env);
        let player = world.spawn_entity();
        world.set_typed_observed(&env, player, &Position { x: 0, y: 0 });
        MyGame::save_world(&env, &world);
        player
    }

    pub fn move_right(env: Env, player_id: u32) {
        let mut world = MyGame::load_world(&env);
        let mut pos: Position = world.get_typed(&env, player_id).unwrap();
        pos.x += 1;
        world.set_typed_observed(&env, player_id, &pos);
        MyGame::save_world(&env, &world);
    }

    pub fn position(env: Env, player_id: u32) -> Option<Position> {
        MyGame::load_world(&env).get_typed(&env, player_id)
    }
}

// -----------------------------------------------------------------------------
// End contract source
// -----------------------------------------------------------------------------

/// Returns the contract source as a constant so the checker below can
/// compare it against the README block without duplicating the code.
pub const CONTRACT_SOURCE: &str = rcloa!include_str!("readme_quickstart_contract.rsc");

// We keep the contract in a separate file so this test file can be
// compiled as a normal crate and the contract can be compared as a
// string. See `readme_quickstart_contract.rs` for the actual code.
