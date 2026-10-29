#![allow(dead_code)]

use cougr_core::component::ComponentTrait;
use soroban_sdk::{contracttype, symbol_short, Address, Bytes, Env, Symbol, Vec};

// ── Constants ────────────────────────────────────────────────────────────────

pub(crate) const BOARD_SIZE: u32 = 8;
pub(crate) const EMPTY: u32 = 0;
pub(crate) const BLACK: u32 = 1;
pub(crate) const WHITE: u32 = 2;
pub(crate) const STATUS_ACTIVE: u32 = 0;
pub(crate) const STATUS_FINISHED: u32 = 1;
pub(crate) const WINNER_NONE: u32 = 0;
pub(crate) const WINNER_DRAW: u32 = 3;
pub(crate) const WORLD_KEY: Symbol = symbol_short!("WORLD");

// ── Components ───────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct BoardComponent {
    pub cells: Vec<u32>,
    pub width: u32,
    pub height: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct TurnComponent {
    pub current_player: u32,
    /// Recomputed each turn: 0=normal, 1=opponent currently has no legal move, 2=neither player has a legal move (triggers game end).
    pub pass_count: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct GameStatusComponent {
    pub status: u32, // 0 = active, 1 = finished
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ScoreComponent {
    pub black_count: u32,
    pub white_count: u32,
}

// ── ComponentTrait implementations ───────────────────────────────────────────

impl ComponentTrait for BoardComponent {
    fn component_type() -> Symbol {
        symbol_short!("boardcomp")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.width.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.height.to_be_bytes()));
        for i in 0..(BOARD_SIZE * BOARD_SIZE) {
            let cell = self.cells.get(i).unwrap_or(EMPTY);
            bytes.append(&Bytes::from_array(env, &cell.to_be_bytes()));
        }
        bytes
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        // 4 (width) + 4 (height) + 64 * 4 (cells) = 272
        if data.len() != 272 {
            return None;
        }
        let width = u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]);
        let height = u32::from_be_bytes([data.get(4)?, data.get(5)?, data.get(6)?, data.get(7)?]);
        let mut cells = Vec::new(env);
        for i in 0..(BOARD_SIZE * BOARD_SIZE) {
            let offset = 8 + i * 4;
            let cell = u32::from_be_bytes([
                data.get(offset)?,
                data.get(offset + 1)?,
                data.get(offset + 2)?,
                data.get(offset + 3)?,
            ]);
            cells.push_back(cell);
        }
        Some(Self {
            cells,
            width,
            height,
        })
    }
}

impl ComponentTrait for TurnComponent {
    fn component_type() -> Symbol {
        symbol_short!("turncomp")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.current_player.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.pass_count.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 8 {
            return None;
        }
        let current_player =
            u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]);
        let pass_count =
            u32::from_be_bytes([data.get(4)?, data.get(5)?, data.get(6)?, data.get(7)?]);
        Some(Self {
            current_player,
            pass_count,
        })
    }
}

impl ComponentTrait for GameStatusComponent {
    fn component_type() -> Symbol {
        symbol_short!("gamestatu")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.status.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 4 {
            return None;
        }
        let status = u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]);
        Some(Self { status })
    }
}

impl ComponentTrait for ScoreComponent {
    fn component_type() -> Symbol {
        symbol_short!("scorecomp")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.black_count.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.white_count.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 8 {
            return None;
        }
        let black_count =
            u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]);
        let white_count =
            u32::from_be_bytes([data.get(4)?, data.get(5)?, data.get(6)?, data.get(7)?]);
        Some(Self {
            black_count,
            white_count,
        })
    }
}

// ── ECS World ────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct ECSWorldState {
    pub board: BoardComponent,
    pub turn: TurnComponent,
    pub status: GameStatusComponent,
    pub score: ScoreComponent,
    pub player_one: Address, // plays BLACK
    pub player_two: Address, // plays WHITE
}

// ── Public API types ─────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    pub current_player: u32,
    pub pass_count: u32,
    pub status: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoardState {
    pub cells: Vec<u32>,
    pub width: u32,
    pub height: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScoreState {
    pub black_count: u32,
    pub white_count: u32,
    pub winner: u32, // 0 = ongoing, 1 = black wins, 2 = white wins, 3 = draw
}
