#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env, Symbol};

// Cougr-Core ECS framework - demonstrates component and system patterns
//
// HOW COUGR-CORE SIMPLIFIES DEVELOPMENT VS VANILLA SOROBAN:
//
// 1. COMPONENT PATTERN: Instead of monolithic state structs, Cougr-Core organizes
//    game data into reusable components (PaddleComponent, BallComponent, BricksComponent).
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

use cougr_core::component::ComponentTrait;

// Game constants
const PADDLE_WIDTH: i32 = 15;
const PADDLE_SPEED: i32 = 3;
const BALL_SPEED: i32 = 1;
const FIELD_WIDTH: i32 = 100;
const FIELD_HEIGHT: i32 = 60;
const BRICK_COLS: usize = 10;
const BRICK_ROWS: usize = 5;
const BRICK_GRID_SIZE: usize = BRICK_COLS * BRICK_ROWS;
const STARTING_LIVES: u32 = 3;

/// Paddle component - demonstrates Cougr-Core Component pattern
#[contracttype]
#[derive(Clone, Debug)]
pub struct PaddleComponent {
    pub x_position: i32,
}

impl ComponentTrait for PaddleComponent {
    fn component_type() -> Symbol {
        symbol_short!("paddle")
    }

    fn serialize(&self, env: &Env) -> soroban_sdk::Bytes {
        let mut bytes = soroban_sdk::Bytes::new(env);
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.x_position.to_be_bytes(),
        ));
        bytes
    }

    fn deserialize(_env: &Env, data: &soroban_sdk::Bytes) -> Option<Self> {
        if data.len() != 4 {
            return None;
        }
        let x_position =
            i32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]);
        Some(Self { x_position })
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

/// Bricks component - demonstrates Cougr-Core Component pattern
/// Grid of bricks represented as a Vec (Soroban Vec for on-chain storage)
#[contracttype]
#[derive(Clone, Debug)]
pub struct BricksComponent {
    pub grid: soroban_sdk::Vec<bool>,
}

impl ComponentTrait for BricksComponent {
    fn component_type() -> Symbol {
        symbol_short!("bricks")
    }

    fn serialize(&self, env: &Env) -> soroban_sdk::Bytes {
        let mut bytes = soroban_sdk::Bytes::new(env);
        for i in 0..self.grid.len() {
            let brick = self.grid.get(i).unwrap_or(false);
            bytes.append(&soroban_sdk::Bytes::from_array(
                env,
                &[if brick { 1u8 } else { 0u8 }],
            ));
        }
        bytes
    }

    fn deserialize(env: &Env, data: &soroban_sdk::Bytes) -> Option<Self> {
        let mut grid = soroban_sdk::Vec::new(env);
        for i in 0..data.len() {
            grid.push_back(data.get(i)? != 0);
        }
        Some(Self { grid })
    }
}

/// Score component - demonstrates Cougr-Core Component pattern
#[contracttype]
#[derive(Clone, Debug)]
pub struct ScoreComponent {
    pub score: u32,
    pub lives: u32,
    pub game_active: bool,
}

impl ComponentTrait for ScoreComponent {
    fn component_type() -> Symbol {
        symbol_short!("score")
    }

    fn serialize(&self, env: &Env) -> soroban_sdk::Bytes {
        let mut bytes = soroban_sdk::Bytes::new(env);
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.score.to_be_bytes(),
        ));
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.lives.to_be_bytes(),
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
        let score = u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]);
        let lives = u32::from_be_bytes([data.get(4)?, data.get(5)?, data.get(6)?, data.get(7)?]);
        let game_active = data.get(8)? != 0;
        Some(Self {
            score,
            lives,
            game_active,
        })
    }
}

/// ECS World State - demonstrates how Cougr-Core organizes game data into components
#[contracttype]
#[derive(Clone, Debug)]
pub struct ECSWorldState {
    pub paddle: PaddleComponent,
    pub ball: BallComponent,
    pub bricks: BricksComponent,
    pub score: ScoreComponent,
}

/// Game state for external API
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    pub paddle_x: i32,
    pub ball_x: i32,
    pub ball_y: i32,
    pub ball_vx: i32,
    pub ball_vy: i32,
    pub score: u32,
    pub lives: u32,
    pub game_active: bool,
    pub bricks_remaining: u32,
}

const ECS_WORLD_KEY: Symbol = symbol_short!("ECSWORLD");

#[contract]
pub struct ArkanoidContract;

#[contractimpl]
impl ArkanoidContract {
    /// Initialize a new game using Cougr-Core ECS component pattern
    pub fn init_game(env: Env) -> GameState {
        // Initialize bricks grid
        let mut bricks_grid = soroban_sdk::Vec::new(&env);
        for _ in 0..BRICK_GRID_SIZE {
            bricks_grid.push_back(true);
        }

        let world_state = ECSWorldState {
            paddle: PaddleComponent {
                x_position: FIELD_WIDTH / 2,
            },
            ball: BallComponent {
                x: FIELD_WIDTH / 2,
                y: FIELD_HEIGHT - 10,
                vx: BALL_SPEED,
                vy: -BALL_SPEED,
            },
            bricks: BricksComponent { grid: bricks_grid },
            score: ScoreComponent {
                score: 0,
                lives: STARTING_LIVES,
                game_active: true,
            },
        };

        env.storage().instance().set(&ECS_WORLD_KEY, &world_state);
        Self::world_to_game_state(&world_state)
    }

