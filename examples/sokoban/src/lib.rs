// sokoban - canonical Cougr example demonstrating the ArchetypeWorld ECS
// backend and component-signature queries.
//
// A level is a grid of walls, crates and goals. The caller pushes their
// player one tile at a time; pushing into a crate shoves it ahead if the
// tile beyond is free. The level is solved once every crate shares its
// position with a goal, checked with a single `without_component`
// ArchetypeQueryBuilder query rather than scanning the whole grid.
//
// Demonstrates:
//   - ArchetypeWorld - entities grouped into archetypes by component
//     signature (Player, Wall, Crate, Goal, OnGoal each live in their own)
//   - ArchetypeQueryBuilder - typed queries over those archetypes,
//     including `without_component` for win detection
//   - impl_marker_component! / impl_component! - the two Cougr component
//     shapes this example needs

#![no_std]

use cougr_core::ArchetypeWorld;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol, Vec};

mod components;
mod systems;

use components::{Crate, Goal, LevelMeta, Player, Position, Wall};

// ─── Tile codes ─────────────────────────────────────────────────────────────

pub const TILE_EMPTY: u32 = 0;
pub const TILE_WALL: u32 = 1;
pub const TILE_CRATE: u32 = 2;
pub const TILE_GOAL: u32 = 3;
pub const TILE_PLAYER: u32 = 4;

// ─── Direction constants ────────────────────────────────────────────────────

pub const NORTH: u32 = 0; // +y
pub const EAST: u32 = 1; // +x
pub const SOUTH: u32 = 2; // -y
pub const WEST: u32 = 3; // -x

// ─── Storage keys ───────────────────────────────────────────────────────────

const WORLD_KEY: Symbol = symbol_short!("world");
const PLAYER_ADDR_KEY: Symbol = symbol_short!("plyraddr");

const MAX_DIM: u32 = 20;

// ─── Contract ───────────────────────────────────────────────────────────────

#[contract]
pub struct SokobanContract;

#[contractimpl]
impl SokobanContract {
    /// Initialize the (single) level this contract instance plays.
    ///
    /// `tiles` is a row-major grid of length `width * height` using the
    /// `TILE_*` codes above: exactly one player tile, and an equal, positive
    /// number of crate and goal tiles. If a level is already stored, only
    /// the same `player` may replace it; a different address is rejected.
    /// Requires `player`'s authorization.
    pub fn init_level(env: Env, player: Address, width: u32, height: u32, tiles: Vec<u32>) {
        player.require_auth();

        let existing_player: Option<Address> = env.storage().instance().get(&PLAYER_ADDR_KEY);
        if let Some(existing) = existing_player {
            if existing != player {
                panic!("level already initialized by another player");
            }
        }

        if width == 0 || height == 0 {
            panic!("level has zero dimension");
        }
        if width > MAX_DIM || height > MAX_DIM {
            panic!("level exceeds maximum dimension");
        }
        if tiles.len() != width * height {
            panic!("tile count does not match dimensions");
        }

        let mut player_count = 0u32;
        let mut crate_count = 0u32;
        let mut goal_count = 0u32;
        for i in 0..tiles.len() {
            match tiles.get(i).unwrap() {
                TILE_EMPTY | TILE_WALL => {}
                TILE_CRATE => crate_count += 1,
                TILE_GOAL => goal_count += 1,
                TILE_PLAYER => player_count += 1,
                _ => panic!("unknown tile code"),
            }
        }
        if player_count != 1 {
            panic!("level must have exactly one player tile");
        }
        if crate_count == 0 || goal_count == 0 {
            panic!("level must have at least one crate and one goal");
        }
        if crate_count != goal_count {
            panic!("crate count must match goal count");
        }

        env.storage().instance().set(&PLAYER_ADDR_KEY, &player);

        let mut world = ArchetypeWorld::new(&env);
        let player_entity = world.spawn_entity();

        for i in 0..tiles.len() {
            let code = tiles.get(i).unwrap();
            if code == TILE_EMPTY {
                continue;
            }

            let x = (i % width) as i32;
            let y = (i / width) as i32;
            let pos = Position::new(x, y);

            match code {
                TILE_WALL => {
                    let entity = world.spawn_entity();
                    world.set_typed(&env, entity, &pos);
                    world.set_typed(&env, entity, &Wall);
                }
                TILE_CRATE => {
                    let entity = world.spawn_entity();
                    world.set_typed(&env, entity, &pos);
                    world.set_typed(&env, entity, &Crate);
                }
                TILE_GOAL => {
                    let entity = world.spawn_entity();
                    world.set_typed(&env, entity, &pos);
                    world.set_typed(&env, entity, &Goal);
                }
                TILE_PLAYER => {
                    world.set_typed(&env, player_entity, &pos);
                    world.set_typed(&env, player_entity, &Player);
                }
                _ => unreachable!(),
            }
        }

        let solved = systems::is_solved(&world, &env);
        world.set_typed(
            &env,
            player_entity,
            &LevelMeta {
                move_count: 0,
                solved,
                width,
                height,
            },
        );

        Self::save_world(&env, &world);
    }

    /// Move the player one tile in `direction` (see the `NORTH`/`EAST`/
    /// `SOUTH`/`WEST` constants), pushing a crate ahead of it if present.
    ///
    /// Panics if the level is not initialized, `player` is not the address
    /// that called `init_level`, the level is already solved, or the move
    /// is blocked by a wall or crate. Requires `player`'s authorization.
    pub fn push(env: Env, player: Address, direction: u32) {
        player.require_auth();

        let registered: Address = env
            .storage()
            .instance()
            .get(&PLAYER_ADDR_KEY)
            .unwrap_or_else(|| panic!("level not initialized"));

        if registered != player {
            panic!("caller is not the registered player");
        }

        let (dx, dy) = Self::direction_delta(direction);

        let mut world = Self::load_world(&env);
        systems::try_push(&mut world, &env, dx, dy);
        Self::save_world(&env, &world);
    }

    /// Return the current level counters (move count, solved, dimensions).
    pub fn level_meta(env: Env) -> LevelMeta {
        let world = Self::load_world(&env);
        world
            .get_typed::<LevelMeta>(&env, systems::PLAYER_ID)
            .unwrap_or_else(|| panic!("level not initialized"))
    }

    /// Return the player's current position.
    pub fn player_position(env: Env) -> Position {
        let world = Self::load_world(&env);
        world
            .get_typed::<Position>(&env, systems::PLAYER_ID)
            .unwrap_or_else(|| panic!("level not initialized"))
    }

    fn direction_delta(direction: u32) -> (i32, i32) {
        match direction {
            NORTH => (0, 1),
            EAST => (1, 0),
            SOUTH => (0, -1),
            WEST => (-1, 0),
            _ => panic!("invalid direction"),
        }
    }

    fn load_world(env: &Env) -> ArchetypeWorld {
        env.storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| ArchetypeWorld::new(env))
    }

    fn save_world(env: &Env, world: &ArchetypeWorld) {
        env.storage().instance().set(&WORLD_KEY, world);
    }
}

#[cfg(test)]
mod test;
