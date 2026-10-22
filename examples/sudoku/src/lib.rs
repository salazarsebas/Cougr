#![no_std]

mod components;
mod systems;
#[cfg(test)]
mod test;

use components::{
    CellState, ECSWorldState, GameState, GameStatusComponent, MoveCountComponent, BOARD_SIZE,
    STATUS_PLAYING, WORLD_KEY,
};
use systems::{
    board_update_system, end_condition_system, get_cell, init_board, input_validation_system,
    is_fixed, placement_validation_system,
};

use soroban_sdk::{contract, contractimpl, Env, Vec};

#[contract]
pub struct SudokuContract;

#[contractimpl]
impl SudokuContract {
    pub fn init_game(env: Env, puzzle: Vec<u32>) {
        if env
            .storage()
            .instance()
            .get::<_, ECSWorldState>(&WORLD_KEY)
            .is_some()
        {
            panic!("Game already initialized");
        }
        if puzzle.len() != BOARD_SIZE * BOARD_SIZE {
            panic!("Puzzle must have exactly 81 cells");
        }
        for i in 0..(BOARD_SIZE * BOARD_SIZE) {
            if puzzle.get(i).unwrap_or(0) > 9 {
                panic!("Puzzle contains invalid value");
            }
        }
        let (board, fixed) = init_board(&env, puzzle);
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                let val = systems::get_cell(&board.cells, row, col);
                if val != 0
                    && (!systems::check_row(&board.cells, row, col, val)
                        || !systems::check_col(&board.cells, col, row, val)
                        || !systems::check_block(&board.cells, row, col, val))
                {
                    panic!("Puzzle givens violate Sudoku constraints");
                }
            }
        }
        let world = ECSWorldState {
            board,
            fixed,
            status: GameStatusComponent {
                status: STATUS_PLAYING,
            },
            moves: MoveCountComponent { moves: 0 },
        };
        env.storage().instance().set(&WORLD_KEY, &world);
    }

    pub fn submit_value(env: Env, row: u32, col: u32, value: u32) {
        let mut world = Self::load_world(&env);
        if world.status.status != STATUS_PLAYING {
            panic!("Puzzle already solved");
        }
        if let Err(msg) = input_validation_system(&world.fixed, row, col, value) {
            panic!("{}", msg);
        }
        if !placement_validation_system(&world.board, row, col, value) {
            panic!("Placement violates Sudoku constraints");
        }
        world.board = board_update_system(world.board, row, col, value);
        world.moves.moves += 1;
        world.status = end_condition_system(&world.board);
        env.storage().instance().set(&WORLD_KEY, &world);
    }

    pub fn get_state(env: Env) -> GameState {
        let world = Self::load_world(&env);
        GameState {
            status: world.status.status,
            moves: world.moves.moves,
        }
    }

    pub fn get_cell(env: Env, row: u32, col: u32) -> CellState {
        if row >= BOARD_SIZE || col >= BOARD_SIZE {
            panic!("Cell out of bounds");
        }
        let world = Self::load_world(&env);
        CellState {
            value: get_cell(&world.board.cells, row, col),
            fixed: is_fixed(&world.fixed.fixed, row, col),
        }
    }

    pub fn is_solved(env: Env) -> bool {
        let world = Self::load_world(&env);
        world.status.status == components::STATUS_SOLVED
    }

    fn load_world(env: &Env) -> ECSWorldState {
        env.storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"))
    }
}
