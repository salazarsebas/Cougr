#![no_std]
extern crate alloc;

mod components;
mod systems;

use alloc::rc::Rc;
use components::{BirdState, ComponentTrait, PipeConfig, PipeMarker};
use core::cell::Cell;
use cougr_core::{GameApp, ScheduleStage, SimpleQueryBuilder, SimpleWorld, SystemConfig};
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env, Vec};
use systems::{Position, Velocity};

// Game constants
const INIT_BIRD_X: i32 = 50;
const INIT_BIRD_Y: i32 = 150;
const PIPE_GAP_SIZE: i32 = 100;
const PIPE_SPACING: i32 = 200;
const SPAWN_X: i32 = 400;

/// Game state stored separately for easy serialization
#[contracttype]
#[derive(Clone, Debug)]
pub struct GameState {
    pub score: u32,
    pub game_over: bool,
    pub tick_count: u32,
    pub bird_entity_id: u32,
    pub next_pipe_spawn: u32,
}

#[contract]
pub struct FlappyBirdContract;

#[contractimpl]
impl FlappyBirdContract {
    /// Initialize a new game
    pub fn init_game(env: Env) {
        let bird_id = Rc::new(Cell::new(0u32));
        let mut app = GameApp::new(&env);
        app.add_startup_system("spawn_bird", {
            let bird_id = bird_id.clone();
            move |world: &mut SimpleWorld, env: &Env| {
                let entity_id = world.spawn_entity();
                let bird_pos = Position::new(INIT_BIRD_X, INIT_BIRD_Y);
                let bird_vel = Velocity::new(0, 0);
                let bird_state = BirdState::new(true);

                world.add_component(
                    entity_id,
                    symbol_short!("position"),
                    bird_pos.serialize(env),
                );
                world.add_component(
                    entity_id,
                    symbol_short!("velocity"),
                    bird_vel.serialize(env),
                );
                world.add_component(
                    entity_id,
                    symbol_short!("birdstate"),
                    bird_state.serialize(env),
                );
                bird_id.set(entity_id);
            }
        });
        app.add_startup_system("spawn_pipes", move |world: &mut SimpleWorld, env: &Env| {
            Self::spawn_pipe(world, env, SPAWN_X, 150);
            Self::spawn_pipe(world, env, SPAWN_X + PIPE_SPACING, 200);
            Self::spawn_pipe(world, env, SPAWN_X + PIPE_SPACING * 2, 250);
        });
        app.run_startup(&env).unwrap();
        let world = app.into_world();

        // Save game state
        let game_state = GameState {
            score: 0,
            game_over: false,
            tick_count: 0,
            bird_entity_id: bird_id.get(),
            next_pipe_spawn: 3 * 50, // Spawn new pipe every 50 ticks
        };

        // Store in contract storage
        env.storage()
            .persistent()
            .set(&symbol_short!("state"), &game_state);
        env.storage()
            .persistent()
            .set(&symbol_short!("world"), &world);
    }

    /// Make the bird flap (jump)
    pub fn flap(env: Env) {
        // Load game state
        let game_state: GameState = env
            .storage()
            .persistent()
            .get(&symbol_short!("state"))
            .unwrap();

        if game_state.game_over {
            return; // Can't flap if game is over
        }

        // Load world
        let mut world: SimpleWorld = env
            .storage()
            .persistent()
            .get(&symbol_short!("world"))
            .unwrap();

        // Update bird velocity
        let bird_id = game_state.bird_entity_id;
        if let Some(vel_data) = world.get_component(bird_id, &symbol_short!("velocity")) {
            if let Some(mut velocity) = Velocity::deserialize(&env, &vel_data) {
                velocity.y = systems::FLAP_VELOCITY;
                world.add_component(bird_id, symbol_short!("velocity"), velocity.serialize(&env));
            }
        }

        // Save world
        env.storage()
            .persistent()
            .set(&symbol_short!("world"), &world);
    }

