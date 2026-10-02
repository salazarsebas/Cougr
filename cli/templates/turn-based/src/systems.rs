//! Game rules for {{crate_name}}.
//!
//! Turn validation and win detection live here as pure functions over component
//! values. `lib.rs` owns storage; this module owns the rules, so a rule change
//! never risks touching persistence and can be tested on its own.

use soroban_sdk::Vec;

use crate::components::{
    Board, TurnState, BOARD_HEIGHT, BOARD_WIDTH, CELL_COUNT, DRAW, EMPTY, IN_PROGRESS,
    MARK_O, MARK_X, O_WINS, WIN_LENGTH, X_WINS,
};

/// Why a proposed move is not legal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveError {
    /// The match already has a winner or ended in a draw.
    GameOver,
    /// The cell index is outside the board.
    OutOfBounds,
    /// The cell already holds a mark.
    Occupied,
    /// It is the other player's turn.
    NotYourTurn,
    /// The caller is neither of the two registered players.
    NotAPlayer,
}

/// Mark the given player places this turn.
pub fn mark_for_turn(turn: &TurnState) -> u32 {
    if turn.is_x_turn {
        MARK_X
    } else {
        MARK_O
    }
}

/// Check that `position` is a legal move for the player to move right now.
pub fn validate_move(
    board: &Board,
    turn: &TurnState,
    position: u32,
    is_player_x: bool,
    is_player_o: bool,
) -> Result<(), MoveError> {
    if turn.status != IN_PROGRESS {
        return Err(MoveError::GameOver);
    }
    if position >= CELL_COUNT {
        return Err(MoveError::OutOfBounds);
    }
    if !is_player_x && !is_player_o {
        return Err(MoveError::NotAPlayer);
    }
    if turn.is_x_turn != is_player_x {
        return Err(MoveError::NotYourTurn);
    }
    if board.cells.get(position).unwrap_or(MARK_X) != EMPTY {
        return Err(MoveError::Occupied);
    }
    Ok(())
}

/// Turn state after a legal move has been written to `cells`.
pub fn advance(turn: &TurnState, cells: &Vec<u32>) -> TurnState {
    let move_count = turn.move_count + 1;
    let status = detect_status(cells, move_count);
    TurnState {
        is_x_turn: if status == IN_PROGRESS {
            !turn.is_x_turn
        } else {
            turn.is_x_turn
        },
        move_count,
        status,
    }
}

/// Win/draw detection over the current cells.
///
/// Returns `IN_PROGRESS`, `X_WINS`, `O_WINS`, or `DRAW`.
pub fn detect_status(cells: &Vec<u32>, move_count: u32) -> u32 {
    for row in 0..BOARD_HEIGHT {
        for col in 0..BOARD_WIDTH {
            let mark = cells.get(row * BOARD_WIDTH + col).unwrap_or(EMPTY);
            if mark == EMPTY { continue; }
            for (dr, dc) in [(0i32, 1i32), (1, 0), (1, 1), (1, -1)] {
                let mut won = true;
                for step in 1..WIN_LENGTH {
                    let r = row as i32 + dr * step as i32;
                    let c = col as i32 + dc * step as i32;
                    if r < 0 || c < 0 || r >= BOARD_HEIGHT as i32 || c >= BOARD_WIDTH as i32
                        || cells.get(r as u32 * BOARD_WIDTH + c as u32).unwrap_or(EMPTY) != mark {
                        won = false;
                        break;
                    }
                }
                if won { return if mark == MARK_X { X_WINS } else { O_WINS }; }
            }
        }
    }
    if move_count >= CELL_COUNT {
        DRAW
    } else {
        IN_PROGRESS
    }
}
