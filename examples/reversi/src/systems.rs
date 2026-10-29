use soroban_sdk::{Env, Vec};

use crate::components::{
    BoardComponent, GameStatusComponent, ScoreComponent, TurnComponent, BLACK, BOARD_SIZE, EMPTY,
    STATUS_ACTIVE, STATUS_FINISHED, WHITE,
};

const DIRS: [(i32, i32); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

// ── Board helpers ─────────────────────────────────────────────────────────────

pub(crate) fn idx(row: u32, col: u32) -> u32 {
    row * BOARD_SIZE + col
}

pub(crate) fn get_cell(cells: &Vec<u32>, row: u32, col: u32) -> u32 {
    cells.get(row * BOARD_SIZE + col).unwrap_or(EMPTY)
}

pub(crate) fn opponent_of(player: u32) -> u32 {
    if player == BLACK {
        WHITE
    } else {
        BLACK
    }
}

pub(crate) fn is_board_full(board: &BoardComponent) -> bool {
    for i in 0..(BOARD_SIZE * BOARD_SIZE) {
        if board.cells.get(i).unwrap_or(EMPTY) == EMPTY {
            return false;
        }
    }
    true
}

pub(crate) fn init_board(env: &Env) -> BoardComponent {
    let mut cells = Vec::new(env);
    for _ in 0..(BOARD_SIZE * BOARD_SIZE) {
        cells.push_back(EMPTY);
    }
    cells.set(idx(3, 3), WHITE);
    cells.set(idx(3, 4), BLACK);
    cells.set(idx(4, 3), BLACK);
    cells.set(idx(4, 4), WHITE);
    BoardComponent {
        cells,
        width: BOARD_SIZE,
        height: BOARD_SIZE,
    }
}

// ── ScoringSystem ─────────────────────────────────────────────────────────────

pub(crate) fn scoring_system(board: &BoardComponent) -> ScoreComponent {
    let mut black_count = 0u32;
    let mut white_count = 0u32;
    for i in 0..(BOARD_SIZE * BOARD_SIZE) {
        let cell = board.cells.get(i).unwrap_or(EMPTY);
        if cell == BLACK {
            black_count += 1;
        } else if cell == WHITE {
            white_count += 1;
        }
    }
    ScoreComponent {
        black_count,
        white_count,
    }
}

// ── MoveValidationSystem ──────────────────────────────────────────────────────

pub(crate) fn move_validation_system(
    board: &BoardComponent,
    row: u32,
    col: u32,
    player: u32,
) -> bool {
    if row >= BOARD_SIZE || col >= BOARD_SIZE {
        return false;
    }
    if get_cell(&board.cells, row, col) != EMPTY {
        return false;
    }
    let opp = opponent_of(player);
    for (dr, dc) in DIRS {
        if flips_in_dir(&board.cells, row, col, player, opp, dr, dc) > 0 {
            return true;
        }
    }
    false
}

pub(crate) fn flips_in_dir(
    cells: &Vec<u32>,
    row: u32,
    col: u32,
    player: u32,
    opp: u32,
    dr: i32,
    dc: i32,
) -> u32 {
    let mut r = row as i32 + dr;
    let mut c = col as i32 + dc;
    let mut count = 0u32;
    while r >= 0 && r < BOARD_SIZE as i32 && c >= 0 && c < BOARD_SIZE as i32 {
        let cell = get_cell(cells, r as u32, c as u32);
        if cell == opp {
            count += 1;
            r += dr;
            c += dc;
        } else if cell == player {
            return count;
        } else {
            return 0;
        }
    }
    0
}

pub(crate) fn has_legal_moves(board: &BoardComponent, player: u32) -> bool {
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            if move_validation_system(board, row, col, player) {
                return true;
            }
        }
    }
    false
}

// ── FlipResolutionSystem ──────────────────────────────────────────────────────

pub(crate) fn flip_resolution_system(
    mut board: BoardComponent,
    row: u32,
    col: u32,
    player: u32,
) -> BoardComponent {
    let opp = opponent_of(player);
    board.cells.set(idx(row, col), player);
    for (dr, dc) in DIRS {
        let n = flips_in_dir(&board.cells, row, col, player, opp, dr, dc);
        if n > 0 {
            let mut r = row as i32 + dr;
            let mut c = col as i32 + dc;
            for _ in 0..n {
                board.cells.set(idx(r as u32, c as u32), player);
                r += dr;
                c += dc;
            }
        }
    }
    board
}

// ── TurnSystem ────────────────────────────────────────────────────────────────

pub(crate) fn turn_system(board: &BoardComponent, current: u32, opponent: u32) -> TurnComponent {
    if has_legal_moves(board, opponent) {
        TurnComponent {
            current_player: opponent,
            pass_count: 0,
        }
    } else {
        pass_system(board, current)
    }
}

// ── PassSystem ────────────────────────────────────────────────────────────────

/// Handles automatic pass when the next player has no legal move.
pub(crate) fn pass_system(board: &BoardComponent, current: u32) -> TurnComponent {
    if has_legal_moves(board, current) {
        // Current player continues; opponent is auto-passed
        TurnComponent {
            current_player: current,
            pass_count: 1,
        }
    } else {
        // Both players locked; game will end
        TurnComponent {
            current_player: current,
            pass_count: 2,
        }
    }
}

// ── EndConditionSystem ────────────────────────────────────────────────────────

pub(crate) fn end_condition_system(
    board: &BoardComponent,
    turn: &TurnComponent,
) -> GameStatusComponent {
    if turn.pass_count >= 2 || is_board_full(board) {
        GameStatusComponent {
            status: STATUS_FINISHED,
        }
    } else {
        GameStatusComponent {
            status: STATUS_ACTIVE,
        }
    }
}
