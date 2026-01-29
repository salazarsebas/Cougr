#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env, Symbol};
// Cougr-Core ECS framework - demonstrates component and system patterns
//
// HOW COUGR-CORE SIMPLIFIES DEVELOPMENT VS VANILLA SOROBAN:
//
// 1. COMPONENT PATTERN: Instead of monolithic state structs, Cougr-Core organizes
//    game data into reusable components (PaddleComponent, BallComponent, ScoreComponent).
//    This makes code more modular and easier to extend with new features.
//
// 2. SYSTEM PATTERN: Game logic is organized into discrete systems (PhysicsSystem,
//    CollisionSystem, ScoringSystem) that operate on components. This separation of
//    concerns makes the codebase more maintainable and testable.
//
// 3. SCALABILITY: Adding new game features (e.g., power-ups, multiple balls) is as
//    simple as adding new components and systems, without refactoring existing code.
//
// 4. CLARITY: The ECS architecture makes it immediately clear what data exists
//    (components) and what operations are performed (systems), improving code readability.
//
// 5. REUSABILITY: Components and systems can be reused across different games,
//    reducing development time for future projects.
// Import Cougr-Core ECS types
use cougr_core::component::ComponentTrait;

// Game constants
const PADDLE_HEIGHT: i32 = 15;
const PADDLE_SPEED: i32 = 2;
const BALL_SPEED: i32 = 1;
const FIELD_WIDTH: i32 = 100;
const FIELD_HEIGHT: i32 = 60;
const WINNING_SCORE: u32 = 5;

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

const ECS_WORLD_KEY: Symbol = symbol_short!("ECSWORLD");

#[contract]
pub struct PongContract;

#[contractimpl]
impl PongContract {
    /// Initialize a new game using Cougr-Core ECS component pattern
    /// Demonstrates: Entity creation with components
    pub fn init_game(env: Env) -> GameState {
        // Create ECS world with entities and components
        // Following Cougr-Core pattern: each game object is an entity with components
        let world_state = ECSWorldState {
            // Entity 0: Player 1 Paddle
            player1_paddle: PaddleComponent {
                player_id: 1,
                y_position: FIELD_HEIGHT / 2,
            },
            // Entity 1: Player 2 Paddle
            player2_paddle: PaddleComponent {
                player_id: 2,
                y_position: FIELD_HEIGHT / 2,
            },
            // Entity 2: Ball
            ball: BallComponent {
                x: FIELD_WIDTH / 2,
                y: FIELD_HEIGHT / 2,
                vx: BALL_SPEED,
                vy: BALL_SPEED,
            },
            // Entity 3: Score
            score: ScoreComponent {
                player1_score: 0,
                player2_score: 0,
                game_active: true,
            },
        };

        env.storage().instance().set(&ECS_WORLD_KEY, &world_state);
        Self::world_to_game_state(&world_state)
    }

    /// Move a player's paddle
    /// Demonstrates: Component query and update pattern from Cougr-Core
    pub fn move_paddle(env: Env, player: u32, direction: i32) -> GameState {
        let mut world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&ECS_WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        if !world_state.score.game_active {
            return Self::world_to_game_state(&world_state);
        }

        // Query pattern: Find paddle component by player_id
        // This demonstrates Cougr-Core's component query approach
        let movement = direction * PADDLE_SPEED;

        if world_state.player1_paddle.player_id == player {
            // Update component
            let new_y = world_state.player1_paddle.y_position + movement;
            world_state.player1_paddle.y_position = Self::clamp_paddle_position(new_y);
        } else if world_state.player2_paddle.player_id == player {
            // Update component
            let new_y = world_state.player2_paddle.y_position + movement;
            world_state.player2_paddle.y_position = Self::clamp_paddle_position(new_y);
        }

        env.storage().instance().set(&ECS_WORLD_KEY, &world_state);
        Self::world_to_game_state(&world_state)
    }

