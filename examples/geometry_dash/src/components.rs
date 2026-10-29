use cougr_core::ComponentTrait;
use soroban_sdk::{contracttype, symbol_short, Bytes, Env, Symbol};

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum PlayerMode {
    Cube = 0,
    Ship = 1,
    Wave = 2,
    Ball = 3,
}

impl ComponentTrait for PlayerMode {
    fn component_type() -> Symbol {
        symbol_short!("mode")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        let val: u8 = match self {
            PlayerMode::Cube => 0,
            PlayerMode::Ship => 1,
            PlayerMode::Wave => 2,
            PlayerMode::Ball => 3,
        };
        bytes.append(&Bytes::from_array(env, &[val]));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 1 {
            return None;
        }
        match data.get(0).unwrap() {
            0 => Some(PlayerMode::Cube),
            1 => Some(PlayerMode::Ship),
            2 => Some(PlayerMode::Wave),
            3 => Some(PlayerMode::Ball),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum ObstacleKind {
    Spike = 0,
    Block = 1,
    Portal = 2,
    Pad = 3,
}

#[contracttype]
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Playing = 0,
    Crashed = 1,
    Completed = 2,
}

impl ComponentTrait for GameStatus {
    fn component_type() -> Symbol {
        symbol_short!("status")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        let val: u8 = match self {
            GameStatus::Playing => 0,
            GameStatus::Crashed => 1,
            GameStatus::Completed => 2,
        };
        bytes.append(&Bytes::from_array(env, &[val]));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 1 {
            return None;
        }
        match data.get(0).unwrap() {
            0 => Some(GameStatus::Playing),
            1 => Some(GameStatus::Crashed),
            2 => Some(GameStatus::Completed),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl ComponentTrait for Position {
    fn component_type() -> Symbol {
        symbol_short!("position")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.x.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.y.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 8 {
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
        Some(Self { x, y })
    }
}

#[derive(Clone, Debug)]
pub struct Velocity {
    pub vx: i32,
    pub vy: i32,
}

impl ComponentTrait for Velocity {
    fn component_type() -> Symbol {
        symbol_short!("velocity")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.vx.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.vy.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 8 {
            return None;
        }
        let vx = i32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let vy = i32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);
        Some(Self { vx, vy })
    }
}

#[derive(Clone, Debug)]
pub struct Progress {
    pub distance: u32,
    pub score: u32,
    pub attempts: u32,
}

impl ComponentTrait for Progress {
    fn component_type() -> Symbol {
        symbol_short!("progress")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.distance.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.score.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.attempts.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 12 {
            return None;
        }
        let distance = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let score = u32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);
        let attempts = u32::from_be_bytes([
            data.get(8).unwrap(),
            data.get(9).unwrap(),
            data.get(10).unwrap(),
            data.get(11).unwrap(),
        ]);
        Some(Self {
            distance,
            score,
            attempts,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Obstacle {
    pub kind: ObstacleKind,
    pub trigger_mode: Option<PlayerMode>,
}

impl ComponentTrait for Obstacle {
    fn component_type() -> Symbol {
        symbol_short!("obstacle")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        let kind_val: u8 = match self.kind {
            ObstacleKind::Spike => 0,
            ObstacleKind::Block => 1,
            ObstacleKind::Portal => 2,
            ObstacleKind::Pad => 3,
        };
        bytes.append(&Bytes::from_array(env, &[kind_val]));
        let mode_val: u8 = match self.trigger_mode {
            None => 255,
            Some(PlayerMode::Cube) => 0,
            Some(PlayerMode::Ship) => 1,
            Some(PlayerMode::Wave) => 2,
            Some(PlayerMode::Ball) => 3,
        };
        bytes.append(&Bytes::from_array(env, &[mode_val]));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 2 {
            return None;
        }
        let kind = match data.get(0).unwrap() {
            0 => ObstacleKind::Spike,
            1 => ObstacleKind::Block,
            2 => ObstacleKind::Portal,
            3 => ObstacleKind::Pad,
            _ => return None,
        };
        let trigger_mode = match data.get(1).unwrap() {
            0 => Some(PlayerMode::Cube),
            1 => Some(PlayerMode::Ship),
            2 => Some(PlayerMode::Wave),
            3 => Some(PlayerMode::Ball),
            255 => None,
            _ => return None,
        };
        Some(Self { kind, trigger_mode })
    }
}
