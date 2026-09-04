// tic_tac_toe - modernized using:
//   - impl_rich_component! for Address-bearing components (no manual serialize)
//   - impl_component!      for fixed-size turn state (zero-cost, typed)
//   - SorobanGame          for clean load/save without repeating the key
//
// Before (v1.0): ~400 lines, 3× manual serialize/deserialize implementations
// After  (v1.1): ~250 lines, 0 manual serialization code

#![no_std]

mod components;
mod systems;
#[cfg(test)]
mod test;

use components::{Board, Players, TurnState, GAME_ENTITY};
use systems::detect_winner;

use cougr_core::game::SorobanGame;
use cougr_core::impl_soroban_game;
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

// ─── API return types ─────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    pub cells: Vec<u32>,
    pub player_x: Address,
    pub player_o: Address,
    pub is_x_turn: bool,
    pub move_count: u32,
    pub status: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MoveResult {
    pub success: bool,
    pub game_state: GameState,
    pub message: Symbol,
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct TicTacToeContract;

impl_soroban_game!(TicTacToeContract, "ttt_world");

#[contractimpl]
impl TicTacToeContract {
    /// Initialise a new game, discarding any previous state.
    pub fn init_game(env: Env, player_x: Address, player_o: Address) -> GameState {
        let mut world = cougr_core::simple_world::SimpleWorld::new(&env);
        let entity = world.spawn_entity(); // always 1
        debug_assert_eq!(entity, GAME_ENTITY);

        world.set_rich(&env, GAME_ENTITY, &Board::new(&env));
        world.set_rich(&env, GAME_ENTITY, &Players { player_x, player_o });
        world.set_typed(
            &env,
            GAME_ENTITY,
            &TurnState {
                is_x_turn: true,
                move_count: 0,
                status: 0,
            },
        );

        TicTacToeContract::save_world(&env, &world);
        Self::read_game_state(&env, &world)
    }

    /// Attempt to place the caller's mark at `position` (0–8).
    pub fn make_move(env: Env, player: Address, position: u32) -> MoveResult {
        let mut world = TicTacToeContract::load_world(&env);

        let players: Players = world
            .get_rich::<Players>(&env, GAME_ENTITY)
            .unwrap_or_else(|| panic!("game not initialised"));
        let turn: TurnState = world
            .get_typed::<TurnState>(&env, GAME_ENTITY)
            .unwrap_or_else(|| panic!("game not initialised"));
        let mut board: Board = world
            .get_rich::<Board>(&env, GAME_ENTITY)
            .unwrap_or_else(|| panic!("game not initialised"));

        // Validate
        if turn.status != 0 {
            return Self::failure(&env, &world, symbol_short!("gameover"));
        }
        if position >= 9 {
            return Self::failure(&env, &world, symbol_short!("invalid"));
        }
        let is_player_x = player == players.player_x;
        let is_player_o = player == players.player_o;
        if !is_player_x && !is_player_o {
            return Self::failure(&env, &world, symbol_short!("notplay"));
        }
        if turn.is_x_turn && !is_player_x {
            return Self::failure(&env, &world, symbol_short!("notturn"));
        }
        if !turn.is_x_turn && !is_player_o {
            return Self::failure(&env, &world, symbol_short!("notturn"));
        }
        if board.cells.get(position).unwrap_or(1) != 0 {
            return Self::failure(&env, &world, symbol_short!("occupied"));
        }

        // Execute
        let mark = if turn.is_x_turn { 1u32 } else { 2u32 };
        board.cells.set(position, mark);

        let new_move_count = turn.move_count + 1;
        let new_status = detect_winner(&board.cells, new_move_count);
        let new_is_x_turn = if new_status == 0 {
            !turn.is_x_turn
        } else {
            turn.is_x_turn
        };

        world.set_rich(&env, GAME_ENTITY, &board);
        world.set_typed(
            &env,
            GAME_ENTITY,
            &TurnState {
                is_x_turn: new_is_x_turn,
                move_count: new_move_count,
                status: new_status,
            },
        );

        TicTacToeContract::save_world(&env, &world);

        MoveResult {
            success: true,
            game_state: Self::read_game_state(&env, &world),
            message: symbol_short!("ok"),
        }
    }

    /// Return the current game state.
    pub fn get_state(env: Env) -> GameState {
        let world = TicTacToeContract::load_world(&env);
        Self::read_game_state(&env, &world)
    }

    /// Return true if `position` is a legal move right now.
    pub fn is_valid_move(env: Env, position: u32) -> bool {
        if position >= 9 {
            return false;
        }
        let world = TicTacToeContract::load_world(&env);
        let turn: TurnState = match world.get_typed::<TurnState>(&env, GAME_ENTITY) {
            Some(t) => t,
            None => return false,
        };
        if turn.status != 0 {
            return false;
        }
        let board: Board = match world.get_rich::<Board>(&env, GAME_ENTITY) {
            Some(b) => b,
            None => return false,
        };
        board.cells.get(position).unwrap_or(1) == 0
    }

    /// Return the winner's address, or `None` if the game is ongoing or drawn.
    pub fn get_winner(env: Env) -> Option<Address> {
        let world = TicTacToeContract::load_world(&env);
        let turn: TurnState = world.get_typed::<TurnState>(&env, GAME_ENTITY)?;
        let players: Players = world.get_rich::<Players>(&env, GAME_ENTITY)?;
        match turn.status {
            1 => Some(players.player_x),
            2 => Some(players.player_o),
            _ => None,
        }
    }

    /// Reset the board but keep the same players.
    pub fn reset_game(env: Env) -> GameState {
        let world = TicTacToeContract::load_world(&env);
        let players: Players = world
            .get_rich::<Players>(&env, GAME_ENTITY)
            .unwrap_or_else(|| panic!("game not initialised"));
        Self::init_game(env, players.player_x, players.player_o)
    }

    // ─── Internal helpers ─────────────────────────────────────────────────────

    fn read_game_state(env: &Env, world: &cougr_core::simple_world::SimpleWorld) -> GameState {
        let board: Board = world
            .get_rich::<Board>(env, GAME_ENTITY)
            .unwrap_or_else(|| Board::new(env));
        let players: Players = world
            .get_rich::<Players>(env, GAME_ENTITY)
            .unwrap_or_else(|| panic!("players not found"));
        let turn: TurnState = world
            .get_typed::<TurnState>(env, GAME_ENTITY)
            .unwrap_or(TurnState {
                is_x_turn: true,
                move_count: 0,
                status: 0,
            });

        GameState {
            cells: board.cells,
            player_x: players.player_x,
            player_o: players.player_o,
            is_x_turn: turn.is_x_turn,
            move_count: turn.move_count,
            status: turn.status,
        }
    }

    fn failure(
        env: &Env,
        world: &cougr_core::simple_world::SimpleWorld,
        msg: Symbol,
    ) -> MoveResult {
        MoveResult {
            success: false,
            game_state: Self::read_game_state(env, world),
            message: msg,
        }
    }
}
