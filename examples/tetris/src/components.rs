#![allow(dead_code)]

use cougr_core::ComponentTrait;
use soroban_sdk::{contracttype, symbol_short, Bytes, Env, Symbol};

/// The active tetromino shape.
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

/// Component that stores the active piece's shape, grid position, and rotation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PieceComponent {
    pub shape: TetrominoShape,
    pub x: i32,
    pub y: i32,
    pub rotation: u32,
}

impl PieceComponent {
    pub fn new(shape: TetrominoShape, x: i32, y: i32) -> Self {
        Self { shape, x, y, rotation: 0 }
    }
}

impl ComponentTrait for PieceComponent {
    fn component_type() -> Symbol {
        symbol_short!("piece")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &[self.shape as u8]));
        bytes.append(&Bytes::from_array(env, &self.x.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.y.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.rotation.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 13 {
            return None;
        }
        let shape = match data.get(0).unwrap() {
            0 => TetrominoShape::I,
            1 => TetrominoShape::J,
            2 => TetrominoShape::L,
            3 => TetrominoShape::O,
            4 => TetrominoShape::S,
            5 => TetrominoShape::T,
            6 => TetrominoShape::Z,
            _ => return None,
        };
        let x = i32::from_be_bytes([
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
            data.get(4).unwrap(),
        ]);
        let y = i32::from_be_bytes([
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
            data.get(8).unwrap(),
        ]);
        let rotation = u32::from_be_bytes([
            data.get(9).unwrap(),
            data.get(10).unwrap(),
            data.get(11).unwrap(),
            data.get(12).unwrap(),
        ]);
        Some(Self { shape, x, y, rotation })
    }
}
