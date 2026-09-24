//! {{crate_name}} - a Cougr game contract generated from the `{{template_id}}` template.
//!
//! Two players alternate placing marks on a 3×3 board until one of them lines
//! up three or the board fills. It is the reference shape for any turn-based
//! game: one match per contract instance, an explicit turn owner, and moves
//! validated before they are written.
//!
//! Demonstrates:
//!   - `impl_rich_component!` - components holding `Address` and `Vec` fields
//!   - `impl_component!` - fixed-size turn state
//!   - `SorobanGame` - standard world load/save
//!   - `impl_soroban_game!` - wires the trait to a `#[contract]` struct

#![no_std]

pub mod components;
pub mod systems;
#[cfg(test)]
mod test;

use components::{Board, ConfigError, Players, TurnBasedConfig, TurnState, GAME_ENTITY, O_WINS, X_WINS};
use systems::{advance, mark_for_turn, validate_move, MoveError};

use cougr_core::game::SorobanGame;
use cougr_core::impl_soroban_game;
use cougr_core::simple_world::SimpleWorld;
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

// ─── API return types ─────────────────────────────────────────────────────────

/// Everything a client needs to render the match.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    pub cells: Vec<u32>,
    pub player_x: Address,
    pub player_o: Address,
    pub is_x_turn: bool,
    pub move_count: u32,
    pub status: u32,
    pub config: TurnBasedConfig,
}

/// Result of a `make_move` call: whether it landed, the resulting state, and a
/// short reason code when it did not.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MoveResult {
    pub success: bool,
    pub game_state: GameState,
    pub message: Symbol,
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
#[derive(Clone)]
pub struct {{ContractName}};

// Generates `load_world` / `save_world` against the "world" instance-storage key.
impl_soroban_game!({{ContractName}}, "world");

#[contractimpl]
impl {{ContractName}} {
    /// Start a new match between `player_x` and `player_o`, discarding any
    /// previous state.
    pub fn init_game(
        env: Env,
        player_x: Address,
        player_o: Address,
        config: Option<TurnBasedConfig>,
    ) -> Result<GameState, ConfigError> {
        let mut world = SimpleWorld::new(&env);
        let entity = world.spawn_entity();
        debug_assert_eq!(entity, GAME_ENTITY);

        let config = config.unwrap_or_else(|| TurnBasedConfig {
            board_width: 3,
            board_height: 3,
            win_length: 3,
            first_player: symbol_short!("x"),
        });
        config.validate()?;

        world.set_rich(&env, GAME_ENTITY, &config);
        world.set_rich_observed(&env, GAME_ENTITY, &Board::new(&env, config.board_width, config.board_height));
        world.set_rich(&env, GAME_ENTITY, &Players { player_x, player_o });
        world.set_typed_observed(&env, GAME_ENTITY, &TurnState::opening(config.first_player == symbol_short!("x")));

        {{ContractName}}::save_world(&env, &world);
        Ok(Self::read_state(&env, &world))
    }

    /// Place the caller's mark at `position` (`0`–`8`).
    ///
    /// Returns `success: false` with a reason code rather than panicking, so a
    /// client can show the rejection without losing the current state.
    pub fn make_move(env: Env, player: Address, position: u32) -> MoveResult {
        player.require_auth();

        let mut world = {{ContractName}}::load_world(&env);
        let config = Self::config(&env, &world);
        let players = Self::players(&env, &world);
        let turn = Self::turn(&env, &world);
        let mut board = Self::board(&env, &world);

        let is_player_x = player == players.player_x;
        let is_player_o = player == players.player_o;

        if let Err(err) = validate_move(&board, &turn, &config, position, is_player_x, is_player_o) {
            return Self::rejected(&env, &world, err);
        }

        board.cells.set(position, mark_for_turn(&turn));
        let turn = advance(&turn, &board.cells, &config);

        world.set_rich_observed(&env, GAME_ENTITY, &board);
        world.set_typed_observed(&env, GAME_ENTITY, &turn);
        {{ContractName}}::save_world(&env, &world);

        MoveResult {
            success: true,
            game_state: Self::read_state(&env, &world),
            message: symbol_short!("ok"),
        }
    }

