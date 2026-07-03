use cougr_core::component::ComponentTrait;
use soroban_sdk::{contracttype, symbol_short, Address, Bytes, BytesN, Env, Map, Symbol};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: Color,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct BoardState {
    pub state_hash: BytesN<32>,
    pub pieces: Map<u32, Piece>,
}

impl ComponentTrait for BoardState {
    fn component_type() -> Symbol {
        symbol_short!("board")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        for i in 0..32 {
            bytes.push_back(self.state_hash.get(i).unwrap());
        }
        bytes
    }

    fn deserialize(_env: &Env, _data: &Bytes) -> Option<Self> {
        None
    }
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameStatus {
    Playing,
    Checkmate,
    Draw,
    Resigned,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct TurnState {
    pub current: Address,
    pub move_count: u32,
    pub status: GameStatus,
}

impl ComponentTrait for TurnState {
    fn component_type() -> Symbol {
        symbol_short!("turn")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.move_count.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, _data: &Bytes) -> Option<Self> {
        None
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ProofRecord {
    pub last_proof: Bytes,
    pub verified: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct GameState {
    pub white: Address,
    pub black: Address,
    pub board: BoardState,
    pub turn: TurnState,
    pub proof_record: ProofRecord,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MoveResult {
    Success,
    InvalidProof,
    WrongTurn,
    GameOver,
}

pub(crate) const GAME_KEY: Symbol = symbol_short!("GAME");
pub(crate) const VK_KEY: Symbol = symbol_short!("VK");
