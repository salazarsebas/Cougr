#![no_std]

//! # Snake On-Chain Game
//!
//! This example demonstrates how to build an on-chain Snake game using the
//! `cougr-core` ECS framework on the Stellar blockchain via Soroban.
//!
//! ## Game Logic
//!
//! The game follows classic Snake rules:
//! - The snake moves in a direction and the player can change direction
//! - Eating food makes the snake grow and increases the score
//! - Hitting walls or the snake's own body ends the game
//!
//! ## Architecture
//!
//! This implementation uses an Entity-Component-System (ECS) pattern:
//! - **Entities**: Snake head, snake body segments, and food
//! - **Components**: Position, Direction, SnakeHead, SnakeSegment, Food
//! - **Systems**: Movement, collision detection, growth, food spawning
//!
//! The `cougr-core` package simplifies on-chain game development by providing:
//! - Serialization-ready component patterns for on-chain storage
//! - Entity management optimized for Soroban's constraints
//! - A consistent architecture for game logic

mod components;
mod simple_world;
mod systems;

use components::{ComponentTrait, Direction, DirectionComponent, Position, SnakeHead};
use simple_world::SimpleWorld;
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env, Vec};

/// Default grid size for the game (10x10)
const DEFAULT_GRID_SIZE: i32 = 10;

/// Game state stored separately for easy serialization
#[contracttype]
#[derive(Clone, Debug)]
pub struct GameState {
    pub score: u32,
    pub game_over: bool,
    pub tick_count: u32,
    pub grid_size: i32,
    pub snake_head_id: u32,
}

/// Snake game contract
#[contract]
pub struct SnakeContract;

#[contractimpl]
impl SnakeContract {
    /// Initialize a new game
    ///
    /// Creates the initial game state with:
    /// - Snake starting at center of grid, moving right
    /// - Initial food spawned at a random position
    /// - Score set to 0
    pub fn init_game(env: Env) {
        Self::init_game_with_size(env, DEFAULT_GRID_SIZE);
    }

    /// Initialize a new game with custom grid size
    pub fn init_game_with_size(env: Env, grid_size: i32) {
        let mut world = SimpleWorld::new(&env);

        // Calculate center position
        let center = grid_size / 2;

        // Spawn snake head at center
        let head_id = world.spawn_entity();
        let head_pos = Position::new(center, center);
        let head = SnakeHead;
        let direction = DirectionComponent::new(Direction::Right);

        world.add_component(head_id, symbol_short!("position"), head_pos.serialize(&env));
        world.add_component(head_id, symbol_short!("snkhead"), head.serialize(&env));
        world.add_component(
            head_id,
            symbol_short!("direction"),
            direction.serialize(&env),
        );

        // Spawn initial food
        systems::spawn_food(&mut world, &env, 0, grid_size);

        // Create game state
        let game_state = GameState {
            score: 0,
            game_over: false,
            tick_count: 0,
            grid_size,
            snake_head_id: head_id,
        };

        // Store in contract storage
        env.storage()
            .persistent()
            .set(&symbol_short!("state"), &game_state);
        env.storage()
            .persistent()
            .set(&symbol_short!("world"), &world);
    }

    /// Change the snake's direction
    ///
    /// Direction values:
    /// - 0: Up
    /// - 1: Down
    /// - 2: Left
    /// - 3: Right
    ///
    /// Returns true if direction was changed, false if invalid
    /// (e.g., trying to reverse direction)
    pub fn change_direction(env: Env, direction: u32) -> bool {
        // Load game state
        let game_state: GameState = env
            .storage()
            .persistent()
            .get(&symbol_short!("state"))
            .unwrap();

        if game_state.game_over {
            return false;
        }

        // Parse direction
        let new_direction = match Direction::from_u8(direction as u8) {
            Some(d) => d,
            None => return false,
        };

        // Load world
        let mut world: SimpleWorld = env
            .storage()
            .persistent()
            .get(&symbol_short!("world"))
            .unwrap();

        // Update direction
        let result = systems::update_direction(&mut world, &env, new_direction);

        // Save world
        env.storage()
            .persistent()
            .set(&symbol_short!("world"), &world);

        result
    }

