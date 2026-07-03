#![no_std]

mod components;
mod systems;

pub use components::{GameState, Piece, TetrominoShape};
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Vec};

#[contract]
pub struct TetrisContract;

#[contractimpl]
impl TetrisContract {
    /// Initialize the game
    pub fn init_game(env: Env) -> GameState {
        let board = Vec::from_array(&env, [0u32; 20]); // 20 empty rows

        // Spawn initial pieces
        let current_piece = systems::generate_piece(&env);
        let next_piece = systems::generate_piece(&env);

        let state = GameState {
            board,
            current_piece,
            next_piece,
            score: 0,
            level: 1,
            lines_cleared: 0,
            game_over: false,
        };

        systems::save_state(&env, &state);
        state
    }

    /// Move current piece left
    pub fn move_left(env: Env) -> bool {
        let mut state = Self::get_state(env.clone());
        if state.game_over {
            return false;
        }

        if systems::try_move(&env, &mut state, -1, 0, 0) {
            systems::save_state(&env, &state);
            true
        } else {
            false
        }
    }

    /// Move current piece right
    pub fn move_right(env: Env) -> bool {
        let mut state = Self::get_state(env.clone());
        if state.game_over {
            return false;
        }

        if systems::try_move(&env, &mut state, 1, 0, 0) {
            systems::save_state(&env, &state);
            true
        } else {
            false
        }
    }

    /// Move current piece down (soft drop)
    pub fn move_down(env: Env) -> bool {
        let mut state = Self::get_state(env.clone());
        if state.game_over {
            return false;
        }

        if systems::try_move(&env, &mut state, 0, 1, 0) {
            systems::save_state(&env, &state);
            true
        } else {
            // Lock piece if it can't move down
            systems::lock_piece(&env, &mut state);
            systems::save_state(&env, &state);
            false
        }
    }

    /// Rotate piece
    pub fn rotate(env: Env) -> bool {
        let mut state = Self::get_state(env.clone());
        if state.game_over {
            return false;
        }

        // Rotation is +1 to index (clockwise)
        if systems::try_move(&env, &mut state, 0, 0, 1) {
            systems::save_state(&env, &state);
            true
        } else {
            false
        }
    }

    /// Hard drop
    pub fn drop(env: Env) -> u32 {
        let mut state = Self::get_state(env.clone());
        if state.game_over {
            return 0;
        }

        let mut dropped = 0;
        while systems::try_move(&env, &mut state, 0, 1, 0) {
            dropped += 1;
        }

        systems::lock_piece(&env, &mut state);
        systems::save_state(&env, &state);
        dropped
    }

    /// Update tick (gravity)
    pub fn update_tick(env: Env) -> GameState {
        let mut state = Self::get_state(env.clone());
        if state.game_over {
            return state;
        }

        // Try to move down
        if !systems::try_move(&env, &mut state, 0, 1, 0) {
            systems::lock_piece(&env, &mut state);
        }

        systems::save_state(&env, &state);
        state
    }

    /// Get current state
    pub fn get_state(env: Env) -> GameState {
        env.storage()
            .instance()
            .get(&symbol_short!("game"))
            .expect("Game not initialized")
    }
}

// --------------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------------

#[cfg(test)]
#[cfg(test)]
mod test;
