use cougr_core::ComponentTrait;
use soroban_sdk::{contracttype, symbol_short, Bytes, Env, Symbol};

// ── Game constants ────────────────────────────────────────────────────────────

pub const GAME_WIDTH: i32 = 40;
pub const GAME_HEIGHT: i32 = 30;
pub const INVADER_COLS: u32 = 8;
pub const INVADER_ROWS: u32 = 4;
pub const SHIP_Y: i32 = GAME_HEIGHT - 2;
pub const INVADER_WIN_Y: i32 = SHIP_Y - 2;
pub const SHOOT_COOLDOWN: u32 = 3;
pub const BULLET_SPEED: i32 = 2;
pub const INVADER_MOVE_INTERVAL: u32 = 5;

// ── Tag components ────────────────────────────────────────────────────────────

/// Marks an entity as the player ship.
#[derive(Clone, Debug)]
pub struct ShipTag;

impl ComponentTrait for ShipTag {
    fn component_type() -> Symbol { symbol_short!("ship") }
    fn serialize(&self, env: &Env) -> Bytes { Bytes::from_array(env, &[1u8]) }
    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() == 1 { Some(Self) } else { None }
    }
}

/// Marks an entity as an invader and stores its point value and alive flag.
#[derive(Clone, Debug)]
pub struct InvaderTag {
    pub points: u32,
    pub active: bool,
}

impl ComponentTrait for InvaderTag {
    fn component_type() -> Symbol { symbol_short!("invader") }
    fn serialize(&self, env: &Env) -> Bytes {
        let mut b = Bytes::new(env);
        b.append(&Bytes::from_array(env, &self.points.to_be_bytes()));
        b.append(&Bytes::from_array(env, &[if self.active { 1u8 } else { 0u8 }]));
        b
    }
    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 5 { return None; }
        let points = u32::from_be_bytes([
            data.get(0).unwrap(), data.get(1).unwrap(),
            data.get(2).unwrap(), data.get(3).unwrap(),
        ]);
        let active = data.get(4).unwrap() != 0;
        Some(Self { points, active })
    }
}

/// Marks an entity as a bullet and stores its owner (0 = player, 1 = enemy).
#[derive(Clone, Debug)]
pub struct BulletTag {
    pub owner: u8,
}

impl ComponentTrait for BulletTag {
    fn component_type() -> Symbol { symbol_short!("bullet") }
    fn serialize(&self, env: &Env) -> Bytes {
        Bytes::from_array(env, &[self.owner])
    }
    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() == 1 { Some(Self { owner: data.get(0).unwrap() }) } else { None }
    }
}

// ── Data components ───────────────────────────────────────────────────────────

/// 2-D position on the game grid.
#[derive(Clone, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self { Self { x, y } }
}

impl ComponentTrait for Position {
    fn component_type() -> Symbol { symbol_short!("position") }
    fn serialize(&self, env: &Env) -> Bytes {
        let mut b = Bytes::new(env);
        b.append(&Bytes::from_array(env, &self.x.to_be_bytes()));
        b.append(&Bytes::from_array(env, &self.y.to_be_bytes()));
        b
    }
    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 8 { return None; }
        let x = i32::from_be_bytes([
            data.get(0).unwrap(), data.get(1).unwrap(),
            data.get(2).unwrap(), data.get(3).unwrap(),
        ]);
        let y = i32::from_be_bytes([
            data.get(4).unwrap(), data.get(5).unwrap(),
            data.get(6).unwrap(), data.get(7).unwrap(),
        ]);
        Some(Self { x, y })
    }
}

/// Per-tick velocity.
#[derive(Clone, Debug)]
pub struct Velocity {
    pub dx: i32,
    pub dy: i32,
}

impl Velocity {
    pub fn new(dx: i32, dy: i32) -> Self { Self { dx, dy } }
}

impl ComponentTrait for Velocity {
    fn component_type() -> Symbol { symbol_short!("velocity") }
    fn serialize(&self, env: &Env) -> Bytes {
        let mut b = Bytes::new(env);
        b.append(&Bytes::from_array(env, &self.dx.to_be_bytes()));
        b.append(&Bytes::from_array(env, &self.dy.to_be_bytes()));
        b
    }
    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 8 { return None; }
        let dx = i32::from_be_bytes([
            data.get(0).unwrap(), data.get(1).unwrap(),
            data.get(2).unwrap(), data.get(3).unwrap(),
        ]);
        let dy = i32::from_be_bytes([
            data.get(4).unwrap(), data.get(5).unwrap(),
            data.get(6).unwrap(), data.get(7).unwrap(),
        ]);
        Some(Self { dx, dy })
    }
}

/// Hit-point pool.
#[derive(Clone, Debug)]
pub struct Health {
    pub current: u32,
}

impl Health {
    pub fn new(hp: u32) -> Self { Self { current: hp } }
    pub fn is_alive(&self) -> bool { self.current > 0 }
    pub fn take_damage(&mut self) { self.current = self.current.saturating_sub(1); }
}

impl ComponentTrait for Health {
    fn component_type() -> Symbol { symbol_short!("health") }
    fn serialize(&self, env: &Env) -> Bytes {
        Bytes::from_array(env, &self.current.to_be_bytes())
    }
    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 4 { return None; }
        let current = u32::from_be_bytes([
            data.get(0).unwrap(), data.get(1).unwrap(),
            data.get(2).unwrap(), data.get(3).unwrap(),
        ]);
        Some(Self { current })
    }
}

/// Invader type, used to determine point value.
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum InvaderType {
    Squid = 0,
    Crab = 1,
    Octopus = 2,
}

impl InvaderType {
    pub fn points(self) -> u32 {
        match self {
            InvaderType::Squid => 30,
            InvaderType::Crab => 20,
            InvaderType::Octopus => 10,
        }
    }
}
