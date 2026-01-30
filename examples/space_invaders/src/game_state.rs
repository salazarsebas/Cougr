//! Game state structures for Space Invaders
//! 
//! This module defines all the data structures needed to represent
//! the game state on-chain using Soroban's storage.
//!
//! **Cougr-Core Integration**: This module demonstrates how to use
//! cougr-core's ECS components for game entity data management.

use soroban_sdk::contracttype;

// Import cougr-core Position component for entity position tracking
// This demonstrates proper integration of cougr-core into game logic
pub use cougr_core::components::Position as CougrPosition;

/// Direction for ship movement
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Direction {
    Left = 0,
    Right = 1,
}

/// Type of invader (affects points and behavior)
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum InvaderType {
    /// Top row invaders - 30 points
    Squid = 0,
    /// Middle row invaders - 20 points
    Crab = 1,
    /// Bottom row invaders - 10 points
    Octopus = 2,
}

impl InvaderType {
    /// Get points for destroying this invader type
    pub fn points(&self) -> u32 {
        match self {
            InvaderType::Squid => 30,
            InvaderType::Crab => 20,
            InvaderType::Octopus => 10,
        }
    }
}

/// Entity position component - wraps cougr-core's Position
/// This demonstrates the ECS component pattern from cougr-core
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityPosition {
    /// X coordinate on the game grid
    pub x: i32,
    /// Y coordinate on the game grid
    pub y: i32,
}

impl EntityPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    
    /// Convert to cougr-core Position for ECS integration
    pub fn to_cougr_position(&self) -> CougrPosition {
        CougrPosition {
            x: self.x as u32,
            y: self.y as u32,
        }
    }
    
    /// Create from cougr-core Position
    pub fn from_cougr_position(pos: &CougrPosition) -> Self {
        Self {
            x: pos.x as i32,
            y: pos.y as i32,
        }
    }
}

/// Velocity component for moving entities
/// Follows cougr-core's ECS component pattern
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Velocity {
    /// Horizontal velocity
    pub dx: i32,
    /// Vertical velocity  
    pub dy: i32,
}

impl Velocity {
    pub fn new(dx: i32, dy: i32) -> Self {
        Self { dx, dy }
    }
    
    /// Apply velocity to a position (movement system pattern)
    pub fn apply_to(&self, pos: &mut EntityPosition) {
        pos.x += self.dx;
        pos.y += self.dy;
    }
}

/// Health component for entities that can be damaged
/// Follows cougr-core's ECS component pattern
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Health {
    /// Current health points
    pub current: u32,
    /// Maximum health points
    pub max: u32,
}

impl Health {
    pub fn new(max: u32) -> Self {
        Self { current: max, max }
    }
    
    pub fn take_damage(&mut self, amount: u32) {
        if self.current > amount {
            self.current -= amount;
        } else {
            self.current = 0;
        }
    }
    
    pub fn is_alive(&self) -> bool {
        self.current > 0
    }
}

/// Represents a single invader in the grid
/// Uses ECS component pattern with Position and Health
#[contracttype]
#[derive(Clone, Debug)]
pub struct Invader {
    /// Position component (cougr-core pattern)
    pub position: EntityPosition,
    /// Type of invader
    pub invader_type: InvaderType,
    /// Health component (cougr-core pattern)
    pub health: Health,
    /// Whether the invader is still alive
    pub active: bool,
}

impl Invader {
    pub fn new(x: i32, y: i32, invader_type: InvaderType) -> Self {
        Self {
            position: EntityPosition::new(x, y),
            invader_type,
            health: Health::new(1),
            active: true,
        }
    }
    
    /// Get X position (convenience accessor)
    pub fn x(&self) -> i32 {
        self.position.x
    }
    
    /// Get Y position (convenience accessor)
    pub fn y(&self) -> i32 {
        self.position.y
    }
}

/// Represents a bullet (player or enemy)
/// Uses ECS component pattern with Position and Velocity
#[contracttype]
#[derive(Clone, Debug)]
pub struct Bullet {
    /// Position component (cougr-core pattern)
    pub position: EntityPosition,
    /// Velocity component (cougr-core pattern)
    pub velocity: Velocity,
    /// Whether the bullet is still active
    pub active: bool,
}