    /// Move the paddle
    pub fn move_paddle(env: Env, direction: i32) -> GameState {
        let mut world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&ECS_WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        if !world_state.score.game_active {
            return Self::world_to_game_state(&world_state);
        }

        let movement = direction * PADDLE_SPEED;
        let new_x = world_state.paddle.x_position + movement;
        world_state.paddle.x_position = Self::clamp_paddle_position(new_x);

        env.storage().instance().set(&ECS_WORLD_KEY, &world_state);
        Self::world_to_game_state(&world_state)
    }

    /// Update game tick - demonstrates Cougr-Core System pattern
    pub fn update_tick(env: Env) -> GameState {
        let mut world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&ECS_WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        if !world_state.score.game_active {
            return Self::world_to_game_state(&world_state);
        }

        // PhysicsSystem: Update ball position
        Self::physics_system(&mut world_state);

        // CollisionSystem: Handle all collisions
        Self::collision_system(&mut world_state);

        // ScoringSystem: Check win/loss conditions
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

    /// Check if game is over
    pub fn check_game_over(env: Env) -> bool {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&ECS_WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        !world_state.score.game_active
    }

    // ECS Systems - Following Cougr-Core System pattern

    /// PhysicsSystem: Updates ball position based on velocity
    fn physics_system(world: &mut ECSWorldState) {
        world.ball.x += world.ball.vx;
        world.ball.y += world.ball.vy;
    }

    /// CollisionSystem: Handles all collision detection and response
    fn collision_system(world: &mut ECSWorldState) {
        let paddle_half_width = PADDLE_WIDTH / 2;

        // Top wall collision
        if world.ball.y <= 0 {
            world.ball.vy = -world.ball.vy;
            world.ball.y = 0;
        }

        // Side wall collisions
        if world.ball.x <= 0 {
            world.ball.vx = -world.ball.vx;
            world.ball.x = 0;
        } else if world.ball.x >= FIELD_WIDTH {
            world.ball.vx = -world.ball.vx;
            world.ball.x = FIELD_WIDTH;
        }

        // Bottom wall collision (lose life)
        if world.ball.y >= FIELD_HEIGHT && world.score.lives > 0 {
            world.score.lives -= 1;
            Self::reset_ball(&mut world.ball);
            if world.score.lives == 0 {
                world.score.game_active = false;
            }
        }

        // Paddle collision
        if world.ball.y >= FIELD_HEIGHT - 5 && world.ball.vy > 0 {
            let paddle_left = world.paddle.x_position - paddle_half_width;
            let paddle_right = world.paddle.x_position + paddle_half_width;

            if world.ball.x >= paddle_left && world.ball.x <= paddle_right {
                world.ball.vy = -world.ball.vy;
                world.ball.y = FIELD_HEIGHT - 5;
            }
        }

        // Brick collisions
        Self::check_brick_collisions(world);
    }

    /// ScoringSystem: Handles scoring logic and win conditions
    fn scoring_system(world: &mut ECSWorldState) {
        // Check if all bricks are broken (win condition)
        let bricks_remaining = world.bricks.grid.iter().filter(|&b| b).count();
        if bricks_remaining == 0 {
            world.score.game_active = false;
        }
    }

    // Helper functions

    fn clamp_paddle_position(x: i32) -> i32 {
        let paddle_half_width = PADDLE_WIDTH / 2;
        x.clamp(paddle_half_width, FIELD_WIDTH - paddle_half_width)
    }

    fn reset_ball(ball: &mut BallComponent) {
        ball.x = FIELD_WIDTH / 2;
        ball.y = FIELD_HEIGHT - 10;
        ball.vx = BALL_SPEED;
        ball.vy = -BALL_SPEED;
    }

    fn check_brick_collisions(world: &mut ECSWorldState) {
        let brick_width = FIELD_WIDTH / BRICK_COLS as i32;
        let brick_height = 3;
        let brick_area_height = BRICK_ROWS as i32 * brick_height;

        // Only check if ball is in brick area
        if world.ball.y < 0 || world.ball.y > brick_area_height {
            return;
        }

        // Calculate which brick the ball might be hitting
        let col = (world.ball.x / brick_width).clamp(0, (BRICK_COLS - 1) as i32) as usize;
        let row = (world.ball.y / brick_height).clamp(0, (BRICK_ROWS - 1) as i32) as usize;
        let index = row * BRICK_COLS + col;

        if index < BRICK_GRID_SIZE {
            if let Some(brick_present) = world.bricks.grid.get(index as u32) {
                if brick_present {
                    // Break the brick
                    world.bricks.grid.set(index as u32, false);
                    world.score.score += 10;

                    // Bounce the ball
                    world.ball.vy = -world.ball.vy;
                }
            }
        }
    }

    fn world_to_game_state(world: &ECSWorldState) -> GameState {
        let bricks_remaining = world.bricks.grid.iter().filter(|&b| b).count() as u32;

        GameState {
            paddle_x: world.paddle.x_position,
            ball_x: world.ball.x,
            ball_y: world.ball.y,
            ball_vx: world.ball.vx,
            ball_vy: world.ball.vy,
            score: world.score.score,
            lives: world.score.lives,
            game_active: world.score.game_active,
            bricks_remaining,
        }
    }
}

#[cfg(test)]
mod test;
