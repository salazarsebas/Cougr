use soroban_sdk::{symbol_short, Address, Env, Symbol};

use crate::components::{BoardComponent, ECSWorldState, COLS, ROWS};

/// Validation System - checks if move is legal
pub(crate) fn validation_system(
    world: &ECSWorldState,
    player: &Address,
    column: u32,
) -> (bool, Symbol) {
    // Check if game is over
    if world.game_state.status != 0 {
        return (false, symbol_short!("gameover"));
    }

    // Check column bounds
    if column >= COLS {
        return (false, symbol_short!("invalid"));
    }

    // Check if player is registered
    let is_player_one = *player == world.players.player_one;
    let is_player_two = *player == world.players.player_two;

    if !is_player_one && !is_player_two {
        return (false, symbol_short!("notplay"));
    }

    // Check turn order
    if world.game_state.is_player_one_turn && !is_player_one {
        return (false, symbol_short!("notturn"));
    }
    if !world.game_state.is_player_one_turn && !is_player_two {
        return (false, symbol_short!("notturn"));
    }

    // Check if column is full
    if world.board.is_column_full(&Env::default(), column) {
        return (false, symbol_short!("full"));
    }

    (true, symbol_short!("ok"))
}

/// Gravity System - finds lowest empty row in column
pub(crate) fn gravity_system(world: &mut ECSWorldState, column: u32) -> u32 {
    world
        .board
        .get_lowest_empty_row(&Env::default(), column)
        .expect("Column should not be full - validated before calling")
}

/// Execution System - places piece on board
pub(crate) fn execution_system(world: &mut ECSWorldState, row: u32, column: u32) {
    let cell_value = if world.game_state.is_player_one_turn {
        1u32
    } else {
        2u32
    };
    world
        .board
        .set_cell(&Env::default(), row, column, cell_value);
    world.game_state.move_count += 1;
    world.game_state.last_move_col = Some(column);
}

/// Win Detection System - checks for 4-in-a-row patterns
pub(crate) fn win_detection_system(world: &mut ECSWorldState) {
    let board = &world.board;
    let env = Env::default();

    // Check all positions for a winning piece
    for row in 0..ROWS {
        for col in 0..COLS {
            let cell = board.get_cell(&env, row, col);
            if cell == 0 {
                continue;
            }

            // Check horizontal
            if check_horizontal(board, &env, row, col, cell) {
                world.game_state.status = cell;
                return;
            }

            // Check vertical
            if check_vertical(board, &env, row, col, cell) {
                world.game_state.status = cell;
                return;
            }

            // Check diagonal (bottom-left to top-right)
            if check_diagonal_positive(board, &env, row, col, cell) {
                world.game_state.status = cell;
                return;
            }

            // Check diagonal (top-left to bottom-right)
            if check_diagonal_negative(board, &env, row, col, cell) {
                world.game_state.status = cell;
                return;
            }
        }
    }
}

/// Check horizontal connection (4 in a row)
fn check_horizontal(board: &BoardComponent, env: &Env, row: u32, col: u32, cell: u32) -> bool {
    if col + 3 >= COLS {
        return false;
    }

    for i in 0..4 {
        if board.get_cell(env, row, col + i) != cell {
            return false;
        }
    }
    true
}

/// Check vertical connection (4 in a column)
fn check_vertical(board: &BoardComponent, env: &Env, row: u32, col: u32, cell: u32) -> bool {
    if row + 3 >= ROWS {
        return false;
    }

    for i in 0..4 {
        if board.get_cell(env, row + i, col) != cell {
            return false;
        }
    }
    true
}

/// Check diagonal with positive slope (bottom-left to top-right)
fn check_diagonal_positive(
    board: &BoardComponent,
    env: &Env,
    row: u32,
    col: u32,
    cell: u32,
) -> bool {
    if row + 3 >= ROWS || col + 3 >= COLS {
        return false;
    }

    for i in 0..4 {
        if board.get_cell(env, row + i, col + i) != cell {
            return false;
        }
    }
    true
}

/// Check diagonal with negative slope (top-left to bottom-right)
fn check_diagonal_negative(
    board: &BoardComponent,
    env: &Env,
    row: u32,
    col: u32,
    cell: u32,
) -> bool {
    if row < 3 || col + 3 >= COLS {
        return false;
    }

    for i in 0..4 {
        if board.get_cell(env, row - i, col + i) != cell {
            return false;
        }
    }
    true
}

/// Draw System - detects full board with no winner
pub(crate) fn draw_system(world: &mut ECSWorldState) {
    // Maximum moves = ROWS * COLS = 42
    if world.game_state.move_count >= ROWS * COLS && world.game_state.status == 0 {
        world.game_state.status = 3; // Draw
    }
}

/// Turn System - switches turns between players
pub(crate) fn turn_system(world: &mut ECSWorldState) {
    if world.game_state.status == 0 {
        world.game_state.is_player_one_turn = !world.game_state.is_player_one_turn;
    }
}
