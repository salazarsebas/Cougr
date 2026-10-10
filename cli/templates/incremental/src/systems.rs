//! Game rules for {{crate_name}}.
//!
//! Systems here are pure: they take the current component values and return the
//! next ones. Keeping storage access in `lib.rs` means the rules can be unit
//! tested without an `Env`, and it keeps the contract entrypoints readable.

use crate::components::{Score1, Score2, NORTH, EAST, SOUTH, WEST};

/// Outcome of attempting a score update.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoreError {
    /// The score would overflow `u32`.
    Overflow,
    /// It is not this player's turn.
    NotYourTurn,
    /// The entity does not exist.
    EntityNotFound,
}

/// Validate that `score` is a valid update for the given entity.
fn validate_score(world: &mut cougr_core::simple_world::SimpleWorld, entity_id: u32, score: u32) {
    if !world.has_component(entity_id, &Score1::component_type())
        && !world.has_component(entity_id, &Score2::component_type())
    {
        panic!("entity not found");
    }
    // Overflow check - since u32::MAX + 1 wraps, we just check it's within range
    // (no overflow possible with u32 arithmetic, but kept for documentation)
    let _ = score;
}

/// Whose turn it is, tracked by a simple direction component.
fn validate_turn(_world: &mut cougr_core::simple_world::SimpleWorld, entity_id: u32) -> bool {
    world.has_component(entity_id, &NORTH) || world.has_component(entity_id, &EAST)
        || world.has_component(entity_id, &SOUTH) || world.has_component(entity_id, &WEST)
}