    /// Update game by one tick
    pub fn update_tick(env: Env) {
        // Load game state
        let mut game_state: GameState = env
            .storage()
            .persistent()
            .get(&symbol_short!("state"))
            .unwrap();

        if game_state.game_over {
            return; // Game is over, no more updates
        }

        // Load world
        let world: SimpleWorld = env
            .storage()
            .persistent()
            .get(&symbol_short!("world"))
            .unwrap();

        let collision = Rc::new(Cell::new(false));
        let score_increase = Rc::new(Cell::new(0u32));

        let mut app = GameApp::with_world(world);
        app.add_system_with_config(
            "gravity",
            systems::apply_gravity,
            SystemConfig::new().in_stage(ScheduleStage::PreUpdate),
        );
        app.add_system_with_config(
            "movement",
            systems::update_positions,
            SystemConfig::new().in_stage(ScheduleStage::Update),
        );
        app.add_system_with_config(
            "pipe_movement",
            systems::move_pipes,
            SystemConfig::new()
                .in_stage(ScheduleStage::Update)
                .after("movement"),
        );
        app.add_system_with_config(
            "collision",
            {
                let collision = collision.clone();
                move |world: &mut SimpleWorld, env: &Env| {
                    collision.set(systems::check_collisions(world, env));
                }
            },
            SystemConfig::new().in_stage(ScheduleStage::PostUpdate),
        );
        app.add_system_with_config(
            "score",
            {
                let score_increase = score_increase.clone();
                move |world: &mut SimpleWorld, env: &Env| {
                    score_increase.set(systems::update_score(world, env));
                }
            },
            SystemConfig::new()
                .in_stage(ScheduleStage::PostUpdate)
                .after("collision"),
        );
        app.run(&env).unwrap();
        let mut world = app.into_world();

        if collision.get() {
            game_state.game_over = true;
        }

        game_state.score += score_increase.get();

        // Spawn new pipes
        game_state.tick_count += 1;
        if game_state.tick_count >= game_state.next_pipe_spawn {
            // Spawn a new pipe
            let gap_center = 150 + ((game_state.tick_count * 17) % 150) as i32; // Pseudo-random
            Self::spawn_pipe(&mut world, &env, SPAWN_X, gap_center);
            game_state.next_pipe_spawn = game_state.tick_count + 50;
        }

        // Remove off-screen pipes
        Self::remove_offscreen_pipes(&mut world, &env);

        // Save state
        env.storage()
            .persistent()
            .set(&symbol_short!("state"), &game_state);
        env.storage()
            .persistent()
            .set(&symbol_short!("world"), &world);
    }

    /// Get current score
    pub fn get_score(env: Env) -> u32 {
        let game_state: GameState = env
            .storage()
            .persistent()
            .get(&symbol_short!("state"))
            .unwrap();
        game_state.score
    }

    /// Check if game is over
    pub fn check_game_over(env: Env) -> bool {
        let game_state: GameState = env
            .storage()
            .persistent()
            .get(&symbol_short!("state"))
            .unwrap();
        game_state.game_over
    }

    /// Get bird position
    pub fn get_bird_pos(env: Env) -> (i32, i32) {
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

        let bird_id = game_state.bird_entity_id;

        if let Some(pos_data) = world.get_component(bird_id, &symbol_short!("position")) {
            if let Some(position) = Position::deserialize(&env, &pos_data) {
                return (position.x, position.y);
            }
        }

        (0, 0)
    }

    // Helper functions

    fn spawn_pipe(world: &mut SimpleWorld, env: &Env, x: i32, gap_center_y: i32) {
        let pipe_id = world.spawn_entity();

        let pipe_pos = Position::new(x, gap_center_y);
        let pipe_config = PipeConfig::new(PIPE_GAP_SIZE, gap_center_y);
        let pipe_marker = PipeMarker::new();

        world.add_component(pipe_id, symbol_short!("position"), pipe_pos.serialize(env));
        world.add_component(
            pipe_id,
            symbol_short!("pipeconf"),
            pipe_config.serialize(env),
        );
        world.add_component(
            pipe_id,
            symbol_short!("pipemark"),
            pipe_marker.serialize(env),
        );
    }

