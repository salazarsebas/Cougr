#![no_std]

mod components;
mod systems;
#[cfg(test)]
mod tests;

use components::{
    BoardComponent, ECSWorldState, GameState, GameStateComponent, MineLayoutComponent,
    RevealResult, VisibleCellState, CELL_MINE, COLS, MINES, ROWS, STATUS_LOST, STATUS_PLAYING,
    WORLD_KEY,
};
use systems::{completion_system, place_mines_deterministic};

#[cfg(test)]
use components::CELL_HIDDEN;

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Vec};

#[contract]
pub struct MinesweeperContract;

#[contractimpl]
impl MinesweeperContract {
    /// Initialize a new game with deterministic mine layout
    pub fn init_game(env: Env) -> GameState {
        let mut next_entity_id = 0u32;

        // Create empty board
        let board = BoardComponent::new(&env, next_entity_id);
        next_entity_id += 1;

        // Create mine layout and place mines deterministically
        let mut mine_layout = MineLayoutComponent::new(&env, next_entity_id);
        place_mines_deterministic(&mut mine_layout, &env);
        next_entity_id += 1;

        // Create game state
        let game_state = GameStateComponent::new(next_entity_id);
        next_entity_id += 1;

        let world_state = ECSWorldState {
            board,
            mine_layout,
            game_state,
            next_entity_id,
        };

        env.storage().instance().set(&WORLD_KEY, &world_state);
        Self::to_game_state(&env, &world_state)
    }

    /// Reveal a cell at (row, col)
    pub fn reveal_cell(env: Env, row: u32, col: u32) -> RevealResult {
        let mut world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        // Validation
        if world_state.game_state.status != STATUS_PLAYING {
            return RevealResult {
                success: false,
                is_mine: false,
                adjacent_mines: 0,
                message: symbol_short!("over"),
            };
        }

        if row >= ROWS || col >= COLS {
            return RevealResult {
                success: false,
                is_mine: false,
                adjacent_mines: 0,
                message: symbol_short!("invalid"),
            };
        }

        // Check if already revealed
        if !world_state.board.is_hidden(&env, row, col) {
            return RevealResult {
                success: false,
                is_mine: false,
                adjacent_mines: 0,
                message: symbol_short!("revealed"),
            };
        }

        // Check if mine
        if world_state.mine_layout.has_mine(&env, row, col) {
            // Game over - loss
            world_state.board.set_cell(&env, row, col, CELL_MINE);
            world_state.game_state.status = STATUS_LOST;

            env.storage().instance().set(&WORLD_KEY, &world_state);

            return RevealResult {
                success: true,
                is_mine: true,
                adjacent_mines: 0,
                message: symbol_short!("boom"),
            };
        }

        // Safe cell - reveal it
        let adjacent_count = world_state.mine_layout.count_adjacent_mines(&env, row, col);
        world_state.board.set_cell(&env, row, col, adjacent_count);
        world_state.game_state.revealed_count += 1;

        // Check for win condition
        completion_system(&mut world_state);

        env.storage().instance().set(&WORLD_KEY, &world_state);

        RevealResult {
            success: true,
            is_mine: false,
            adjacent_mines: adjacent_count,
            message: symbol_short!("ok"),
        }
    }

    /// Get the current game state
    pub fn get_state(env: Env) -> GameState {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        Self::to_game_state(&env, &world_state)
    }

    /// Get visible state of a specific cell
    pub fn get_visible_cell(env: Env, row: u32, col: u32) -> VisibleCellState {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        if row >= ROWS || col >= COLS {
            return VisibleCellState {
                is_revealed: false,
                is_mine: false,
                adjacent_mines: 0,
            };
        }

        let is_revealed = world_state.board.is_revealed(&env, row, col);
        let is_mine = world_state.mine_layout.has_mine(&env, row, col);
        let adjacent_mines = if is_revealed {
            world_state.mine_layout.count_adjacent_mines(&env, row, col)
        } else {
            0
        };

        VisibleCellState {
            is_revealed,
            is_mine: is_revealed && is_mine, // Only show mine if revealed
            adjacent_mines,
        }
    }

    /// Check if the game is finished
    pub fn is_finished(env: Env) -> bool {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        world_state.game_state.status != STATUS_PLAYING
    }

    /// Get the board state (for debugging/viewing)
    pub fn get_board(env: Env) -> Vec<u32> {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        world_state.board.cells
    }

    /// Reset the game
    pub fn reset_game(env: Env) -> GameState {
        Self::init_game(env)
    }

    fn to_game_state(_env: &Env, world: &ECSWorldState) -> GameState {
        let total_safe = (ROWS * COLS) - MINES;
        let safe_cells_remaining = total_safe - world.game_state.revealed_count;

        GameState {
            rows: ROWS,
            cols: COLS,
            total_mines: MINES,
            status: world.game_state.status,
            revealed_count: world.game_state.revealed_count,
            safe_cells_remaining,
        }
    }
}
