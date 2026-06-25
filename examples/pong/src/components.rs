use cougr_core::component::ComponentTrait;
use soroban_sdk::{contracttype, symbol_short, Env, Symbol};

/// Paddle component - demonstrates Cougr-Core Component pattern
#[contracttype]
#[derive(Clone, Debug)]
pub struct PaddleComponent {
    pub player_id: u32,
    pub y_position: i32,
}

// Implement Cougr-Core ComponentTrait for PaddleComponent
impl ComponentTrait for PaddleComponent {
    fn component_type() -> Symbol {
        symbol_short!("paddle")
    }

    fn serialize(&self, env: &Env) -> soroban_sdk::Bytes {
        let mut bytes = soroban_sdk::Bytes::new(env);
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.player_id.to_be_bytes(),
        ));
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.y_position.to_be_bytes(),
        ));
        bytes
    }

    fn deserialize(_env: &Env, data: &soroban_sdk::Bytes) -> Option<Self> {
        if data.len() != 8 {
            return None;
        }
        let player_id =
            u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]);
        let y_position =
            i32::from_be_bytes([data.get(4)?, data.get(5)?, data.get(6)?, data.get(7)?]);
        Some(Self {
            player_id,
            y_position,
        })
    }
}

/// Ball component - demonstrates Cougr-Core Component pattern
#[contracttype]
#[derive(Clone, Debug)]
pub struct BallComponent {
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
}

// Implement Cougr-Core ComponentTrait for BallComponent
impl ComponentTrait for BallComponent {
    fn component_type() -> Symbol {
        symbol_short!("ball")
    }

    fn serialize(&self, env: &Env) -> soroban_sdk::Bytes {
        let mut bytes = soroban_sdk::Bytes::new(env);
        bytes.append(&soroban_sdk::Bytes::from_array(env, &self.x.to_be_bytes()));
        bytes.append(&soroban_sdk::Bytes::from_array(env, &self.y.to_be_bytes()));
        bytes.append(&soroban_sdk::Bytes::from_array(env, &self.vx.to_be_bytes()));
        bytes.append(&soroban_sdk::Bytes::from_array(env, &self.vy.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, data: &soroban_sdk::Bytes) -> Option<Self> {
        if data.len() != 16 {
            return None;
        }
        let x = i32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]);
        let y = i32::from_be_bytes([data.get(4)?, data.get(5)?, data.get(6)?, data.get(7)?]);
        let vx = i32::from_be_bytes([data.get(8)?, data.get(9)?, data.get(10)?, data.get(11)?]);
        let vy = i32::from_be_bytes([data.get(12)?, data.get(13)?, data.get(14)?, data.get(15)?]);
        Some(Self { x, y, vx, vy })
    }
}

/// Score component - demonstrates Cougr-Core Component pattern
#[contracttype]
#[derive(Clone, Debug)]
pub struct ScoreComponent {
    pub player1_score: u32,
    pub player2_score: u32,
    pub game_active: bool,
}

// Implement Cougr-Core ComponentTrait for ScoreComponent
impl ComponentTrait for ScoreComponent {
    fn component_type() -> Symbol {
        symbol_short!("score")
    }

    fn serialize(&self, env: &Env) -> soroban_sdk::Bytes {
        let mut bytes = soroban_sdk::Bytes::new(env);
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.player1_score.to_be_bytes(),
        ));
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.player2_score.to_be_bytes(),
        ));
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &[if self.game_active { 1u8 } else { 0u8 }],
        ));
        bytes
    }

    fn deserialize(_env: &Env, data: &soroban_sdk::Bytes) -> Option<Self> {
        if data.len() != 9 {
            return None;
        }
        let player1_score =
            u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]);
        let player2_score =
            u32::from_be_bytes([data.get(4)?, data.get(5)?, data.get(6)?, data.get(7)?]);
        let game_active = data.get(8)? != 0;
        Some(Self {
            player1_score,
            player2_score,
            game_active,
        })
    }
}

/// ECS World State - serializable version using Cougr-Core component pattern
/// This demonstrates how Cougr-Core organizes game data into components
#[contracttype]
#[derive(Clone, Debug)]
pub struct ECSWorldState {
    /// Entity 0: Player 1 Paddle with PaddleComponent
    pub player1_paddle: PaddleComponent,
    /// Entity 1: Player 2 Paddle with PaddleComponent  
    pub player2_paddle: PaddleComponent,
    /// Entity 2: Ball with BallComponent
    pub ball: BallComponent,
    /// Entity 3: Game Score with ScoreComponent
    pub score: ScoreComponent,
}

/// Game state for external API
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    pub player1_paddle_y: i32,
    pub player2_paddle_y: i32,
    pub ball_x: i32,
    pub ball_y: i32,
    pub ball_vx: i32,
    pub ball_vy: i32,
    pub player1_score: u32,
    pub player2_score: u32,
    pub game_active: bool,
}

