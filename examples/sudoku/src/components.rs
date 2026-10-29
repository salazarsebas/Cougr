#![allow(dead_code)]

use cougr_core::component::ComponentTrait;
use soroban_sdk::{contracttype, symbol_short, Bytes, Env, Symbol, Vec};

// ── Constants ────────────────────────────────────────────────────────────────

pub(crate) const BOARD_SIZE: u32 = 9;
pub(crate) const EMPTY: u32 = 0;
pub(crate) const STATUS_PLAYING: u32 = 0;
pub(crate) const STATUS_SOLVED: u32 = 1;
pub(crate) const WORLD_KEY: Symbol = symbol_short!("WORLD");

// ── Components ───────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct BoardComponent {
    pub cells: Vec<u32>, // 81 elements, row-major; 0=empty, 1–9=value
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct FixedCellsComponent {
    pub fixed: Vec<bool>, // 81 elements; true=immutable (puzzle givens)
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct GameStatusComponent {
    pub status: u32, // 0=playing, 1=solved
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct MoveCountComponent {
    pub moves: u32,
}

// ── ComponentTrait implementations ───────────────────────────────────────────

impl ComponentTrait for BoardComponent {
    fn component_type() -> Symbol {
        symbol_short!("sudokubd")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        for i in 0..(BOARD_SIZE * BOARD_SIZE) {
            let cell = self.cells.get(i).unwrap_or(EMPTY);
            bytes.append(&Bytes::from_array(env, &cell.to_be_bytes()));
        }
        bytes
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        // 81 cells * 4 bytes = 324
        if data.len() != 324 {
            return None;
        }
        let mut cells = Vec::new(env);
        for i in 0..(BOARD_SIZE * BOARD_SIZE) {
            let offset = i * 4;
            let cell = u32::from_be_bytes([
                data.get(offset)?,
                data.get(offset + 1)?,
                data.get(offset + 2)?,
                data.get(offset + 3)?,
            ]);
            cells.push_back(cell);
        }
        Some(Self { cells })
    }
}

impl ComponentTrait for FixedCellsComponent {
    fn component_type() -> Symbol {
        symbol_short!("fixed")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        for i in 0..(BOARD_SIZE * BOARD_SIZE) {
            let flag: u8 = if self.fixed.get(i).unwrap_or(false) {
                1
            } else {
                0
            };
            bytes.append(&Bytes::from_array(env, &[flag, 0, 0, 0]));
        }
        bytes
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        // 81 cells * 4 bytes each (bool stored as u32-aligned byte) = 324
        if data.len() != 324 {
            return None;
        }
        let mut fixed = Vec::new(env);
        for i in 0..(BOARD_SIZE * BOARD_SIZE) {
            fixed.push_back(data.get(i * 4)? != 0);
        }
        Some(Self { fixed })
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

impl ComponentTrait for MoveCountComponent {
    fn component_type() -> Symbol {
        symbol_short!("movecount")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.moves.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 4 {
            return None;
        }
        let moves = u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]);
        Some(Self { moves })
    }
}

// ── ECS World ────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct ECSWorldState {
    pub board: BoardComponent,
    pub fixed: FixedCellsComponent,
    pub status: GameStatusComponent,
    pub moves: MoveCountComponent,
}

// ── Public API types ─────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    pub status: u32, // 0=playing, 1=solved
    pub moves: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellState {
    pub value: u32,  // 0=empty, 1–9
    pub fixed: bool, // true=immutable
}
