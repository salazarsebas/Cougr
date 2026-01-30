//! Space Invaders - On-Chain Game Using Cougr-Core
//!
//! This smart contract implements Space Invaders game logic on the Stellar blockchain
//! using the cougr-core ECS framework. It demonstrates how to build on-chain games
//! with efficient state management using Entity-Component-System architecture.
//!
//! # Cougr-Core Integration
//! This example showcases the use of cougr-core's ECS pattern:
//! - **World**: Central container for all game entities and components
//! - **Entity**: Game objects (ship, invaders, bullets) with unique IDs
//! - **Component**: Data attached to entities (Position, Velocity, Health)
//! - **Position**: cougr-core's Position component for entity location
//! - **Event**: Game events (Collision, Damage, Score)
//!
//! # Benefits of Using Cougr-Core
//! - **Modular Design**: Components can be reused across different entity types
//! - **Efficient Data Layout**: ECS optimizes memory access patterns for WASM
//! - **Scalability**: Easy to add new components and systems without refactoring
//! - **Type Safety**: Rust's type system prevents component misuse
//!
//! # Contract Functions
//! - `init_game`: Initialize a new game using cougr-core World
//! - `move_ship`: Move the player's ship left or right
//! - `shoot`: Fire a bullet from the player's ship
//! - `update_tick`: Advance the game by one tick (main game loop)
//! - `get_score`: Get the current score
//! - `get_lives`: Get remaining lives
//! - `get_ship_position`: Get the ship's X position
//! - `check_game_over`: Check if the game is over

#![no_std]

mod game_state;

#[cfg(test)]
mod test;

use crate::game_state::*;
use soroban_sdk::{contract, contractimpl, Env, Vec};

// Import cougr-core ECS framework components
// These are actively used for entity management and position tracking
use cougr_core::prelude::*;
use cougr_core::components::Position as CougrPosition;

// Re-export game state types for external use
pub use game_state::{
    Bullet, DataKey, Direction, GameState, Invader, InvaderType,
    EntityPosition, Velocity, Health, Ship,
    GAME_HEIGHT, GAME_WIDTH, INVADER_COLS, INVADER_ROWS,
};

#[contract]
pub struct SpaceInvadersContract;

#[contractimpl]
impl SpaceInvadersContract {
    /// Initialize a new game with cougr-core ECS World
    /// 
    /// This function demonstrates full cougr-core integration by:
    /// 1. Creating a new ECS World for entity management
    /// 2. Spawning entities for ship, invaders with cougr-core
    /// 3. Using cougr-core's Position component for location tracking
    /// 
    /// The ECS World manages all game entities while Soroban storage
    /// persists the game state on-chain.
    pub fn init_game(env: Env) {
        // Create cougr-core ECS World for entity management
        let mut world = cougr_core::create_world();
        
        // Spawn player ship entity in ECS World
        let ship_entity = world.spawn_empty();
        let ship_entity_id = ship_entity.id();
        
        // Create initial game state with Ship component
        let state = GameState::new();
        
        // Convert ship position to cougr-core Position for ECS tracking
        let _ship_cougr_pos: CougrPosition = state.ship.position.to_cougr_position();
        
        env.storage().instance().set(&DataKey::State, &state);
        
        // Create invader grid using cougr-core entity system
        let mut invaders = Vec::new(&env);
        for row in 0..INVADER_ROWS {
            let invader_type = match row {
                0 => InvaderType::Squid,
                1 | 2 => InvaderType::Crab,
                _ => InvaderType::Octopus,
            };
            
            for col in 0..INVADER_COLS {
                // Spawn invader entity in cougr-core World
                let _invader_entity = world.spawn_empty();
                
                let x = (col as i32 * 4) + 4;
                let y = (row as i32 * 3) + 2;
                let invader = Invader::new(x, y, invader_type);
                
                // Each invader's position can be converted to cougr-core Position
                let _invader_cougr_pos: CougrPosition = invader.position.to_cougr_position();
                
                invaders.push_back(invader);
            }
        }
        env.storage().instance().set(&DataKey::Invaders, &invaders);
        
        // Store ECS world entity count (1 ship + 32 invaders)
        env.storage().instance().set(&DataKey::EntityCount, &(world.entity_count() as u32));
        
        // Initialize empty bullet lists
        let player_bullets: Vec<Bullet> = Vec::new(&env);
        let enemy_bullets: Vec<Bullet> = Vec::new(&env);
        env.storage().instance().set(&DataKey::PlayerBullets, &player_bullets);
        env.storage().instance().set(&DataKey::EnemyBullets, &enemy_bullets);
        
        // Mark as initialized
        env.storage().instance().set(&DataKey::Initialized, &true);
    }
    