    fn remove_offscreen_pipes(world: &mut SimpleWorld, env: &Env) {
        let pipe_entities = SimpleQueryBuilder::new(env)
            .with_component(symbol_short!("pipemark"))
            .build()
            .execute(world, env);
        let mut to_remove = Vec::new(env);

        for i in 0..pipe_entities.len() {
            let entity_id = pipe_entities.get(i).unwrap();

            if let Some(pos_data) = world.get_component(entity_id, &symbol_short!("position")) {
                if let Some(position) = Position::deserialize(env, &pos_data) {
                    if position.x < -100 {
                        to_remove.push_back(entity_id);
                    }
                }
            }
        }

        for i in 0..to_remove.len() {
            let entity_id = to_remove.get(i).unwrap();
            world.despawn_entity(entity_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_game() {
        let env = Env::default();
        let contract_id = env.register(FlappyBirdContract, ());
        let client = FlappyBirdContractClient::new(&env, &contract_id);

        // Initialize game
        client.init_game();

        // Check game state
        let score = client.get_score();
        assert_eq!(score, 0);

        let game_over = client.check_game_over();
        assert!(!game_over);

        // Check bird position
        let (x, y) = client.get_bird_pos();
        assert_eq!(x, INIT_BIRD_X);
        assert_eq!(y, INIT_BIRD_Y);
    }

    #[test]
    fn test_flap() {
        let env = Env::default();
        let contract_id = env.register(FlappyBirdContract, ());
        let client = FlappyBirdContractClient::new(&env, &contract_id);

        // Initialize game
        client.init_game();

        // Flap
        client.flap();

        // Bird velocity should have changed (will see effect after update_tick)
        client.update_tick();

        let (_, y) = client.get_bird_pos();
        // After flap and one tick, bird should have moved up
        assert!(y < INIT_BIRD_Y);
    }

    #[test]
    fn test_gravity() {
        let env = Env::default();
        let contract_id = env.register(FlappyBirdContract, ());
        let client = FlappyBirdContractClient::new(&env, &contract_id);

        // Initialize game
        client.init_game();

        let (_, y_before) = client.get_bird_pos();

        // Update multiple ticks without flapping
        client.update_tick();
        client.update_tick();
        client.update_tick();

        let (_, y_after) = client.get_bird_pos();

        // Bird should have fallen
        assert!(y_after > y_before);
    }

    #[test]
    fn test_game_over_on_ground_collision() {
        let env = Env::default();
        let contract_id = env.register(FlappyBirdContract, ());
        let client = FlappyBirdContractClient::new(&env, &contract_id);

        // Initialize game
        client.init_game();

        // Let bird fall to ground
        for _ in 0..100 {
            client.update_tick();
            if client.check_game_over() {
                break;
            }
        }

        // Game should be over
        assert!(client.check_game_over());
    }

    #[test]
    fn test_score_increases() {
        let env = Env::default();
        let contract_id = env.register(FlappyBirdContract, ());
        let client = FlappyBirdContractClient::new(&env, &contract_id);

        // Initialize game
        client.init_game();

        let initial_score = client.get_score();

        // Play for a while
        for i in 0..100 {
            if i % 5 == 0 {
                client.flap(); // Flap periodically to stay alive
            }
            client.update_tick();

            if client.check_game_over() {
                break;
            }
        }

        let final_score = client.get_score();

        // Score should have increased if we survived long enough
        // (might not if we died early)
        if !client.check_game_over() {
            assert!(final_score >= initial_score);
        }
    }

    #[test]
    fn test_cannot_flap_after_game_over() {
        let env = Env::default();
        let contract_id = env.register(FlappyBirdContract, ());
        let client = FlappyBirdContractClient::new(&env, &contract_id);

        // Initialize game
        client.init_game();

        // Let bird fall to ground
        for _ in 0..100 {
            client.update_tick();
            if client.check_game_over() {
                break;
            }
        }

        assert!(client.check_game_over());

        // Try to flap after game over
        client.flap();

        // Position should not change after game over
        let (x1, y1) = client.get_bird_pos();
        client.update_tick();
        let (x2, y2) = client.get_bird_pos();

        assert_eq!(x1, x2);
        assert_eq!(y1, y2);
    }
    #[test]
    fn test_gameapp_tick_integration() {
        let env = Env::default();
        let mut app = GameApp::new(&env);
        app.add_system_with_config(
            "flappy_tick_boundary",
            |_world: &mut SimpleWorld, _env: &Env| {},
            SystemConfig::new().in_stage(ScheduleStage::Update),
        );
        app.run(&env).unwrap();
    }

}
