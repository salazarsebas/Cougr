use soroban_sdk::Env;

use crate::components::{ECSWorldState, MineLayoutComponent, COLS, MINES, ROWS, STATUS_WON};

/// Place mines in a deterministic pattern (for proof-friendly implementation)
/// Uses a fixed pattern that can be verified
pub(crate) fn place_mines_deterministic(mine_layout: &mut MineLayoutComponent, env: &Env) {
    // Deterministic mine placement - fixed positions for verifiability
    // Using a scattered pattern across the 9x9 grid
    let mine_positions = [
        (1, 1),
        (1, 5),
        (2, 7),
        (3, 3),
        (3, 8),
        (4, 6),
        (5, 2),
        (5, 4),
        (5, 7),
        (7, 0),
        (7, 5),
    ];

    for (row, col) in mine_positions.iter() {
        if *row < ROWS && *col < COLS {
            mine_layout.set_mine(env, *row, *col);
        }
    }
}

/// Completion System - checks if all safe cells are revealed
pub(crate) fn completion_system(world: &mut ECSWorldState) {
    // Total safe cells = total cells - mines
    let total_safe = (ROWS * COLS) - MINES;

    if world.game_state.revealed_count >= total_safe {
        world.game_state.status = STATUS_WON;
    }
}
