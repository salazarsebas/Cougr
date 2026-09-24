//! Game rules for {{crate_name}}.
//!
//! Turn validation and win detection live here as pure functions over component
//! values. `lib.rs` owns storage; this module owns the rules, so a rule change
//! never risks touching persistence and can be tested on its own.

use soroban_sdk::Vec;

use crate::components::{
    Board, TurnBasedConfig, TurnState, DRAW, EMPTY, IN_PROGRESS, MARK_O, MARK_X, O_WINS, X_WINS,
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
    config: &TurnBasedConfig,
    position: u32,
    is_player_x: bool,
    is_player_o: bool,
) -> Result<(), MoveError> {
    if turn.status != IN_PROGRESS {
        return Err(MoveError::GameOver);
    }
    if position >= config.board_width * config.board_height {
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
pub fn advance(turn: &TurnState, cells: &Vec<u32>, config: &TurnBasedConfig) -> TurnState {
    let move_count = turn.move_count + 1;
    let status = detect_status(cells, move_count, config);
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
pub fn detect_status(cells: &Vec<u32>, move_count: u32, config: &TurnBasedConfig) -> u32 {
    let w = config.board_width;
    let h = config.board_height;
    let win_len = config.win_length;

    let check_line = |start_x: u32, start_y: u32, dx: i32, dy: i32| -> u32 {
        let mut x = start_x as i32;
        let mut y = start_y as i32;
        let mut count = 0;
        let mut current_mark = EMPTY;

        for _ in 0..win_len {
            if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
                break;
            }
            let idx = (y * w as i32 + x) as u32;
            let mark = cells.get(idx).unwrap_or(EMPTY);
            
            if mark == EMPTY {
                break;
            }
            if current_mark == EMPTY {
                current_mark = mark;
                count = 1;
            } else if current_mark == mark {
                count += 1;
            } else {
                break;
            }
            
            x += dx;
            y += dy;
        }

        if count == win_len {
            if current_mark == MARK_X { X_WINS } else { O_WINS }
        } else {
            EMPTY
        }
    };

    // Horizontal
    for y in 0..h {
        for x in 0..=w.saturating_sub(win_len) {
            let res = check_line(x, y, 1, 0);
            if res != EMPTY { return res; }
        }
    }

    // Vertical
    for x in 0..w {
        for y in 0..=h.saturating_sub(win_len) {
            let res = check_line(x, y, 0, 1);
            if res != EMPTY { return res; }
        }
    }

    // Diagonal (top-left to bottom-right)
    for y in 0..=h.saturating_sub(win_len) {
        for x in 0..=w.saturating_sub(win_len) {
            let res = check_line(x, y, 1, 1);
            if res != EMPTY { return res; }
        }
    }

    // Anti-diagonal (top-right to bottom-left)
    for y in 0..=h.saturating_sub(win_len) {
        for x in win_len - 1..w {
            let res = check_line(x, y, -1, 1);
            if res != EMPTY { return res; }
        }
    }

    if move_count >= w * h {
        DRAW
    } else {
        IN_PROGRESS
    }
}
