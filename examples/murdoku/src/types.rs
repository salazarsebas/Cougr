//! Domain types and invariants for the Murdoku puzzle game contract.

use soroban_sdk::{contracterror, contracttype, Address, String, Vec};

/*
INVARIANTS:
1. A valid grid has exactly grid_size * grid_size cells.
2. A valid solution has exactly one occurrence of each suspect index (1..=N) per row and per column.
3. ClueType::MustBeInCell requires both row and col to be set; adjacency clues require two suspect IDs.
4. Grid size must be 4 or 5 for v1; reject other values at submit time (enforced in the registry issue).
*/

/// An identifier and display name for a suspect in a puzzle.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Suspect {
    pub id: u32,
    pub name: String,
}

/// The type of clue/constraint constraint on the puzzle.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ClueType {
    MustBeInCell = 1,
    CannotBeInCell = 2,
    MustBeInRow = 3,
    MustBeInCol = 4,
    MustBeAdjacentTo = 5,
    CannotBeAdjacentTo = 6,
}

/// A single constraint: which ClueType, which suspect(s), and the relevant coordinates or relationship.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Clue {
    pub clue_type: ClueType,
    pub suspect_ids: Vec<u32>,
    pub row: Option<u32>,
    pub col: Option<u32>,
}

/// Metadata associated with a puzzle.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PuzzleMetadata {
    pub creator: Address,
    pub grid_size: u32,
    pub difficulty: u32,
    pub name: String,
    pub creation_ledger: u32,
}

/// Outcome of a placement attempt.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum MoveResult {
    Ok = 1,
    InvalidCoordinates = 2,
    RowConflict = 3,
    ColConflict = 4,
    CellOccupied = 5,
    GameAlreadySolved = 6,
    GameNotStarted = 7,
}

/// Contract-level panic reasons.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PuzzleError {
    InvalidGridSize = 1,
    InvalidSuspects = 2,
    InvalidSolution = 3,
    InvalidClues = 4,
    PuzzleNotFound = 5,
    Unauthorized = 6,
    GameAlreadySolved = 7,
    GameNotStarted = 8,
}
