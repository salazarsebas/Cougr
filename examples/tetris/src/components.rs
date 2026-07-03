use soroban_sdk::{contracttype, Vec};

#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TetrominoShape {
    I = 0,
    J = 1,
    L = 2,
    O = 3,
    S = 4,
    T = 5,
    Z = 6,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    pub shape: TetrominoShape,
    pub x: i32,
    pub y: i32,
    pub rotation: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    pub board: Vec<u32>,
    pub current_piece: Piece,
    pub next_piece: Piece,
    pub score: u32,
    pub level: u32,
    pub lines_cleared: u32,
    pub game_over: bool,
}