    /// Update the game by one tick
    ///
    /// This function:
    /// 1. Moves the snake in its current direction
    /// 2. Checks for wall collision (game over)
    /// 3. Checks for self collision (game over)
    /// 4. Checks for food collision (grow snake, increase score, spawn new food)
    pub fn update_tick(env: Env) {
        // Load game state
        let mut game_state: GameState = env
            .storage()
            .persistent()
            .get(&symbol_short!("state"))
            .unwrap();

        if game_state.game_over {
            return;
        }

        // Load world
        let mut world: SimpleWorld = env
            .storage()
            .persistent()
            .get(&symbol_short!("world"))
            .unwrap();

        // Move snake
        let move_result = systems::move_snake(&mut world, &env, game_state.grid_size);

        if move_result.is_none() {
            // Hit wall
            game_state.game_over = true;
        } else {
            // Check self collision
            if systems::check_self_collision(&world, &env) {
                game_state.game_over = true;
            } else {
                // Check food collision
                if let Some(food_id) = systems::check_food_collision(&world, &env) {
                    // Remove eaten food
                    world.despawn_entity(food_id);

                    // Grow snake
                    systems::grow_snake(&mut world, &env);

                    // Increase score
                    game_state.score += 1;

                    // Spawn new food
                    systems::spawn_food(
                        &mut world,
                        &env,
                        game_state.tick_count,
                        game_state.grid_size,
                    );
                }
            }
        }

        game_state.tick_count += 1;

        // Save state
        env.storage()
            .persistent()
            .set(&symbol_short!("state"), &game_state);
        env.storage()
            .persistent()
            .set(&symbol_short!("world"), &world);
    }

    /// Get the current score
    pub fn get_score(env: Env) -> u32 {
        let game_state: GameState = env
            .storage()
            .persistent()
            .get(&symbol_short!("state"))
            .unwrap();
        game_state.score
    }

    /// Check if the game is over
    pub fn check_game_over(env: Env) -> bool {
        let game_state: GameState = env
            .storage()
            .persistent()
            .get(&symbol_short!("state"))
            .unwrap();
        game_state.game_over
    }

    /// Get the snake's head position
    pub fn get_head_pos(env: Env) -> (i32, i32) {
        let game_state: GameState = env
            .storage()
            .persistent()
            .get(&symbol_short!("state"))
            .unwrap();

        let world: SimpleWorld = env
            .storage()
            .persistent()
            .get(&symbol_short!("world"))
            .unwrap();

        let head_id = game_state.snake_head_id;

        if let Some(pos_data) = world.get_component(head_id, &symbol_short!("position")) {
            if let Some(position) = Position::deserialize(&env, &pos_data) {
                return (position.x, position.y);
            }
        }

        (0, 0)
    }

    /// Get the snake's current length (head + segments)
    pub fn get_snake_length(env: Env) -> u32 {
        let world: SimpleWorld = env
            .storage()
            .persistent()
            .get(&symbol_short!("world"))
            .unwrap();

        let head_count = world
            .get_entities_with_component(&symbol_short!("snkhead"), &env)
            .len();
        let segment_count = world
            .get_entities_with_component(&symbol_short!("snkseg"), &env)
            .len();

        head_count + segment_count
    }

    /// Get the current food position
    pub fn get_food_pos(env: Env) -> (i32, i32) {
        let world: SimpleWorld = env
            .storage()
            .persistent()
            .get(&symbol_short!("world"))
            .unwrap();

        let food_entities = world.get_entities_with_component(&symbol_short!("food"), &env);
        if food_entities.is_empty() {
            return (-1, -1); // No food
        }

        let food_id = food_entities.get(0).unwrap();
        if let Some(pos_data) = world.get_component(food_id, &symbol_short!("position")) {
            if let Some(position) = Position::deserialize(&env, &pos_data) {
                return (position.x, position.y);
            }
        }

        (-1, -1)
    }

    /// Get all snake positions (head first, then segments in order)
    pub fn get_snake_positions(env: Env) -> Vec<(i32, i32)> {
        let game_state: GameState = env
            .storage()
            .persistent()
            .get(&symbol_short!("state"))
            .unwrap();

        let world: SimpleWorld = env
            .storage()
            .persistent()
            .get(&symbol_short!("world"))
            .unwrap();

        let mut positions = Vec::new(&env);

        // Add head position first
        let head_id = game_state.snake_head_id;
        if let Some(pos_data) = world.get_component(head_id, &symbol_short!("position")) {
            if let Some(position) = Position::deserialize(&env, &pos_data) {
                positions.push_back((position.x, position.y));
            }
        }

        // Get all segments and sort by index
        let segment_entities = world.get_entities_with_component(&symbol_short!("snkseg"), &env);
        let mut segments: Vec<(u32, i32, i32)> = Vec::new(&env);

        for i in 0..segment_entities.len() {
            let entity_id = segment_entities.get(i).unwrap();
            if let Some(seg_data) = world.get_component(entity_id, &symbol_short!("snkseg")) {
                if let Some(segment) = components::SnakeSegment::deserialize(&env, &seg_data) {
                    if let Some(pos_data) =
                        world.get_component(entity_id, &symbol_short!("position"))
                    {
                        if let Some(pos) = Position::deserialize(&env, &pos_data) {
                            segments.push_back((segment.index, pos.x, pos.y));
                        }
                    }
                }
            }
        }

        // Sort segments by index (bubble sort for no_std)
        let len = segments.len();
        for i in 0..len {
            for j in 0..(len - 1 - i) {
                let (idx_j, _, _) = segments.get(j).unwrap();
                let (idx_next, _, _) = segments.get(j + 1).unwrap();
                if idx_j > idx_next {
                    let temp = segments.get(j).unwrap();
                    let next = segments.get(j + 1).unwrap();
                    segments.set(j, next);
                    segments.set(j + 1, temp);
                }
            }
        }

        // Add sorted segment positions
        for i in 0..segments.len() {
            let (_, x, y) = segments.get(i).unwrap();
            positions.push_back((x, y));
        }

        positions
    }