    /// Update game tick - demonstrates Cougr-Core System pattern
    /// Systems: PhysicsSystem, CollisionSystem, ScoringSystem
    pub fn update_tick(env: Env) -> GameState {
        let mut world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&ECS_WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        if !world_state.score.game_active {
            return Self::world_to_game_state(&world_state);
        }

        // PhysicsSystem: Update ball position based on velocity
        Self::physics_system(&mut world_state);

        // CollisionSystem: Handle wall and paddle collisions
        Self::collision_system(&mut world_state);

        // ScoringSystem: Check for scoring and update scores
        Self::scoring_system(&mut world_state);

        env.storage().instance().set(&ECS_WORLD_KEY, &world_state);
        Self::world_to_game_state(&world_state)
    }

    /// Get current game state
    pub fn get_game_state(env: Env) -> GameState {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&ECS_WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        Self::world_to_game_state(&world_state)
    }

    /// Reset the game
    pub fn reset_game(env: Env) -> GameState {
        Self::init_game(env)
    }

    // ECS Systems - Following Cougr-Core System pattern

    /// PhysicsSystem: Updates ball position based on velocity
    /// Demonstrates: Cougr-Core System that operates on BallComponent
    fn physics_system(world: &mut ECSWorldState) {
        // Query ball component and update position
        world.ball.x += world.ball.vx;
        world.ball.y += world.ball.vy;
    }

    /// CollisionSystem: Handles all collision detection and response
    /// Demonstrates: Cougr-Core System that queries multiple components
    fn collision_system(world: &mut ECSWorldState) {
        let paddle_half_height = PADDLE_HEIGHT / 2;

        // Wall collision detection
        if world.ball.y <= 0 || world.ball.y >= FIELD_HEIGHT {
            world.ball.vy = -world.ball.vy;
            world.ball.y = world.ball.y.clamp(0, FIELD_HEIGHT);
        }

        // Paddle collision detection - Query paddle components
        // Left paddle (Player 1)
        if world.ball.x <= 5 && world.ball.vx < 0 {
            let paddle_top = world.player1_paddle.y_position - paddle_half_height;
            let paddle_bottom = world.player1_paddle.y_position + paddle_half_height;

            if world.ball.y >= paddle_top && world.ball.y <= paddle_bottom {
                world.ball.vx = -world.ball.vx;
                world.ball.x = 5;
            }
        }

        // Right paddle (Player 2)
        if world.ball.x >= FIELD_WIDTH - 5 && world.ball.vx > 0 {
            let paddle_top = world.player2_paddle.y_position - paddle_half_height;
            let paddle_bottom = world.player2_paddle.y_position + paddle_half_height;

            if world.ball.y >= paddle_top && world.ball.y <= paddle_bottom {
                world.ball.vx = -world.ball.vx;
                world.ball.x = FIELD_WIDTH - 5;
            }
        }
    }

    /// ScoringSystem: Handles scoring logic and win conditions
    /// Demonstrates: Cougr-Core System that updates ScoreComponent
    fn scoring_system(world: &mut ECSWorldState) {
        // Check if ball passed paddles
        if world.ball.x <= 0 {
            world.score.player2_score += 1;
            Self::reset_ball(&mut world.ball);
        } else if world.ball.x >= FIELD_WIDTH {
            world.score.player1_score += 1;
            Self::reset_ball(&mut world.ball);
        }

        // Check win condition
        if world.score.player1_score >= WINNING_SCORE || world.score.player2_score >= WINNING_SCORE
        {
            world.score.game_active = false;
        }
    }

    // Helper functions

    fn clamp_paddle_position(y: i32) -> i32 {
        let paddle_half_height = PADDLE_HEIGHT / 2;
        y.clamp(paddle_half_height, FIELD_HEIGHT - paddle_half_height)
    }

    fn reset_ball(ball: &mut BallComponent) {
        ball.x = FIELD_WIDTH / 2;
        ball.y = FIELD_HEIGHT / 2;
        ball.vx = -ball.vx;
    }

    /// Convert ECS World State to external GameState format
    fn world_to_game_state(world: &ECSWorldState) -> GameState {
        GameState {
            player1_paddle_y: world.player1_paddle.y_position,
            player2_paddle_y: world.player2_paddle.y_position,
            ball_x: world.ball.x,
            ball_y: world.ball.y,
            ball_vx: world.ball.vx,
            ball_vy: world.ball.vy,
            player1_score: world.score.player1_score,
            player2_score: world.score.player2_score,
            game_active: world.score.game_active,
        }
    }
}

#[cfg(test)]
mod test;
