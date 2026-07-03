//! Murdoku game components using cougr-core's ComponentTrait

use cougr_core::component::{ComponentStorage, ComponentTrait};
use soroban_sdk::xdr::{FromXdr, ToXdr};
use soroban_sdk::{contracttype, symbol_short, Bytes, Env, String, Symbol, Vec};

/// A single pre-filled cell clue in the puzzle grid.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Clue {
    pub row: u32,
    pub col: u32,
    pub suspect_idx: u32,
}

/// Metadata associated with a Murdoku puzzle.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PuzzleMetadata {
    pub name: String,
    pub difficulty: String,
}

/// Grid size component representing the grid's side length.
#[derive(Clone, Debug, PartialEq)]
pub struct GridSize {
    pub size: u32,
}

impl ComponentTrait for GridSize {
    fn component_type() -> Symbol {
        symbol_short!("gridsize")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        self.size.to_xdr(env)
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        let size = u32::from_xdr(env, data).ok()?;
        Some(Self { size })
    }

    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table
    }
}

/// Suspects component storing the list of names.
#[derive(Clone, Debug, PartialEq)]
pub struct Suspects {
    pub list: Vec<String>,
}

impl ComponentTrait for Suspects {
    fn component_type() -> Symbol {
        symbol_short!("suspects")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        self.list.clone().to_xdr(env)
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        let list = soroban_sdk::Vec::<String>::from_xdr(env, data).ok()?;
        Some(Self { list })
    }

    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table
    }
}

/// Clues component storing the list of pre-filled cells.
#[derive(Clone, Debug, PartialEq)]
pub struct Clues {
    pub list: Vec<Clue>,
}

impl ComponentTrait for Clues {
    fn component_type() -> Symbol {
        symbol_short!("clues")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        self.list.clone().to_xdr(env)
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        let list = soroban_sdk::Vec::<Clue>::from_xdr(env, data).ok()?;
        Some(Self { list })
    }

    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table
    }
}

/// Solution component storing the full flat Latin square solution.
#[derive(Clone, Debug, PartialEq)]
pub struct Solution {
    pub grid: Vec<u32>,
}

impl ComponentTrait for Solution {
    fn component_type() -> Symbol {
        symbol_short!("solution")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        self.grid.clone().to_xdr(env)
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        let grid = soroban_sdk::Vec::<u32>::from_xdr(env, data).ok()?;
        Some(Self { grid })
    }

    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table
    }
}

/// Metadata component storing description/name.
#[derive(Clone, Debug, PartialEq)]
pub struct Metadata {
    pub meta: PuzzleMetadata,
}

impl ComponentTrait for Metadata {
    fn component_type() -> Symbol {
        symbol_short!("metadata")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        self.meta.clone().to_xdr(env)
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        let meta = PuzzleMetadata::from_xdr(env, data).ok()?;
        Some(Self { meta })
    }

    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table
    }
}

/// Active player board state (flat grid of suspect indices, 0 = empty).
#[derive(Clone, Debug, PartialEq)]
pub struct Board {
    pub grid: Vec<u32>,
}

impl ComponentTrait for Board {
    fn component_type() -> Symbol {
        symbol_short!("board")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        self.grid.clone().to_xdr(env)
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        let grid = soroban_sdk::Vec::<u32>::from_xdr(env, data).ok()?;
        Some(Self { grid })
    }

    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table
    }
}

/// Tracks whether the player has solved the puzzle.
#[derive(Clone, Debug, PartialEq)]
pub struct GameStatus {
    pub solved: bool,
}

impl ComponentTrait for GameStatus {
    fn component_type() -> Symbol {
        symbol_short!("status")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        self.solved.to_xdr(env)
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        let solved = bool::from_xdr(env, data).ok()?;
        Some(Self { solved })
    }

    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table
    }
}

/// Counts moves made in the current session.
#[derive(Clone, Debug, PartialEq)]
pub struct MoveCount {
    pub count: u32,
}

impl ComponentTrait for MoveCount {
    fn component_type() -> Symbol {
        symbol_short!("moves")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        self.count.to_xdr(env)
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        let count = u32::from_xdr(env, data).ok()?;
        Some(Self { count })
    }

    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table
    }
}

/// Solution commitment component storing the Poseidon2 hash (ZK mode).
#[cfg(feature = "zk")]
#[derive(Clone, Debug, PartialEq)]
pub struct SolutionCommitment {
    pub commitment: soroban_sdk::BytesN<32>,
}

#[cfg(feature = "zk")]
impl ComponentTrait for SolutionCommitment {
    fn component_type() -> Symbol {
        symbol_short!("solcommit")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        self.commitment.clone().to_xdr(env)
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        let commitment = soroban_sdk::BytesN::<32>::from_xdr(env, data).ok()?;
        Some(Self { commitment })
    }

    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table
    }
}