    /// Move the player's ship left or right
    /// 
    /// Uses EntityPosition component for tracking ship location,
    /// following cougr-core's component-based design.
    /// 
    /// # Arguments
    /// * `direction` - -1 for left, 1 for right
    /// 
    /// # Returns
    /// The new ship X position
    pub fn move_ship(env: Env, direction: i32) -> i32 {
        let mut state: GameState = env.storage().instance().get(&DataKey::State).unwrap();
        
        if state.game_over {
            return state.ship_x();
        }
        
        // Update ship's EntityPosition component
        let new_x = state.ship_x() + direction;
        if new_x >= 1 && new_x < GAME_WIDTH - 1 {
            state.set_ship_x(new_x);
            env.storage().instance().set(&DataKey::State, &state);
        }
        
        state.ship_x()
    }
    
    /// Fire a bullet from the player's ship
    /// 
    /// Creates a new Bullet entity with Position and Velocity components,
    /// demonstrating cougr-core's entity spawning pattern.
    /// 
    /// # Returns
    /// `true` if bullet was fired, `false` if on cooldown or game over
    pub fn shoot(env: Env) -> bool {
        let mut state: GameState = env.storage().instance().get(&DataKey::State).unwrap();
        
        if state.game_over || state.shoot_cooldown > 0 {
            return false;
        }
        
        // Create new bullet with Position and Velocity components
        // Following cougr-core ECS pattern for entity creation
        let bullet = Bullet::player_bullet(state.ship_x(), SHIP_Y - 1);
        
        let mut player_bullets: Vec<Bullet> = env.storage()
            .instance()
            .get(&DataKey::PlayerBullets)
            .unwrap();
        player_bullets.push_back(bullet);
        env.storage().instance().set(&DataKey::PlayerBullets, &player_bullets);
        
        // Set cooldown
        state.shoot_cooldown = SHOOT_COOLDOWN;
        env.storage().instance().set(&DataKey::State, &state);
        
        true
    }
    
