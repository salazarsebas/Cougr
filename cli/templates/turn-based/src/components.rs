//! ECS components for {{crate_name}}.
//!
//! `Board` and `Players` hold `Vec` and `Address` fields, which need an XDR
//! codec - that is exactly what `impl_rich_component!` provides, replacing the
//! hand-written serialize/deserialize pairs an older Soroban contract would
//! carry. `TurnState` is all fixed-size scalars, so the cheaper
//! `impl_component!` is enough.

use cougr_core::{impl_component, impl_component_observed, impl_rich_component};
use soroban_sdk::{contracterror, contracttype, Address, Env, Vec};

/// The single entity every game's state hangs off.
///
/// This contract hosts exactly one match at a time, so a fixed entity ID is
/// simpler than a dynamic entity population.
pub const GAME_ENTITY: u32 = 1;

/// Number of cells on the board.
// ─── Config ───────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TurnBasedConfig {
    pub board_width: u32,
    pub board_height: u32,
    pub win_length: u32,
    pub first_player: soroban_sdk::Symbol,
}

impl TurnBasedConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.board_width < 3 || self.board_width > 8 {
            return Err(ConfigError::InvalidWidth);
        }
        if self.board_height < 3 || self.board_height > 8 {
            return Err(ConfigError::InvalidHeight);
        }
        if self.win_length < 3 || self.win_length > core::cmp::min(self.board_width, self.board_height) {
            return Err(ConfigError::InvalidWinLength);
        }
        if self.first_player != soroban_sdk::symbol_short!("x") && self.first_player != soroban_sdk::symbol_short!("o") {
            return Err(ConfigError::InvalidFirstPlayer);
        }
        Ok(())
    }
}

impl_rich_component!(TurnBasedConfig, "config");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ConfigError {
    InvalidWidth = 1,
    InvalidHeight = 2,
    InvalidWinLength = 3,
    InvalidFirstPlayer = 4,
}

// ─── Cell markers ─────────────────────────────────────────────────────────────

pub const EMPTY: u32 = 0;
pub const MARK_X: u32 = 1;
pub const MARK_O: u32 = 2;

// ─── Game status ──────────────────────────────────────────────────────────────

pub const IN_PROGRESS: u32 = 0;
pub const X_WINS: u32 = 1;
pub const O_WINS: u32 = 2;
pub const DRAW: u32 = 3;

// ─── Components ───────────────────────────────────────────────────────────────

/// Board cells, indexed left-to-right then top-to-bottom.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Board {
    pub cells: Vec<u32>,
}

impl_rich_component!(Board, "board");

impl Board {
    /// An empty board.
    pub fn new(env: &Env, width: u32, height: u32) -> Self {
        let mut cells = Vec::new(env);
        for _ in 0..(width * height) {
            cells.push_back(EMPTY);
        }
        Self { cells }
    }
}

/// Both players' addresses.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Players {
    pub player_x: Address,
    pub player_o: Address,
}

impl_rich_component!(Players, "players");

/// Whose turn it is and whether the match has ended.
#[contracttype]
#[derive(Clone, Debug)]
pub struct TurnState {
    pub is_x_turn: bool,
    pub move_count: u32,
    /// One of `IN_PROGRESS`, `X_WINS`, `O_WINS`, `DRAW`.
    pub status: u32,
}

impl_component_observed!(TurnState, "turnst", Table, {
    is_x_turn: bool,
    move_count: u32,
    status: u32
});

impl TurnState {
    /// Opening turn state.
    pub fn opening(is_x_turn: bool) -> Self {
        Self {
            is_x_turn,
            move_count: 0,
            status: IN_PROGRESS,
        }
    }
}