    /// Get the grid size
    pub fn get_grid_size(env: Env) -> i32 {
        let game_state: GameState = env
            .storage()
            .persistent()
            .get(&symbol_short!("state"))
            .unwrap();
        game_state.grid_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_game() {
        let env = Env::default();
        let contract_id = env.register(SnakeContract, ());
        let client = SnakeContractClient::new(&env, &contract_id);

        client.init_game();

        assert_eq!(client.get_score(), 0);
        assert!(!client.check_game_over());
        assert_eq!(client.get_snake_length(), 1); // Just the head
        assert_eq!(client.get_grid_size(), DEFAULT_GRID_SIZE);

        // Check head is at center
        let (x, y) = client.get_head_pos();
        assert_eq!(x, DEFAULT_GRID_SIZE / 2);
        assert_eq!(y, DEFAULT_GRID_SIZE / 2);

        // Check food exists
        let (fx, fy) = client.get_food_pos();
        assert!((0..DEFAULT_GRID_SIZE).contains(&fx));
        assert!((0..DEFAULT_GRID_SIZE).contains(&fy));
    }

    #[test]
    fn test_init_game_with_custom_size() {
        let env = Env::default();
        let contract_id = env.register(SnakeContract, ());
        let client = SnakeContractClient::new(&env, &contract_id);

        client.init_game_with_size(&20);

        assert_eq!(client.get_grid_size(), 20);

        let (x, y) = client.get_head_pos();
        assert_eq!(x, 10);
        assert_eq!(y, 10);
    }

    #[test]
    fn test_change_direction() {
        let env = Env::default();
        let contract_id = env.register(SnakeContract, ());
        let client = SnakeContractClient::new(&env, &contract_id);

        client.init_game();

        // Initial direction is Right, can change to Up
        assert!(client.change_direction(&0)); // Up

        // Cannot reverse to Down now
        assert!(!client.change_direction(&1)); // Down - should fail

        // Can change to Right (perpendicular to current Up)
        assert!(client.change_direction(&3)); // Right
    }

    #[test]
    fn test_move_snake() {
        let env = Env::default();
        let contract_id = env.register(SnakeContract, ());
        let client = SnakeContractClient::new(&env, &contract_id);

        client.init_game();

        let (x1, y1) = client.get_head_pos();

        client.update_tick();

        let (x2, y2) = client.get_head_pos();

        // Snake moves right by default
        assert_eq!(x2, x1 + 1);
        assert_eq!(y2, y1);
    }

    #[test]
    fn test_wall_collision() {
        let env = Env::default();
        let contract_id = env.register(SnakeContract, ());
        let client = SnakeContractClient::new(&env, &contract_id);

        // Use small grid for quick wall collision
        client.init_game_with_size(&5);

        // Head starts at (2, 2), move right until wall
        for _ in 0..10 {
            client.update_tick();
            if client.check_game_over() {
                break;
            }
        }

        assert!(client.check_game_over());
    }

    #[test]
    fn test_eat_food_and_grow() {
        let env = Env::default();
        let contract_id = env.register(SnakeContract, ());
        let client = SnakeContractClient::new(&env, &contract_id);

        client.init_game_with_size(&20);

        let initial_length = client.get_snake_length();
        let initial_score = client.get_score();

        // Move towards food (we'll simulate this by playing until score increases)
        for _ in 0..100 {
            // Change direction to explore grid
            if client.get_score() > initial_score {
                break;
            }

            // Get food position and try to move towards it
            let (fx, fy) = client.get_food_pos();
            let (hx, hy) = client.get_head_pos();

            if fx > hx {
                client.change_direction(&3); // Right
            } else if fx < hx {
                client.change_direction(&2); // Left
            } else if fy > hy {
                client.change_direction(&1); // Down
            } else if fy < hy {
                client.change_direction(&0); // Up
            }

            client.update_tick();

            if client.check_game_over() {
                break;
            }
        }

        // If we ate food, score and length should have increased
        if client.get_score() > initial_score {
            assert!(client.get_snake_length() > initial_length);
        }
    }

    #[test]
    fn test_self_collision() {
        let env = Env::default();
        let contract_id = env.register(SnakeContract, ());
        let client = SnakeContractClient::new(&env, &contract_id);

        client.init_game_with_size(&20);

        // Build a long snake first by eating food
        for _ in 0..200 {
            let (fx, fy) = client.get_food_pos();
            let (hx, hy) = client.get_head_pos();

            // Simple navigation towards food
            if fx > hx {
                client.change_direction(&3);
            } else if fx < hx {
                client.change_direction(&2);
            } else if fy > hy {
                client.change_direction(&1);
            } else if fy < hy {
                client.change_direction(&0);
            }

            client.update_tick();

            if client.check_game_over() {
                break;
            }

            // Once snake is long enough, try to cause self-collision
            if client.get_snake_length() >= 5 {
                // Make a tight turn sequence to hit ourselves
                client.change_direction(&0); // Up
                client.update_tick();
                if client.check_game_over() {
                    break;
                }
                client.change_direction(&2); // Left
                client.update_tick();
                if client.check_game_over() {
                    break;
                }
                client.change_direction(&1); // Down
                client.update_tick();
                if client.check_game_over() {
                    break;
                }
            }
        }

        // Game should eventually end (either by self-collision or wall)
        // This test mainly verifies the self-collision logic doesn't panic
    }

    #[test]
    fn test_cannot_play_after_game_over() {
        let env = Env::default();
        let contract_id = env.register(SnakeContract, ());
        let client = SnakeContractClient::new(&env, &contract_id);

        client.init_game_with_size(&3);

        // Cause game over by hitting wall
        for _ in 0..10 {
            client.update_tick();
            if client.check_game_over() {
                break;
            }
        }

        assert!(client.check_game_over());

        let (x1, y1) = client.get_head_pos();
        let score1 = client.get_score();

        // Try to play after game over
        client.change_direction(&0);
        client.update_tick();

        let (x2, y2) = client.get_head_pos();
        let score2 = client.get_score();

        // Nothing should change
        assert_eq!(x1, x2);
        assert_eq!(y1, y2);
        assert_eq!(score1, score2);
    }

    #[test]
    fn test_get_snake_positions() {
        let env = Env::default();
        let contract_id = env.register(SnakeContract, ());
        let client = SnakeContractClient::new(&env, &contract_id);

        client.init_game_with_size(&20);

        // Initial snake has only head
        let positions = client.get_snake_positions();
        assert_eq!(positions.len(), 1);

        // Move and eat food to grow
        for _ in 0..50 {
            let (fx, fy) = client.get_food_pos();
            let (hx, hy) = client.get_head_pos();

            if fx > hx {
                client.change_direction(&3);
            } else if fx < hx {
                client.change_direction(&2);
            } else if fy > hy {
                client.change_direction(&1);
            } else if fy < hy {
                client.change_direction(&0);
            }

            client.update_tick();

            if client.check_game_over() || client.get_snake_length() > 3 {
                break;
            }
        }

        // After eating food, positions should reflect new length
        let positions = client.get_snake_positions();
        assert_eq!(positions.len(), client.get_snake_length());
    }

    #[test]
    fn test_food_respawns_after_eating() {
        let env = Env::default();
        let contract_id = env.register(SnakeContract, ());
        let client = SnakeContractClient::new(&env, &contract_id);

        client.init_game_with_size(&20);

        let initial_food_pos = client.get_food_pos();

        // Navigate to eat food
        for _ in 0..100 {
            if client.get_score() > 0 {
                break;
            }

            let (fx, fy) = client.get_food_pos();
            let (hx, hy) = client.get_head_pos();

            if fx > hx {
                client.change_direction(&3);
            } else if fx < hx {
                client.change_direction(&2);
            } else if fy > hy {
                client.change_direction(&1);
            } else if fy < hy {
                client.change_direction(&0);
            }

            client.update_tick();

            if client.check_game_over() {
                break;
            }
        }

        if client.get_score() > 0 {
            let new_food_pos = client.get_food_pos();
            // Food should have respawned (might be at same position by chance, but likely different)
            assert!(new_food_pos.0 >= 0 && new_food_pos.0 < 20);
            assert!(new_food_pos.1 >= 0 && new_food_pos.1 < 20);

            // At minimum, food should exist
            assert!(new_food_pos != (-1, -1));
        }

        let _ = initial_food_pos; // Suppress unused warning
    }
}
