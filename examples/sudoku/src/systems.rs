use soroban_sdk::{Env, Vec};

use crate::components::{
    BoardComponent, FixedCellsComponent, GameStatusComponent, BOARD_SIZE, EMPTY, STATUS_PLAYING,
    STATUS_SOLVED,
};

// ── Board helpers ─────────────────────────────────────────────────────────────

pub(crate) fn idx(row: u32, col: u32) -> u32 {
    row * BOARD_SIZE + col
}

pub(crate) fn get_cell(cells: &Vec<u32>, row: u32, col: u32) -> u32 {
    cells.get(idx(row, col)).unwrap_or(EMPTY)
}

pub(crate) fn is_fixed(fixed: &Vec<bool>, row: u32, col: u32) -> bool {
    fixed.get(idx(row, col)).unwrap_or(false)
}

// ── Board initialisation ──────────────────────────────────────────────────────

pub(crate) fn init_board(env: &Env, puzzle: Vec<u32>) -> (BoardComponent, FixedCellsComponent) {
    let mut cells = Vec::new(env);
    let mut fixed = Vec::new(env);
    for i in 0..(BOARD_SIZE * BOARD_SIZE) {
        let v = puzzle.get(i).unwrap_or(EMPTY);
        cells.push_back(v);
        fixed.push_back(v != 0);
    }
    (BoardComponent { cells }, FixedCellsComponent { fixed })
}

// ── InputSystem ───────────────────────────────────────────────────────────────

/// Validates cell coordinates, editability, and value range.
pub(crate) fn input_validation_system(
    fixed: &FixedCellsComponent,
    row: u32,
    col: u32,
    value: u32,
) -> Result<(), &'static str> {
    if row >= BOARD_SIZE || col >= BOARD_SIZE {
        return Err("Cell out of bounds");
    }
    if is_fixed(&fixed.fixed, row, col) {
        return Err("Cell is fixed");
    }
    if !(1..=9).contains(&value) {
        return Err("Value out of range");
    }
    Ok(())
}

// ── PlacementValidationSystem ─────────────────────────────────────────────────

/// Returns true if placing `value` at (row, col) violates no row, column, or block constraint.
pub(crate) fn placement_validation_system(
    board: &BoardComponent,
    row: u32,
    col: u32,
    value: u32,
) -> bool {
    check_row(&board.cells, row, col, value)
        && check_col(&board.cells, col, row, value)
        && check_block(&board.cells, row, col, value)
}

pub(crate) fn check_row(cells: &Vec<u32>, row: u32, skip_col: u32, value: u32) -> bool {
    for c in 0..BOARD_SIZE {
        if c == skip_col {
            continue;
        }
        if get_cell(cells, row, c) == value {
            return false;
        }
    }
    true
}

pub(crate) fn check_col(cells: &Vec<u32>, col: u32, skip_row: u32, value: u32) -> bool {
    for r in 0..BOARD_SIZE {
        if r == skip_row {
            continue;
        }
        if get_cell(cells, r, col) == value {
            return false;
        }
    }
    true
}

pub(crate) fn check_block(cells: &Vec<u32>, row: u32, col: u32, value: u32) -> bool {
    let block_row = (row / 3) * 3;
    let block_col = (col / 3) * 3;
    for r in block_row..block_row + 3 {
        for c in block_col..block_col + 3 {
            if r == row && c == col {
                continue;
            }
            if get_cell(cells, r, c) == value {
                return false;
            }
        }
    }
    true
}

// ── BoardUpdateSystem ─────────────────────────────────────────────────────────

pub(crate) fn board_update_system(
    mut board: BoardComponent,
    row: u32,
    col: u32,
    value: u32,
) -> BoardComponent {
    board.cells.set(idx(row, col), value);
    board
}

// ── CompletionSystem ──────────────────────────────────────────────────────────

/// Returns true when all 81 cells are filled and no constraint is violated.
pub(crate) fn completion_system(board: &BoardComponent) -> bool {
    // Fast exit: any empty cell means not solved
    for i in 0..(BOARD_SIZE * BOARD_SIZE) {
        if board.cells.get(i).unwrap_or(EMPTY) == EMPTY {
            return false;
        }
    }
    // All rows and columns must be valid
    for i in 0..BOARD_SIZE {
        if !row_valid(&board.cells, i) || !col_valid(&board.cells, i) {
            return false;
        }
    }
    // All nine 3×3 blocks must be valid
    for br in 0..3 {
        for bc in 0..3 {
            if !block_valid(&board.cells, br * 3, bc * 3) {
                return false;
            }
        }
    }
    true
}

fn row_valid(cells: &Vec<u32>, row: u32) -> bool {
    let mut seen = [false; 10];
    for c in 0..BOARD_SIZE {
        let v = get_cell(cells, row, c) as usize;
        if v == 0 || v > 9 || seen[v] {
            return false;
        }
        seen[v] = true;
    }
    true
}

fn col_valid(cells: &Vec<u32>, col: u32) -> bool {
    let mut seen = [false; 10];
    for r in 0..BOARD_SIZE {
        let v = get_cell(cells, r, col) as usize;
        if v == 0 || v > 9 || seen[v] {
            return false;
        }
        seen[v] = true;
    }
    true
}

fn block_valid(cells: &Vec<u32>, start_row: u32, start_col: u32) -> bool {
    let mut seen = [false; 10];
    for r in start_row..start_row + 3 {
        for c in start_col..start_col + 3 {
            let v = get_cell(cells, r, c) as usize;
            if v == 0 || v > 9 || seen[v] {
                return false;
            }
            seen[v] = true;
        }
    }
    true
}

/// Updates game status based on whether the board is fully and correctly solved.
pub(crate) fn end_condition_system(board: &BoardComponent) -> GameStatusComponent {
    if completion_system(board) {
        GameStatusComponent {
            status: STATUS_SOLVED,
        }
    } else {
        GameStatusComponent {
            status: STATUS_PLAYING,
        }
    }
}
