use soroban_sdk::{Env, Vec};

use crate::components::SmallVec4;

// Internal: board cell access
#[inline]
pub(crate) fn board_get(cells: &Vec<i32>, row: u32, col: u32) -> i32 {
    cells.get(row * 8 + col).unwrap_or(0)
}

#[inline]
pub(crate) fn board_set(cells: &mut Vec<i32>, row: u32, col: u32, val: i32) {
    cells.set(row * 8 + col, val);
}
// Internal: piece helpers
/// `true` when `piece` belongs to `player` (1 or 2).
#[inline]
pub(crate) fn owned_by(piece: i32, player: u32) -> bool {
    match player {
        1 => piece == 1 || piece == 2,
        2 => piece == -1 || piece == -2,
        _ => false,
    }
}

/// `true` when `piece` is a king (moves in all four diagonal directions).
#[inline]
pub(crate) fn is_king(piece: i32) -> bool {
    piece == 2 || piece == -2
}

#[inline]
pub(crate) fn forward_delta(player: u32) -> i32 {
    if player == 1 {
        1
    } else {
        -1
    }
}
// System: MoveValidationSystem
pub(crate) fn legal_steps(
    cells: &Vec<i32>,
    row: u32,
    col: u32,
    player: u32,
) -> SmallVec4<(u32, u32)> {
    let piece = board_get(cells, row, col);
    let fd = forward_delta(player);

    let mut deltas: [(i32, i32); 4] = [(0, 0); 4];
    deltas[0] = (fd, 1);
    deltas[1] = (fd, -1);
    let count = if is_king(piece) {
        deltas[2] = (-fd, 1);
        deltas[3] = (-fd, -1);
        4
    } else {
        2
    };

    let mut out = SmallVec4::<(u32, u32)>::new();
    for &(dr, dc) in deltas.iter().take(count) {
        let nr = row as i32 + dr;
        let nc = col as i32 + dc;
        if (0..8).contains(&nr) && (0..8).contains(&nc) {
            let (nr, nc) = (nr as u32, nc as u32);
            if board_get(cells, nr, nc) == 0 {
                out.push((nr, nc));
            }
        }
    }
    out
}

/// Returns all legal capture destinations from `(row, col)` for `player`.
pub(crate) fn legal_captures(
    cells: &Vec<i32>,
    row: u32,
    col: u32,
    player: u32,
) -> SmallVec4<(u32, u32, u32, u32)> {
    let piece = board_get(cells, row, col);
    let fd = forward_delta(player);

    let mut deltas: [(i32, i32); 4] = [(0, 0); 4];
    deltas[0] = (fd, 1);
    deltas[1] = (fd, -1);
    let count = if is_king(piece) {
        deltas[2] = (-fd, 1);
        deltas[3] = (-fd, -1);
        4
    } else {
        2
    };

    let mut out = SmallVec4::<(u32, u32, u32, u32)>::new();
    for &(dr, dc) in deltas.iter().take(count) {
        let mr = row as i32 + dr;
        let mc = col as i32 + dc;
        let lr = row as i32 + 2 * dr;
        let lc = col as i32 + 2 * dc;

        // Bounds-check both squares.
        if !(0..8).contains(&mr) || !(0..8).contains(&mc) {
            continue;
        }
        if !(0..8).contains(&lr) || !(0..8).contains(&lc) {
            continue;
        }

        let (mr, mc) = (mr as u32, mc as u32);
        let (lr, lc) = (lr as u32, lc as u32);

        let mid = board_get(cells, mr, mc);
        let land = board_get(cells, lr, lc);

        // Middle must be an opponent piece; landing must be empty.
        if owned_by(mid, 3 - player) && land == 0 {
            out.push((lr, lc, mr, mc));
        }
    }
    out
}

/// `true` if `player` has at least one capture available anywhere on the board.
pub(crate) fn any_capture_available(cells: &Vec<i32>, player: u32) -> bool {
    for row in 0u32..8 {
        for col in 0u32..8 {
            if owned_by(board_get(cells, row, col), player)
                && !legal_captures(cells, row, col, player).is_empty()
            {
                return true;
            }
        }
    }
    false
}

/// `true` if `player` has at least one legal move (step or capture) anywhere.
pub(crate) fn any_legal_move(cells: &Vec<i32>, player: u32) -> bool {
    for row in 0u32..8 {
        for col in 0u32..8 {
            if owned_by(board_get(cells, row, col), player) {
                if !legal_captures(cells, row, col, player).is_empty() {
                    return true;
                }
                if !legal_steps(cells, row, col, player).is_empty() {
                    return true;
                }
            }
        }
    }
    false
}

// System: PromotionSystem
pub(crate) fn maybe_promote(cells: &mut Vec<i32>, row: u32, col: u32, player: u32) {
    let promotion_row = if player == 1 { 7u32 } else { 0u32 };
    if row == promotion_row {
        let piece = board_get(cells, row, col);
        if !is_king(piece) {
            let king_val = if player == 1 { 2i32 } else { -2i32 };
            board_set(cells, row, col, king_val);
        }
    }
}
// System: EndConditionSystem
/// Returns the winning player number (1 or 2), or 0 if the game continues.
pub(crate) fn check_winner(cells: &Vec<i32>) -> u32 {
    let mut p1_pieces = 0u32;
    let mut p2_pieces = 0u32;
    for row in 0u32..8 {
        for col in 0u32..8 {
            let v = board_get(cells, row, col);
            if v == 1 || v == 2 {
                p1_pieces += 1;
            } else if v == -1 || v == -2 {
                p2_pieces += 1;
            }
        }
    }

    // A player with no pieces loses immediately.
    if p2_pieces == 0 {
        return 1;
    }
    if p1_pieces == 0 {
        return 2;
    }

    if !any_legal_move(cells, 1) {
        return 2;
    }
    if !any_legal_move(cells, 2) {
        return 1;
    }

    0 // Game still active.
}

// Board initialisation helper
pub(crate) fn initial_board(env: &Env) -> Vec<i32> {
    let mut cells: Vec<i32> = Vec::new(env);
    for row in 0u32..8 {
        for col in 0u32..8 {
            let dark = (row + col) % 2 == 1;
            let val: i32 = if dark && row <= 2 {
                1
            } else if dark && row >= 5 {
                -1
            } else {
                0
            };
            cells.push_back(val);
        }
    }
    cells
}
