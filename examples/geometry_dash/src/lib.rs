#![no_std]

mod components;
mod systems;

#[cfg(test)]
mod test;

use components::{GameStatus, Obstacle, ObstacleKind, PlayerMode, Position, Progress, Velocity};
use cougr_core::{ComponentTrait, SimpleWorld};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct GeometryDashContract;

#[contractimpl]
impl GeometryDashContract {
    /// Initialize a new game level
    pub fn init_game(env: Env, player: Address, _level_id: u32) {
        let mut world = SimpleWorld::new(&env);

        let id = world.spawn_entity();

        let pos = Position {
            x: 0,
            y: systems::GROUND_Y,
        };
        let vel = Velocity {
            vx: systems::TICK_MOVEMENT_X,
            vy: 0,
        };
        let mode = PlayerMode::Cube;
        let progress = Progress {
            distance: 0,
            score: 0,
            attempts: 1,
        };
        let status = GameStatus::Playing;

        world.add_component(id, symbol_short!("position"), pos.serialize(&env));
        world.add_component(id, symbol_short!("velocity"), vel.serialize(&env));
        world.add_component(id, symbol_short!("mode"), mode.serialize(&env));
        world.add_component(id, symbol_short!("progress"), progress.serialize(&env));
        world.add_component(id, symbol_short!("status"), status.serialize(&env));

        // Spawn some obstacles for testing
        spawn_obstacle(
            &mut world,
            &env,
            100_000,
            systems::GROUND_Y,
            ObstacleKind::Spike,
            None,
        );
        spawn_obstacle(
            &mut world,
            &env,
            300_000,
            systems::GROUND_Y,
            ObstacleKind::Portal,
            Some(PlayerMode::Ship),
        );

        storage_set_world(&env, &player, &world);
    }

    /// Get current game state
    pub fn get_state(env: Env, player: Address) -> GameStatus {
        let world = storage_get_world(&env, &player);
        let entities = world.get_entities_with_component(&symbol_short!("status"), &env);
        if entities.is_empty() {
            return GameStatus::Crashed;
        }
        let id = entities.get(0).unwrap();
        let data = world.get_component(id, &symbol_short!("status")).unwrap();
        GameStatus::deserialize(&env, &data).unwrap()
    }

    /// Trigger player jump/action
    pub fn jump(env: Env, player: Address) {
        let mut world = storage_get_world(&env, &player);

        // Process input
        systems::input_system(&mut world, &env, true);

        storage_set_world(&env, &player, &world);
    }

    /// Update game by one tick
    pub fn update_tick(env: Env, player: Address) {
        let mut world = storage_get_world(&env, &player);
        let entities = world.get_entities_with_component(&symbol_short!("status"), &env);

        // Check if already crushed
        if !entities.is_empty() {
            let id = entities.get(0).unwrap();
            let data = world.get_component(id, &symbol_short!("status")).unwrap();
            let status = GameStatus::deserialize(&env, &data).unwrap();
            if status != GameStatus::Playing {
                return;
            }
        }

        // Advance simulation
        systems::movement_system(&mut world, &env);

        // Check collisions
        if systems::collision_system(&mut world, &env) && !entities.is_empty() {
            let id = entities.get(0).unwrap();
            world.add_component(
                id,
                symbol_short!("status"),
                GameStatus::Crashed.serialize(&env),
            );
        }

        // Handle mode transitions
        systems::mode_system(&mut world, &env);

        systems::progress_system(&mut world, &env);

        storage_set_world(&env, &player, &world);
    }

    /// Get current player position (useful for testing)
    pub fn get_pos(env: Env, player: Address) -> (i32, i32) {
        let world = storage_get_world(&env, &player);
        let entities = world.get_entities_with_component(&symbol_short!("position"), &env);
        if entities.is_empty() {
            return (0, 0);
        }
        let id = entities.get(0).unwrap();
        let data = world.get_component(id, &symbol_short!("position")).unwrap();
        let pos = Position::deserialize(&env, &data).unwrap();
        (pos.x, pos.y)
    }

    /// Get current score
    pub fn get_score(env: Env, player: Address) -> u32 {
        let world = storage_get_world(&env, &player);
        let entities = world.get_entities_with_component(&symbol_short!("progress"), &env);
        if entities.is_empty() {
            return 0;
        }
        let id = entities.get(0).unwrap();
        let data = world.get_component(id, &symbol_short!("progress")).unwrap();
        let prog = Progress::deserialize(&env, &data).unwrap();
        prog.score
    }

    /// Get current player mode (0: Cube, 1: Ship, 2: Wave, 3: Ball)
    pub fn get_mode(env: Env, player: Address) -> u32 {
        let world = storage_get_world(&env, &player);
        let entities = world.get_entities_with_component(&symbol_short!("mode"), &env);
        if entities.is_empty() {
            return 0;
        }
        let id = entities.get(0).unwrap();
        let data = world.get_component(id, &symbol_short!("mode")).unwrap();
        let mode = PlayerMode::deserialize(&env, &data).unwrap();
        match mode {
            PlayerMode::Cube => 0,
            PlayerMode::Ship => 1,
            PlayerMode::Wave => 2,
            PlayerMode::Ball => 3,
        }
    }
}

fn spawn_obstacle(
    world: &mut SimpleWorld,
    env: &Env,
    x: i32,
    y: i32,
    kind: ObstacleKind,
    trigger_mode: Option<PlayerMode>,
) {
    let id = world.spawn_entity();
    let pos = Position { x, y };
    let obs = Obstacle { kind, trigger_mode };
    world.add_component(id, symbol_short!("position"), pos.serialize(env));
    world.add_component(id, symbol_short!("obstacle"), obs.serialize(env));
}

fn storage_get_world(env: &Env, player: &Address) -> SimpleWorld {
    env.storage()
        .persistent()
        .get(player)
        .expect("Game not initialized")
}

fn storage_set_world(env: &Env, player: &Address, world: &SimpleWorld) {
    env.storage().persistent().set(player, world);
}
