#![no_std]

mod components;
mod systems;
#[cfg(test)]
mod tests;

use components::{
    BoardComponent, DropResult, ECSWorldState, GameState, GameStateComponent, PlayerComponent,
    COLS, ROWS, WORLD_KEY,
};
use systems::{
    draw_system, execution_system, gravity_system, turn_system, validation_system,
    win_detection_system,
};

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Vec};

#[contract]
pub struct ConnectFourContract;

#[contractimpl]
impl ConnectFourContract {
    /// Initialize a new game with two players
    pub fn init_game(env: Env, player_one: Address, player_two: Address) -> GameState {
        let mut next_entity_id = 0u32;

        let board = BoardComponent::new(&env, next_entity_id);
        next_entity_id += 1;

        let players = PlayerComponent::new(player_one.clone(), player_two.clone(), next_entity_id);
        next_entity_id += 1;

        let game_state = GameStateComponent::new(next_entity_id);
        next_entity_id += 1;

        let world_state = ECSWorldState {
            board,
            players,
            game_state,
            next_entity_id,
        };

        env.storage().instance().set(&WORLD_KEY, &world_state);
        Self::to_game_state(&env, &world_state)
    }

    /// Drop a piece in a column (0-6)
    /// Gravity automatically places piece in lowest available row
    pub fn drop_piece(env: Env, player: Address, column: u32) -> DropResult {
        let mut world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        // Validation system
        let validation = validation_system(&world_state, &player, column);
        if !validation.0 {
            return DropResult {
                success: false,
                game_state: Self::to_game_state(&env, &world_state),
                message: validation.1,
                row_placed: None,
            };
        }

        // Gravity placement system
        let row_placed = gravity_system(&mut world_state, column);

        // Execution system - place the piece
        execution_system(&mut world_state, row_placed, column);

        // Win detection system
        win_detection_system(&mut world_state);

        // Draw detection system
        draw_system(&mut world_state);

        // Turn system
        turn_system(&mut world_state);

        env.storage().instance().set(&WORLD_KEY, &world_state);

        DropResult {
            success: true,
            game_state: Self::to_game_state(&env, &world_state),
            message: symbol_short!("ok"),
            row_placed: Some(row_placed),
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

    /// Get the board state as a flattened vector
    pub fn get_board(env: Env) -> Vec<u32> {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        world_state.board.cells
    }

    /// Check if a column is valid (within bounds and not full)
    pub fn is_valid_column(env: Env, column: u32) -> bool {
        if column >= COLS {
            return false;
        }

        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        if world_state.game_state.status != 0 {
            return false;
        }

        !world_state.board.is_column_full(&env, column)
    }

    /// Check if the game is finished
    pub fn is_finished(env: Env) -> bool {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        world_state.game_state.status != 0
    }

    /// Get the winner's address if game is over
    pub fn get_winner(env: Env) -> Option<Address> {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        match world_state.game_state.status {
            1 => Some(world_state.players.player_one),
            2 => Some(world_state.players.player_two),
            _ => None,
        }
    }

    /// Reset the game with the same players
    pub fn reset_game(env: Env) -> GameState {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        Self::init_game(
            env,
            world_state.players.player_one,
            world_state.players.player_two,
        )
    }

    fn to_game_state(env: &Env, world: &ECSWorldState) -> GameState {
        let mut board = Vec::new(env);
        for i in 0..(ROWS * COLS) {
            board.push_back(world.board.cells.get(i).unwrap_or(0));
        }

        GameState {
            board,
            rows: ROWS,
            cols: COLS,
            player_one: world.players.player_one.clone(),
            player_two: world.players.player_two.clone(),
            is_player_one_turn: world.game_state.is_player_one_turn,
            move_count: world.game_state.move_count,
            status: world.game_state.status,
            last_move_col: world.game_state.last_move_col,
        }
    }
}