impl Bullet {
    pub fn new(x: i32, y: i32, direction: i32) -> Self {
        Self {
            position: EntityPosition::new(x, y),
            velocity: Velocity::new(0, direction * BULLET_SPEED),
            active: true,
        }
    }
    
    /// Create a player bullet (moves up)
    pub fn player_bullet(x: i32, y: i32) -> Self {
        Self::new(x, y, -1)
    }
    
    /// Create an enemy bullet (moves down)
    pub fn enemy_bullet(x: i32, y: i32) -> Self {
        Self::new(x, y, 1)
    }
    
    /// Update bullet position using velocity component
    pub fn update(&mut self) {
        self.velocity.apply_to(&mut self.position);
    }
    
    /// Get X position (convenience accessor)
    pub fn x(&self) -> i32 {
        self.position.x
    }
    
    /// Get Y position (convenience accessor)
    pub fn y(&self) -> i32 {
        self.position.y
    }
}

/// Player ship entity with ECS components
#[contracttype]
#[derive(Clone, Debug)]
pub struct Ship {
    /// Position component (cougr-core pattern)
    pub position: EntityPosition,
    /// Health component (lives)
    pub health: Health,
}

impl Ship {
    pub fn new(x: i32, y: i32, lives: u32) -> Self {
        Self {
            position: EntityPosition::new(x, y),
            health: Health::new(lives),
        }
    }
}

/// Main game state structure
#[contracttype]
#[derive(Clone, Debug)]
pub struct GameState {
    /// Player's ship entity with components
    pub ship: Ship,
    /// Player's current score
    pub score: u32,
    /// Whether the game is over
    pub game_over: bool,
    /// Current invader movement direction (1 = right, -1 = left)
    pub invader_direction: i32,
    /// Current game tick (for pacing)
    pub tick: u32,
    /// Cooldown for player shooting (ticks until can shoot again)
    pub shoot_cooldown: u32,
}

impl GameState {
    /// Create a new game state with default values
    pub fn new() -> Self {
        Self {
            ship: Ship::new(GAME_WIDTH / 2, SHIP_Y, 3),
            score: 0,
            game_over: false,
            invader_direction: 1,
            tick: 0,
            shoot_cooldown: 0,
        }
    }
    
    /// Get ship X position (backwards compatibility)
    pub fn ship_x(&self) -> i32 {
        self.ship.position.x
    }
    
    /// Set ship X position (backwards compatibility)
    pub fn set_ship_x(&mut self, x: i32) {
        self.ship.position.x = x;
    }
    
    /// Get remaining lives
    pub fn lives(&self) -> u32 {
        self.ship.health.current
    }
    
    /// Take damage (lose a life)
    pub fn take_damage(&mut self) {
        self.ship.health.take_damage(1);
        if !self.ship.health.is_alive() {
            self.game_over = true;
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

// Game constants
/// Width of the game board
pub const GAME_WIDTH: i32 = 40;
/// Height of the game board
pub const GAME_HEIGHT: i32 = 30;
/// Number of invader columns
pub const INVADER_COLS: u32 = 8;
/// Number of invader rows
pub const INVADER_ROWS: u32 = 4;
/// Ship's Y position (fixed at bottom)
pub const SHIP_Y: i32 = GAME_HEIGHT - 2;
/// Y position where invaders cause game over
pub const INVADER_WIN_Y: i32 = SHIP_Y - 2;
/// Points needed for extra life
pub const EXTRA_LIFE_SCORE: u32 = 1000;
/// Shoot cooldown in ticks
pub const SHOOT_COOLDOWN: u32 = 3;
/// Bullet speed (positions per tick)
pub const BULLET_SPEED: i32 = 2;
/// Invader movement speed (ticks between moves)
pub const INVADER_MOVE_INTERVAL: u32 = 5;

/// Storage keys for Soroban persistent storage
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Main game state
    State,
    /// List of invaders
    Invaders,
    /// List of player bullets
    PlayerBullets,
    /// List of enemy bullets  
    EnemyBullets,
    /// Flag indicating if game has been initialized
    Initialized,
    /// Count of cougr-core entities (demonstrates ECS integration)
    EntityCount,
    /// Cougr-core World state (serialized)
    WorldState,
}
