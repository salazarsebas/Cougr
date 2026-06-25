use crate::components::*;
use cougr_core::{ComponentTrait, SimpleWorld};
use soroban_sdk::{symbol_short, Env};

// Physics constants (scaled by 1000)
pub const TICK_MOVEMENT_X: i32 = 10000; // 10 units per tick
pub const GRAVITY_CUBE: i32 = 1500;
pub const JUMP_FORCE_CUBE: i32 = -25000;

pub const GRAVITY_SHIP: i32 = 800;
pub const LIFT_FORCE_SHIP: i32 = -2000;

pub const GRAVITY_BALL: i32 = 1500;
pub const SWITCH_FORCE_BALL: i32 = -30000;

pub const WAVE_OSCILLATION: i32 = 12000;

pub const GROUND_Y: i32 = 400_000;
pub const CEILING_Y: i32 = 0;

/// MovementSystem - advances the player forward each tick, applies gravity and velocity based on current mode
pub fn movement_system(world: &mut SimpleWorld, env: &Env) {
    let entities_with_vel = world.get_entities_with_component(&symbol_short!("velocity"), env);

    for i in 0..entities_with_vel.len() {
        let id = entities_with_vel.get(i).unwrap();

        if let (Some(pos_data), Some(vel_data), Some(mode_data)) = (
            world.get_component(id, &symbol_short!("position")),
            world.get_component(id, &symbol_short!("velocity")),
            world.get_component(id, &symbol_short!("mode")),
        ) {
            if let (Some(mut pos), Some(mut vel), Some(mode)) = (
                Position::deserialize(env, &pos_data),
                Velocity::deserialize(env, &vel_data),
                PlayerMode::deserialize(env, &mode_data),
            ) {
                // 1. Advance X (Fixed distance)
                pos.x += TICK_MOVEMENT_X;

                // 2. Apply Mode-Specific Gravity/Physics
                match mode {
                    PlayerMode::Cube => {
                        vel.vy += GRAVITY_CUBE;
                    }
                    PlayerMode::Ship => {
                        vel.vy += GRAVITY_SHIP;
                    }
                    PlayerMode::Ball => {
                        // Gravity direction depends on ball state (not implemented yet, default for now)
                        vel.vy += GRAVITY_BALL;
                    }
                    PlayerMode::Wave => {
                        // Wave physics applied in InputSystem mainly (direction of oscillation)
                    }
                }

                // 3. Apply Y Velocity to Position
                pos.y += vel.vy;

                // 4. Clamp Ground/Ceiling
                if pos.y >= GROUND_Y {
                    pos.y = GROUND_Y;
                    vel.vy = 0;
                } else if pos.y <= CEILING_Y {
                    pos.y = CEILING_Y;
                    vel.vy = 0;
                }

                // Update components
                world.add_component(id, symbol_short!("position"), pos.serialize(env));
                world.add_component(id, symbol_short!("velocity"), vel.serialize(env));
            }
        }
    }
}

/// InputSystem - processes jump/action input, adjusts velocity according to mode physics
pub fn input_system(world: &mut SimpleWorld, env: &Env, is_jumping: bool) {
    let entities_with_vel = world.get_entities_with_component(&symbol_short!("velocity"), env);

    for i in 0..entities_with_vel.len() {
        let id = entities_with_vel.get(i).unwrap();

        if let (Some(pos_data), Some(vel_data), Some(mode_data)) = (
            world.get_component(id, &symbol_short!("position")),
            world.get_component(id, &symbol_short!("velocity")),
            world.get_component(id, &symbol_short!("mode")),
        ) {
            if let (Some(pos), Some(mut vel), Some(mode)) = (
                Position::deserialize(env, &pos_data),
                Velocity::deserialize(env, &vel_data),
                PlayerMode::deserialize(env, &mode_data),
            ) {
                if is_jumping {
                    match mode {
                        PlayerMode::Cube => {
                            // Only jump if on ground
                            if pos.y >= GROUND_Y {
                                vel.vy = JUMP_FORCE_CUBE;
                            }
                        }
                        PlayerMode::Ship => {
                            // Apply lift while jumping
                            vel.vy += LIFT_FORCE_SHIP;
                        }
                        PlayerMode::Wave => {
                            // Move up while jumping
                            vel.vy = -WAVE_OSCILLATION;
                        }
                        PlayerMode::Ball => {
                            // Switch gravity
                            if vel.vy.abs() < 1000 {
                                // Only switch if on ground/ceiling approx
                                vel.vy = if vel.vy >= 0 {
                                    SWITCH_FORCE_BALL
                                } else {
                                    -SWITCH_FORCE_BALL
                                };
                            }
                        }
                    }
                } else {
                    if mode == PlayerMode::Wave {
                        // Move down while not jumping
                        vel.vy = WAVE_OSCILLATION;
                    }
                }

                world.add_component(id, symbol_short!("velocity"), vel.serialize(env));
            }
        }
    }
}