    /// Current state of the match.
    pub fn get_state(env: Env) -> GameState {
        let world = {{ContractName}}::load_world(&env);
        Self::read_state(&env, &world)
    }

    /// Whether `position` is playable right now, ignoring who is calling.
    pub fn is_valid_move(env: Env, position: u32) -> bool {
        let world = {{ContractName}}::load_world(&env);
        let config = match world.get_rich::<TurnBasedConfig>(&env, GAME_ENTITY) {
            Some(config) => config,
            None => return false,
        };
        let turn = match world.get_typed::<TurnState>(&env, GAME_ENTITY) {
            Some(turn) => turn,
            None => return false,
        };
        let board = match world.get_rich::<Board>(&env, GAME_ENTITY) {
            Some(board) => board,
            None => return false,
        };
        // The turn owner is the only caller who could legally play, so checking
        // against them answers "is this cell playable" without an address.
        validate_move(&board, &turn, &config, position, turn.is_x_turn, !turn.is_x_turn).is_ok()
    }

    /// The winner's address, or `None` while the match is running or drawn.
    pub fn get_winner(env: Env) -> Option<Address> {
        let world = {{ContractName}}::load_world(&env);
        let turn = world.get_typed::<TurnState>(&env, GAME_ENTITY)?;
        let players = world.get_rich::<Players>(&env, GAME_ENTITY)?;
        match turn.status {
            X_WINS => Some(players.player_x),
            O_WINS => Some(players.player_o),
            _ => None,
        }
    }

    /// Clear the board and keep the same two players.
    pub fn reset_game(env: Env) -> GameState {
        let world = {{ContractName}}::load_world(&env);
        let config = Self::config(&env, &world);
        let players = Self::players(&env, &world);
        // We know config is valid because it was validated in init_game
        Self::init_game(env, players.player_x, players.player_o, Some(config)).unwrap()
    }

    // ─── Internal helpers ─────────────────────────────────────────────────────

    fn config(env: &Env, world: &SimpleWorld) -> TurnBasedConfig {
        world
            .get_rich::<TurnBasedConfig>(env, GAME_ENTITY)
            .unwrap_or_else(|| panic!("game not initialised"))
    }

    fn board(env: &Env, world: &SimpleWorld) -> Board {
        world
            .get_rich::<Board>(env, GAME_ENTITY)
            .unwrap_or_else(|| panic!("game not initialised"))
    }

    fn players(env: &Env, world: &SimpleWorld) -> Players {
        world
            .get_rich::<Players>(env, GAME_ENTITY)
            .unwrap_or_else(|| panic!("game not initialised"))
    }

    fn turn(env: &Env, world: &SimpleWorld) -> TurnState {
        world
            .get_typed::<TurnState>(env, GAME_ENTITY)
            .unwrap_or_else(|| panic!("game not initialised"))
    }

    fn read_state(env: &Env, world: &SimpleWorld) -> GameState {
        let config = Self::config(env, world);
        let board = Self::board(env, world);
        let players = Self::players(env, world);
        let turn = Self::turn(env, world);

        GameState {
            cells: board.cells,
            player_x: players.player_x,
            player_o: players.player_o,
            is_x_turn: turn.is_x_turn,
            move_count: turn.move_count,
            status: turn.status,
            config,
        }
    }

    fn rejected(env: &Env, world: &SimpleWorld, err: MoveError) -> MoveResult {
        let message = match err {
            MoveError::GameOver => symbol_short!("gameover"),
            MoveError::OutOfBounds => symbol_short!("bounds"),
            MoveError::Occupied => symbol_short!("occupied"),
            MoveError::NotYourTurn => symbol_short!("notturn"),
            MoveError::NotAPlayer => symbol_short!("notplay"),
        };
        MoveResult {
            success: false,
            game_state: Self::read_state(env, world),
            message,
        }
    }
}
