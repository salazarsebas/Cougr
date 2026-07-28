//! Game rules for {{crate_name}}.
//!
//! Systems here are pure: they take the current component values and return the
//! next ones. Keeping storage access in `lib.rs` means the rules can be unit
//! tested without an `Env`, and it keeps the contract entrypoints readable.

use crate::components::{Moves, Position, EAST, NORTH, SOUTH, WEST};

/// Outcome of attempting a single step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveError {
    /// The direction is outside the `NORTH..=WEST` range.
    InvalidDirection,
    /// The entity has spent its whole move budget.
    NoMovesRemaining,
}

/// Apply one step in `direction`, returning the updated components.
///
/// The move budget is decremented on success; callers persist the result.
pub fn step(
    position: &Position,
    moves: &Moves,
    direction: u32,
) -> Result<(Position, Moves), MoveError> {
    if direction > WEST {
        return Err(MoveError::InvalidDirection);
    }
    if moves.remaining == 0 {
        return Err(MoveError::NoMovesRemaining);
    }

    let mut next = position.clone();
    match direction {
        NORTH => next.y += 1,
        EAST => next.x += 1,
        SOUTH => next.y -= 1,
        WEST => next.x -= 1,
        _ => return Err(MoveError::InvalidDirection),
    }

    Ok((
        next,
        Moves {
            remaining: moves.remaining - 1,
            last_direction: direction,
        },
    ))
}
