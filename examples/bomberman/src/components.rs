use cougr_core::component::ComponentTrait;
use soroban_sdk::{contracttype, symbol_short, Bytes, Env, Symbol, Vec};

// ── Constants ────────────────────────────────────────────────────────────────

pub const GRID_WIDTH: usize = 15;
pub const GRID_HEIGHT: usize = 13;
pub const INITIAL_LIVES: u32 = 3;
pub const BOMB_TIMER: u32 = 3;
pub const EXPLOSION_DURATION: u32 = 1;

// ── PlayerComponent ──────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub struct PlayerComponent {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub lives: u32,
    pub bomb_capacity: u32,
    pub score: u32,
    pub bomb_power: u32,
    pub speed: u32,
}

impl PlayerComponent {
    pub fn new(id: u32, x: i32, y: i32) -> Self {
        Self {
            id,
            x,
            y,
            lives: INITIAL_LIVES,
            bomb_capacity: 1,
            score: 0,
            bomb_power: 1,
            speed: 1,
        }
    }
}

impl ComponentTrait for PlayerComponent {
    fn component_type() -> Symbol {
        symbol_short!("player")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.id.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.x.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.y.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.lives.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.bomb_capacity.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.score.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.bomb_power.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.speed.to_be_bytes()));
        bytes
    }

    #[allow(unused_variables)]
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 32 {
            return None;
        }
        let id = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let x = i32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);
        let y = i32::from_be_bytes([
            data.get(8).unwrap(),
            data.get(9).unwrap(),
            data.get(10).unwrap(),
            data.get(11).unwrap(),
        ]);
        let lives = u32::from_be_bytes([
            data.get(12).unwrap(),
            data.get(13).unwrap(),
            data.get(14).unwrap(),
            data.get(15).unwrap(),
        ]);
        let bomb_capacity = u32::from_be_bytes([
            data.get(16).unwrap(),
            data.get(17).unwrap(),
            data.get(18).unwrap(),
            data.get(19).unwrap(),
        ]);
        let score = u32::from_be_bytes([
            data.get(20).unwrap(),
            data.get(21).unwrap(),
            data.get(22).unwrap(),
            data.get(23).unwrap(),
        ]);
        let bomb_power = u32::from_be_bytes([
            data.get(24).unwrap(),
            data.get(25).unwrap(),
            data.get(26).unwrap(),
            data.get(27).unwrap(),
        ]);
        let speed = u32::from_be_bytes([
            data.get(28).unwrap(),
            data.get(29).unwrap(),
            data.get(30).unwrap(),
            data.get(31).unwrap(),
        ]);
        Some(Self {
            id,
            x,
            y,
            lives,
            bomb_capacity,
            score,
            bomb_power,
            speed,
        })
    }
}

// ── BombComponent ────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub struct BombComponent {
    pub x: i32,
    pub y: i32,
    pub timer: u32,
    pub power: u32,
    pub owner_id: u32,
}

impl BombComponent {
    pub fn new(x: i32, y: i32, owner_id: u32) -> Self {
        Self {
            x,
            y,
            timer: BOMB_TIMER,
            power: 1,
            owner_id,
        }
    }
}

impl ComponentTrait for BombComponent {
    fn component_type() -> Symbol {
        symbol_short!("bomb")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.x.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.y.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.timer.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.power.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.owner_id.to_be_bytes()));
        bytes
    }

    #[allow(unused_variables)]
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 20 {
            return None;
        }
        let x = i32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let y = i32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);
        let timer = u32::from_be_bytes([
            data.get(8).unwrap(),
            data.get(9).unwrap(),
            data.get(10).unwrap(),
            data.get(11).unwrap(),
        ]);
        let power = u32::from_be_bytes([
            data.get(12).unwrap(),
            data.get(13).unwrap(),
            data.get(14).unwrap(),
            data.get(15).unwrap(),
        ]);
        let owner_id = u32::from_be_bytes([
            data.get(16).unwrap(),
            data.get(17).unwrap(),
            data.get(18).unwrap(),
            data.get(19).unwrap(),
        ]);
        Some(Self {
            x,
            y,
            timer,
            power,
            owner_id,
        })
    }
}

// ── ExplosionComponent ───────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub struct ExplosionComponent {
    pub x: i32,
    pub y: i32,
    pub timer: u32,
}

impl ExplosionComponent {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            timer: EXPLOSION_DURATION,
        }
    }
}

impl ComponentTrait for ExplosionComponent {
    fn component_type() -> Symbol {
        symbol_short!("explosion")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.x.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.y.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.timer.to_be_bytes()));
        bytes
    }

    #[allow(unused_variables)]
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 12 {
            return None;
        }
        let x = i32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let y = i32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);
        let timer = u32::from_be_bytes([
            data.get(8).unwrap(),
            data.get(9).unwrap(),
            data.get(10).unwrap(),
            data.get(11).unwrap(),
        ]);
        Some(Self { x, y, timer })
    }
}

// ── PowerUpType / PowerUpComponent ───────────────────────────────────────────

