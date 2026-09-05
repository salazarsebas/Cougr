//! ECS components for {{crate_name}}.
//!
//! `Position` is declared with `impl_component_observed!` so every change emits
//! an indexed `(COUGR, set, position)` Soroban event - off-chain clients can
//! follow movement from the event stream instead of polling. `Moves` uses the
//! plain `impl_component!` macro because the move budget is only ever read back
//! through an explicit contract call.

use cougr_core::{impl_component, impl_component_observed};
use soroban_sdk::contracttype;

/// World-space position of an entity.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl_component_observed!(Position, "position", Table, { x: i32, y: i32 });

/// Remaining move budget and the last direction taken.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Moves {
    pub remaining: u32,
    pub last_direction: u32,
}

impl_component!(Moves, "moves", Sparse, { remaining: u32, last_direction: u32 });

// ─── Directions ───────────────────────────────────────────────────────────────

pub const NORTH: u32 = 0; // +y
pub const EAST: u32 = 1; // +x
pub const SOUTH: u32 = 2; // −y
pub const WEST: u32 = 3; // −x

/// Moves granted to a freshly spawned entity.
pub const DEFAULT_MOVE_BUDGET: u32 = 1_000;
