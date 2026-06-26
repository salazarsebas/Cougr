use cougr_core::component::ComponentTrait;
use soroban_sdk::{contracttype, symbol_short, Address, Bytes, Env, Symbol, Vec};

/// Board dimensions: 7 columns x 6 rows
pub(crate) const ROWS: u32 = 6;
pub(crate) const COLS: u32 = 7;

pub(crate) const WORLD_KEY: Symbol = symbol_short!("WORLD");

/// Cell value: 0=Empty, 1=Player1, 2=Player2
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Cell {
    Empty = 0,
    Player1 = 1,
    Player2 = 2,
}

/// Board component - stores the 7x6 game board state
#[contracttype]
#[derive(Clone, Debug)]
pub struct BoardComponent {
    pub cells: Vec<u32>, // Flattened 2D array: index = row * COLS + col
    pub entity_id: u32,
}

impl BoardComponent {
    pub fn new(env: &Env, entity_id: u32) -> Self {
        let mut cells = Vec::new(env);
        for _ in 0..(ROWS * COLS) {
            cells.push_back(0u32);
        }
        Self { cells, entity_id }
    }

    /// Get cell value at (row, col)
    pub fn get_cell(&self, _env: &Env, row: u32, col: u32) -> u32 {
        if row >= ROWS || col >= COLS {
            return 0;
        }
        let index = row * COLS + col;
        self.cells.get(index).unwrap_or(0)
    }

    /// Set cell value at (row, col)
    pub fn set_cell(&mut self, _env: &Env, row: u32, col: u32, value: u32) {
        if row >= ROWS || col >= COLS {
            return;
        }
        let index = row * COLS + col;
        self.cells.set(index, value);
    }

    /// Find the lowest empty row in a column (gravity-based placement)
    pub fn get_lowest_empty_row(&self, _env: &Env, col: u32) -> Option<u32> {
        if col >= COLS {
            return None;
        }

        // Start from bottom row and go up
        for row in (0..ROWS).rev() {
            let index = row * COLS + col;
            let cell = self.cells.get(index).unwrap_or(0);
            if cell == 0 {
                return Some(row);
            }
        }
        None // Column is full
    }

    /// Check if column is full
    pub fn is_column_full(&self, env: &Env, col: u32) -> bool {
        self.get_lowest_empty_row(env, col).is_none()
    }
}

impl ComponentTrait for BoardComponent {
    fn component_type() -> Symbol {
        symbol_short!("board")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.entity_id.to_be_bytes()));
        let len = self.cells.len();
        bytes.append(&Bytes::from_array(env, &len.to_be_bytes()));
        for i in 0..len {
            let cell = self.cells.get(i).unwrap_or(0);
            bytes.append(&Bytes::from_array(env, &cell.to_be_bytes()));
        }
        bytes
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }
        let entity_id = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let len = u32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);

        let mut cells = Vec::new(env);
        for i in 0..len {
            let offset = 8 + i * 4;
            if offset + 4 > data.len() {
                break;
            }
            let cell = u32::from_be_bytes([
                data.get(offset).unwrap(),
                data.get(offset + 1).unwrap(),
                data.get(offset + 2).unwrap(),
                data.get(offset + 3).unwrap(),
            ]);
            cells.push_back(cell);
        }
        Some(Self { cells, entity_id })
    }
}

/// Player component - stores both players' addresses
#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerComponent {
    pub player_one: Address,
    pub player_two: Address,
    pub entity_id: u32,
}

impl PlayerComponent {
    pub fn new(player_one: Address, player_two: Address, entity_id: u32) -> Self {
        Self {
            player_one,
            player_two,
            entity_id,
        }
    }
}

/// Game state component
#[contracttype]
#[derive(Clone, Debug)]
pub struct GameStateComponent {
    pub is_player_one_turn: bool,
    pub move_count: u32,
    pub status: u32, // 0=InProgress, 1=Player1Wins, 2=Player2Wins, 3=Draw
    pub last_move_col: Option<u32>,
    pub entity_id: u32,
}

impl GameStateComponent {
    pub fn new(entity_id: u32) -> Self {
        Self {
            is_player_one_turn: true,
            move_count: 0,
            status: 0,
            last_move_col: None,
            entity_id,
        }
    }
}

impl ComponentTrait for GameStateComponent {
    fn component_type() -> Symbol {
        symbol_short!("gstate")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.entity_id.to_be_bytes()));
        bytes.append(&Bytes::from_array(
            env,
            &[if self.is_player_one_turn { 1 } else { 0 }],
        ));
        bytes.append(&Bytes::from_array(env, &self.move_count.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.status.to_be_bytes()));

        // Serialize Option<u32> for last_move_col
        match self.last_move_col {
            Some(col) => {
                bytes.append(&Bytes::from_array(env, &[1u8])); // Some
                bytes.append(&Bytes::from_array(env, &col.to_be_bytes()));
            }
            None => {
                bytes.append(&Bytes::from_array(env, &[0u8])); // None
            }
        }

        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 14 {
            return None;
        }
        let entity_id = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let is_player_one_turn = data.get(4).unwrap() != 0;
        let move_count = u32::from_be_bytes([
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
            data.get(8).unwrap(),
        ]);
        let status = u32::from_be_bytes([
            data.get(9).unwrap(),
            data.get(10).unwrap(),
            data.get(11).unwrap(),
            data.get(12).unwrap(),
        ]);

        let has_last_move = data.get(13).unwrap() != 0;
        let last_move_col = if has_last_move && data.len() >= 18 {
            let col = u32::from_be_bytes([
                data.get(14).unwrap(),
                data.get(15).unwrap(),
                data.get(16).unwrap(),
                data.get(17).unwrap(),
            ]);
            Some(col)
        } else {
            None
        };

        Some(Self {
            is_player_one_turn,
            move_count,
            status,
            last_move_col,
            entity_id,
        })
    }
}

/// ECS World State - stores all game entities and components
#[contracttype]
#[derive(Clone, Debug)]
pub struct ECSWorldState {
    pub board: BoardComponent,
    pub players: PlayerComponent,
    pub game_state: GameStateComponent,
    pub next_entity_id: u32,
}

/// External game state for API consumers
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    pub board: Vec<u32>, // Flattened 7x6 board
    pub rows: u32,
    pub cols: u32,
    pub player_one: Address,
    pub player_two: Address,
    pub is_player_one_turn: bool,
    pub move_count: u32,
    pub status: u32, // 0=InProgress, 1=P1Wins, 2=P2Wins, 3=Draw
    pub last_move_col: Option<u32>,
}

/// Move result returned after each move
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DropResult {
    pub success: bool,
    pub game_state: GameState,
    pub message: Symbol,
    pub row_placed: Option<u32>,
}
