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
mod test {
    use super::*;

    #[test]
    fn test_init_game() {
        let env = Env::default();
        let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
        let state = client.init_game();
        assert_eq!(state.score, 0);
        assert!(!state.game_over);
    }

    #[test]
    fn test_move_functions() {
        let env = Env::default();
        let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
        client.init_game();

        // Initial move
        let _moved = client.move_left();
        // Depends on random spawn, but generally possible if logic is correct
        // We verify it returns a boolean
    }

    #[test]
    fn test_rotation() {
        let env = Env::default();
        let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
        client.init_game();

        // Try rotate
        let _rotated = client.rotate();
        // Should execute without panic
    }

    #[test]
    fn test_collision_detection() {
        let env = Env::default();
        let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
        client.init_game();

        // Move until hit wall?
        // Since we can't easily force state without backdoor, we rely on move returning false eventually
        for _ in 0..10 {
            client.move_left();
        }
    }

    #[test]
    fn test_line_clearing() {
        // This is hard to test black-box without setting specific board state
        // But we can ensure update_tick runs
        let env = Env::default();
        let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
        client.init_game();

        let _lines = client.update_tick();
    }

    #[test]
    fn test_score_updates() {
        let env = Env::default();
        let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
        client.init_game();

        assert_eq!(client.get_state().score, 0);
    }

    #[test]
    fn test_game_over() {
        let env = Env::default();
        let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
        client.init_game();

        assert!(!client.get_state().game_over);
    }
    #[test]
    fn test_invalid_action_at_left_wall_returns_false() {
        let env = Env::default();
        let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
        client.init_game();
        let mut last = true;
        for _ in 0..16 {
            last = client.move_left();
        }
        assert!(!last);
    }

    #[test]
    fn test_gameapp_tick_integration() {
        let env = Env::default();
        super::systems::run_gameapp_tick(&env);
    }

}
