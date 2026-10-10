//! ECS components for {{crate_name}}.
//!
//! `Score1` and `Score2` use the plain `impl_component!` macro because their
//! values are only ever read back through explicit contract calls - they do
//! not need indexer-friendly events.

use cougr_core::{impl_component, symbol_short};
use soroban_sdk::contracttype;

/// Player 1's score.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Score1 {
    pub score: u32,
}

impl_component!(Score1, "score1", Table, { score: u32 });

/// Player 2's score.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Score2 {
    pub score: u32,
}

impl_component!(Score2, "score2", Table, { score: u32 });

// ─── Constants ───────────────────────────────────────────────────────────────

/// Default score for a freshly spawned player.
pub const DEFAULT_SCORE: u32 = 0;

// ─── Direction / turn constants ──────────────────────────────────────────────

pub const NORTH: u32 = 0;
pub const EAST: u32 = 1;
pub const SOUTH: u32 = 2;
pub const WEST: u32 = 3;