pub fn collision_system(world: &mut SimpleWorld, env: &Env) -> bool {
    let entities_with_pos = world.get_entities_with_component(&symbol_short!("position"), env);
    if entities_with_pos.is_empty() {
        return false;
    }

    let id = entities_with_pos.get(0).unwrap();
    let pos_data = world.get_component(id, &symbol_short!("position")).unwrap();
    let pos = Position::deserialize(env, &pos_data).unwrap();

    // Collision with ground/ceiling handled in MovementSystem
    // Here we check for spikes/blocks
    let obstacle_entities = world.get_entities_with_component(&symbol_short!("obstacle"), env);
    for i in 0..obstacle_entities.len() {
        let obs_id = obstacle_entities.get(i).unwrap();
        if let (Some(obs_pos_data), Some(obs_data)) = (
            world.get_component(obs_id, &symbol_short!("position")),
            world.get_component(obs_id, &symbol_short!("obstacle")),
        ) {
            let obs_pos = Position::deserialize(env, &obs_pos_data).unwrap();
            let obs = Obstacle::deserialize(env, &obs_data).unwrap();

            // Simple AABB collision
            if (pos.x - obs_pos.x).abs() < 5000
                && (pos.y - obs_pos.y).abs() < 5000
                && (obs.kind == ObstacleKind::Spike || obs.kind == ObstacleKind::Block)
            {
                return true;
            }
        }
    }
    false
}

pub fn progress_system(world: &mut SimpleWorld, env: &Env) {
    let entities_with_progress = world.get_entities_with_component(&symbol_short!("progress"), env);

    for i in 0..entities_with_progress.len() {
        let id = entities_with_progress.get(i).unwrap();

        if let (Some(pos_data), Some(prog_data)) = (
            world.get_component(id, &symbol_short!("position")),
            world.get_component(id, &symbol_short!("progress")),
        ) {
            if let (Some(pos), Some(mut prog)) = (
                Position::deserialize(env, &pos_data),
                Progress::deserialize(env, &prog_data),
            ) {
                prog.distance = (pos.x / 1000) as u32;
                prog.score = prog.distance; // Score is distance for now
                world.add_component(id, symbol_short!("progress"), prog.serialize(env));
            }
        }
    }
}

pub fn mode_system(world: &mut SimpleWorld, env: &Env) {
    let entities_with_pos = world.get_entities_with_component(&symbol_short!("position"), env);
    if entities_with_pos.is_empty() {
        return;
    }

    let id = entities_with_pos.get(0).unwrap();
    let pos_data = world.get_component(id, &symbol_short!("position")).unwrap();
    let pos = Position::deserialize(env, &pos_data).unwrap();

    let obstacle_entities = world.get_entities_with_component(&symbol_short!("obstacle"), env);
    for i in 0..obstacle_entities.len() {
        let obs_id = obstacle_entities.get(i).unwrap();
        if let (Some(obs_pos_data), Some(obs_data)) = (
            world.get_component(obs_id, &symbol_short!("position")),
            world.get_component(obs_id, &symbol_short!("obstacle")),
        ) {
            let obs_pos = Position::deserialize(env, &obs_pos_data).unwrap();
            let obs = Obstacle::deserialize(env, &obs_data).unwrap();

            // Collision with portal (Portals have larger vertical range)
            let x_match = (pos.x - obs_pos.x).abs() < 10000;
            let y_match = if obs.kind == ObstacleKind::Portal {
                (pos.y - obs_pos.y).abs() < 400_000
            } else {
                (pos.y - obs_pos.y).abs() < 5000
            };

            if x_match && y_match && obs.kind == ObstacleKind::Portal {
                if let Some(new_mode) = obs.trigger_mode {
                    world.add_component(id, symbol_short!("mode"), new_mode.serialize(env));
                }
            }
        }
    }
}

/// Exercises the Cougr `GameApp` tick path for this transitional example.
#[cfg(test)]
pub fn run_gameapp_tick(env: &Env) {
    use cougr_core::{GameApp, ScheduleStage, SystemConfig};

    let mut app = GameApp::new(env);
    app.add_system_with_config(
        "geometry_dash_tick_boundary",
        |_world: &mut SimpleWorld, _env: &Env| {},
        SystemConfig::new().in_stage(ScheduleStage::Update),
    );
    app.run(env).unwrap();
}
