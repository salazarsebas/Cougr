//! Cougr components for the Sokoban puzzle.
//!
//! `Position` is re-exported from `cougr_core::component` rather than
//! redefined here. The gameplay-specific components below are small markers
//! so the archetypes really do differ by signature: a wall, a crate and a
//! goal each live in their own archetype, and `OnGoal` moves a crate into a
//! fourth archetype the moment it settles on a goal tile.

use cougr_core::{impl_component, impl_marker_component};
use soroban_sdk::contracttype;

pub use cougr_core::component::Position;

/// Marks the single entity the authorized caller controls with `push`.
pub struct Player;
impl_marker_component!(Player, "player", Table);

/// Marks an impassable tile. A push toward a `Wall` is rejected.
pub struct Wall;
impl_marker_component!(Wall, "wall", Table);

/// Marks a pushable crate. Named `Crate` rather than `Box` so it does not
/// shadow `alloc::boxed::Box`, which this `#![no_std]` crate still reaches
/// through `soroban-sdk`.
pub struct Crate;
impl_marker_component!(Crate, "crate", Table);

/// Marks a goal tile. A level is solved when every `Crate` entity also
/// carries `OnGoal`, i.e. every goal has a crate parked on it.
pub struct Goal;
impl_marker_component!(Goal, "goal", Table);

/// Added to a `Crate` entity while its position coincides with a `Goal`
/// entity's position, removed the moment it is pushed off. The win check is
/// a single query: entities `with_component(Crate)` and
/// `without_component(OnGoal)` that returns none.
pub struct OnGoal;
impl_marker_component!(OnGoal, "ongoal", Sparse);

/// Level-wide counters, attached to the player entity alongside `Position`
/// and `Player` rather than kept in a second storage slot, so "where does
/// the game state live" has one answer: the `ArchetypeWorld`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelMeta {
    pub move_count: u32,
    pub solved: bool,
    pub width: u32,
    pub height: u32,
}
impl_component!(LevelMeta, "lvlmeta", Table, { move_count: u32, solved: bool, width: u32, height: u32 });