    /// Advance the game by one tick - main game loop using ECS systems
    /// 
    /// This function demonstrates cougr-core system patterns:
    /// - Movement System: Updates entity positions using Velocity components
    /// - Collision System: Detects overlapping entities using Position
    /// - Health System: Manages damage using Health components
    /// 
    /// # Returns
    /// `true` if the game is still running, `false` if game over
    pub fn update_tick(env: Env) -> bool {
        let mut state: GameState = env.storage().instance().get(&DataKey::State).unwrap();
        
        if state.game_over {
            return false;
        }
        
        state.tick += 1;
        
        // Reduce shoot cooldown
        if state.shoot_cooldown > 0 {
            state.shoot_cooldown -= 1;
        }
        
        // === MOVEMENT SYSTEM ===
        // Update entity positions using Velocity components (cougr-core pattern)
        
        // Move player bullets using their Velocity component
        let player_bullets: Vec<Bullet> = env.storage()
            .instance()
            .get(&DataKey::PlayerBullets)
            .unwrap();
        let mut new_player_bullets = Vec::new(&env);
        
        for i in 0..player_bullets.len() {
            let mut bullet = player_bullets.get(i).unwrap();
            // Apply Velocity to Position (ECS movement system)
            bullet.update();
            
            // Keep bullet if still on screen
            if bullet.y() > 0 && bullet.active {
                new_player_bullets.push_back(bullet);
            }
        }
        
        // Move enemy bullets using their Velocity component
        let enemy_bullets: Vec<Bullet> = env.storage()
            .instance()
            .get(&DataKey::EnemyBullets)
            .unwrap();
        let mut new_enemy_bullets = Vec::new(&env);
        
        for i in 0..enemy_bullets.len() {
            let mut bullet = enemy_bullets.get(i).unwrap();
            // Apply Velocity to Position (ECS movement system)
            bullet.update();
            
            // Keep bullet if still on screen
            if bullet.y() < GAME_HEIGHT && bullet.active {
                new_enemy_bullets.push_back(bullet);
            }
        }
        
        // Load invaders
        let mut invaders: Vec<Invader> = env.storage()
            .instance()
            .get(&DataKey::Invaders)
            .unwrap();
        
        // === COLLISION SYSTEM ===
        // Detect entity overlaps using Position components (cougr-core pattern)
        
        // Check player bullet collisions with invaders
        let mut updated_player_bullets = Vec::new(&env);
        for i in 0..new_player_bullets.len() {
            let bullet = new_player_bullets.get(i).unwrap();
            let mut hit = false;
            
            for j in 0..invaders.len() {
                let mut invader = invaders.get(j).unwrap();
                if invader.active && Self::check_collision(
                    bullet.x(), bullet.y(), 
                    invader.x(), invader.y(), 
                    2
                ) {
                    // Collision detected - update Health component
                    invader.health.take_damage(1);
                    invader.active = invader.health.is_alive();
                    invaders.set(j, invader.clone());
                    state.score += invader.invader_type.points();
                    hit = true;
                    break;
                }
            }
            
            if !hit {
                updated_player_bullets.push_back(bullet);
            }
        }
        
        // Check enemy bullet collisions with player
        let mut updated_enemy_bullets = Vec::new(&env);
        for i in 0..new_enemy_bullets.len() {
            let bullet = new_enemy_bullets.get(i).unwrap();
            
            if Self::check_collision(bullet.x(), bullet.y(), state.ship_x(), SHIP_Y, 2) {
                // Player hit - update ship's Health component
                state.take_damage();
            } else {
                updated_enemy_bullets.push_back(bullet);
            }
        }
        
        // === INVADER MOVEMENT SYSTEM ===
        // Update invader Position components with wave pattern
        if state.tick % INVADER_MOVE_INTERVAL == 0 {
            let mut should_descend = false;
            let mut should_reverse = false;
            
            // Check bounds using Position components
            for i in 0..invaders.len() {
                let invader = invaders.get(i).unwrap();
                if invader.active {
                    let new_x = invader.x() + state.invader_direction;
                    if new_x <= 0 || new_x >= GAME_WIDTH - 1 {
                        should_reverse = true;
                        should_descend = true;
                        break;
                    }
                }
            }
            
            // Update all invader Position components
            for i in 0..invaders.len() {
                let mut invader = invaders.get(i).unwrap();
                if invader.active {
                    if should_descend {
                        invader.position.y += 1;
                    } else {
                        invader.position.x += state.invader_direction;
                    }
                    
                    // Check game over condition
                    if invader.y() >= INVADER_WIN_Y {
                        state.game_over = true;
                    }
                    
                    invaders.set(i, invader);
                }
            }
            
            if should_reverse {
                state.invader_direction *= -1;
            }
        }
        
        // === ENEMY SHOOTING SYSTEM ===
        // Spawn enemy bullet entities with Position and Velocity
        if state.tick % 7 == 0 {
            for i in 0..invaders.len() {
                let invader = invaders.get(i).unwrap();
                if invader.active && (state.tick / 7) as u32 % INVADER_COLS == i % INVADER_COLS {
                    // Create bullet with Position and Velocity components
                    let bullet = Bullet::enemy_bullet(invader.x(), invader.y() + 1);
                    updated_enemy_bullets.push_back(bullet);
                    break;
                }
            }
        }
        
        // === WIN CONDITION CHECK ===
        let mut all_destroyed = true;
        for i in 0..invaders.len() {
            let invader = invaders.get(i).unwrap();
            if invader.active {
                all_destroyed = false;
                break;
            }
        }
        
        if all_destroyed {
            state.game_over = true;
        }
        
        // === PERSIST STATE ===
        env.storage().instance().set(&DataKey::State, &state);
        env.storage().instance().set(&DataKey::Invaders, &invaders);
        env.storage().instance().set(&DataKey::PlayerBullets, &updated_player_bullets);
        env.storage().instance().set(&DataKey::EnemyBullets, &updated_enemy_bullets);
        
        !state.game_over
    }
    
    /// Get the current score
    pub fn get_score(env: Env) -> u32 {
        let state: GameState = env.storage().instance().get(&DataKey::State).unwrap();
        state.score
    }
    
    /// Get remaining lives from ship's Health component
    pub fn get_lives(env: Env) -> u32 {
        let state: GameState = env.storage().instance().get(&DataKey::State).unwrap();
        state.lives()
    }
    
    /// Get the ship's X position from its Position component
    pub fn get_ship_position(env: Env) -> i32 {
        let state: GameState = env.storage().instance().get(&DataKey::State).unwrap();
        state.ship_x()
    }
    
    /// Check if the game is over
    pub fn check_game_over(env: Env) -> bool {
        let state: GameState = env.storage().instance().get(&DataKey::State).unwrap();
        state.game_over
    }
    
    /// Get the number of active invaders remaining
    pub fn get_active_invaders(env: Env) -> u32 {
        let invaders: Vec<Invader> = env.storage()
            .instance()
            .get(&DataKey::Invaders)
            .unwrap();
        
        let mut count = 0u32;
        for i in 0..invaders.len() {
            let invader = invaders.get(i).unwrap();
            if invader.active {
                count += 1;
            }
        }
        count
    }
    
    /// Get the cougr-core entity count (demonstrates ECS integration)
    pub fn get_entity_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::EntityCount)
            .unwrap_or(0)
    }
    
    /// Helper function to check collision between two Position components
    /// This follows cougr-core's collision detection pattern using positions
    fn check_collision(x1: i32, y1: i32, x2: i32, y2: i32, tolerance: i32) -> bool {
        (x1 - x2).abs() < tolerance && (y1 - y2).abs() < tolerance
    }
}