#[contracttype]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PowerUpType {
    Capacity = 0,
    Power = 1,
    Speed = 2,
}

#[contracttype]
#[derive(Clone)]
pub struct PowerUpComponent {
    pub x: i32,
    pub y: i32,
    pub power_up_type: PowerUpType,
}

impl PowerUpComponent {
    pub fn new(x: i32, y: i32, power_up_type: PowerUpType) -> Self {
        Self {
            x,
            y,
            power_up_type,
        }
    }
}

impl ComponentTrait for PowerUpComponent {
    fn component_type() -> Symbol {
        symbol_short!("powerup")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.x.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.y.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &[self.power_up_type as u8]));
        bytes
    }

    #[allow(unused_variables)]
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 9 {
            return None;
        }
        let x = i32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let y = i32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);
        let power_up_type = match data.get(8).unwrap() {
            0 => PowerUpType::Capacity,
            1 => PowerUpType::Power,
            2 => PowerUpType::Speed,
            _ => return None,
        };
        Some(Self {
            x,
            y,
            power_up_type,
        })
    }
}

// ── CellType / GridComponent ─────────────────────────────────────────────────

#[contracttype]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellType {
    Empty = 0,
    Wall = 1,
    Destructible = 2,
    PowerUp = 3,
}

#[contracttype]
#[derive(Clone)]
pub struct GridComponent {
    pub cells: Vec<CellType>,
}

impl GridComponent {
    #[allow(clippy::if_same_then_else)]
    pub fn new(env: &Env) -> Self {
        let mut cells = Vec::new(env);
        for _ in 0..(GRID_WIDTH * GRID_HEIGHT) {
            cells.push_back(CellType::Empty);
        }
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let index = y * GRID_WIDTH + x;
                if x == 0 || x == GRID_WIDTH - 1 || y == 0 || y == GRID_HEIGHT - 1 {
                    cells.set(index as u32, CellType::Wall);
                } else if x % 2 == 0 && y % 2 == 0 {
                    cells.set(index as u32, CellType::Wall);
                }
            }
        }
        for x in 1..GRID_WIDTH - 1 {
            for y in 1..GRID_HEIGHT - 1 {
                let index = y * GRID_WIDTH + x;
                if (x + y) % 3 == 0 && cells.get(index as u32).unwrap() == CellType::Empty {
                    cells.set(index as u32, CellType::Destructible);
                }
            }
        }
        Self { cells }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> CellType {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            self.cells
                .get((y * GRID_WIDTH + x) as u32)
                .unwrap_or(CellType::Wall)
        } else {
            CellType::Wall
        }
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell_type: CellType) {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            self.cells.set((y * GRID_WIDTH + x) as u32, cell_type);
        }
    }

    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= GRID_WIDTH as i32 || y >= GRID_HEIGHT as i32 {
            return false;
        }
        matches!(
            self.get_cell(x as usize, y as usize),
            CellType::Empty | CellType::PowerUp
        )
    }
}

impl ComponentTrait for GridComponent {
    fn component_type() -> Symbol {
        symbol_short!("grid")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        for cell in self.cells.iter() {
            bytes.append(&Bytes::from_array(env, &[cell as u8]));
        }
        bytes
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != (GRID_WIDTH * GRID_HEIGHT) as u32 {
            return None;
        }
        let mut cells = Vec::new(env);
        for i in 0..GRID_WIDTH * GRID_HEIGHT {
            let cell = match data.get(i as u32).unwrap() {
                0 => CellType::Empty,
                1 => CellType::Wall,
                2 => CellType::Destructible,
                3 => CellType::PowerUp,
                _ => return None,
            };
            cells.push_back(cell);
        }
        Some(Self { cells })
    }
}

// ── GameStateComponent ───────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub struct GameStateComponent {
    pub current_tick: u32,
    pub game_over: bool,
    pub winner_id: Option<u32>,
}

impl GameStateComponent {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            current_tick: 0,
            game_over: false,
            winner_id: None,
        }
    }
}

impl ComponentTrait for GameStateComponent {
    fn component_type() -> Symbol {
        symbol_short!("gstate")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.current_tick.to_be_bytes()));
        bytes.append(&Bytes::from_array(
            env,
            &[if self.game_over { 1 } else { 0 }],
        ));
        match self.winner_id {
            Some(id) => {
                bytes.append(&Bytes::from_array(env, &[1]));
                bytes.append(&Bytes::from_array(env, &id.to_be_bytes()));
            }
            None => {
                bytes.append(&Bytes::from_array(env, &[0]));
            }
        }
        bytes
    }

    #[allow(unused_variables)]
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 6 {
            return None;
        }
        let current_tick = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let game_over = data.get(4).unwrap() != 0;
        let has_winner = data.get(5).unwrap() != 0;
        let winner_id = if has_winner && data.len() >= 10 {
            Some(u32::from_be_bytes([
                data.get(6).unwrap(),
                data.get(7).unwrap(),
                data.get(8).unwrap(),
                data.get(9).unwrap(),
            ]))
        } else {
            None
        };
        Some(Self {
            current_tick,
            game_over,
            winner_id,
        })
    }
}
