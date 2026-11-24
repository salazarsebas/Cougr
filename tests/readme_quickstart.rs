#![no_std]

use cougr_core::game::SorobanGame;
use cougr_core::{impl_component_observed, impl_soroban_game};
use soroban_sdk::{contract, contractimpl, contracttype, Env};

// 1. Define a component - one macro call, no boilerplate
#[contracttype]
#[derive(Clone, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
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
// README_QUICKSTART_END

extern crate std;

#[test]
fn readme_quickstart_matches_compiled_contract() {
    const README: &str = include_str!("../README.md");
    const TEST: &str = include_str!("readme_quickstart.rs");
    const HEADING: &str = "## Quick start - your first on-chain game in 30 lines";
    const END_MARKER: &str = "// README_QUICKSTART_END";

    let readme = README.replace("\r\n", "\n");
    let test = TEST.replace("\r\n", "\n");
    let quickstart = readme
        .split_once(HEADING)
        .expect("README quick-start heading exists")
        .1;
    let contract = quickstart
        .split_once("```rust\n")
        .expect("README quick-start Rust block exists")
        .1
        .split_once("\n```")
        .expect("README quick-start Rust block is closed")
        .0;
    let test_contract = test
        .split_once("#![no_std]\n")
        .expect("test contract starts with no_std")
        .1;
    let test_contract = test_contract
        .split_once(END_MARKER)
        .expect("test contract end marker exists")
        .0;
    let test_contract = std::format!("#![no_std]\n{test_contract}");

    assert_eq!(contract.trim_end(), test_contract.trim_end());
}
