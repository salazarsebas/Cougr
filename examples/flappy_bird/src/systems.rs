use crate::components::{BirdState, ComponentTrait, PipeConfig, PipeMarker};
use crate::simple_world::SimpleWorld;
use soroban_sdk::{symbol_short, Bytes, Env, Symbol};

// Define our own Position and Velocity types that match cougr-core's but with ComponentTrait
#[derive(Clone, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl ComponentTrait for Position {
    fn component_type() -> Symbol {
        symbol_short!("position")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        let x_bytes = Bytes::from_array(env, &self.x.to_be_bytes());
        let y_bytes = Bytes::from_array(env, &self.y.to_be_bytes());
        bytes.append(&x_bytes);
        bytes.append(&y_bytes);
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
    pub x: i32,
    pub y: i32,
}

impl Velocity {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl ComponentTrait for Velocity {
    fn component_type() -> Symbol {
        symbol_short!("velocity")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        let x_bytes = Bytes::from_array(env, &self.x.to_be_bytes());
        let y_bytes = Bytes::from_array(env, &self.y.to_be_bytes());
        bytes.append(&x_bytes);
        bytes.append(&y_bytes);
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

// Game constants
pub const GRAVITY: i32 = 2;
pub const FLAP_VELOCITY: i32 = -15;
pub const PIPE_SPEED: i32 = 3;
pub const GROUND_Y: i32 = 400;
pub const BIRD_SIZE: i32 = 20;
pub const PIPE_WIDTH: i32 = 50;

/// Apply gravity to bird velocity
pub fn apply_gravity(world: &mut SimpleWorld, env: &Env) {
    // Find bird entity (entity with BirdState component)
    let bird_entities = world.get_entities_with_component(&symbol_short!("birdstate"), env);

    for i in 0..bird_entities.len() {
        let entity_id = bird_entities.get(i).unwrap();

        // Get current velocity
        if let Some(vel_data) = world.get_component(entity_id, &symbol_short!("velocity")) {
            if let Some(mut velocity) = Velocity::deserialize(env, &vel_data) {
                // Apply gravity
                velocity.y += GRAVITY;

                // Update component
                world.add_component(
                    entity_id,
                    symbol_short!("velocity"),
                    velocity.serialize(env),
                );
            }
        }
    }
}

/// Update positions based on velocities
pub fn update_positions(world: &mut SimpleWorld, env: &Env) {
    let entities_with_velocity = world.get_entities_with_component(&symbol_short!("velocity"), env);

    for i in 0..entities_with_velocity.len() {
        let entity_id = entities_with_velocity.get(i).unwrap();

        // Check if entity has both position and velocity
        if world.has_component(entity_id, &symbol_short!("position")) {
            if let (Some(pos_data), Some(vel_data)) = (
                world.get_component(entity_id, &symbol_short!("position")),
                world.get_component(entity_id, &symbol_short!("velocity")),
            ) {
                if let (Some(mut position), Some(velocity)) = (
                    Position::deserialize(env, &pos_data),
                    Velocity::deserialize(env, &vel_data),
                ) {
                    // Update position
                    position.x += velocity.x;
                    position.y += velocity.y;

                    // Update component
                    world.add_component(
                        entity_id,
                        symbol_short!("position"),
                        position.serialize(env),
                    );
                }
            }
        }
    }
}

/// Move pipes left
pub fn move_pipes(world: &mut SimpleWorld, env: &Env) {
    let pipe_entities = world.get_entities_with_component(&symbol_short!("pipemark"), env);

    for i in 0..pipe_entities.len() {
        let entity_id = pipe_entities.get(i).unwrap();

        if let Some(pos_data) = world.get_component(entity_id, &symbol_short!("position")) {
            if let Some(mut position) = Position::deserialize(env, &pos_data) {
                // Move left
                position.x -= PIPE_SPEED;

                // Update component
                world.add_component(
                    entity_id,
                    symbol_short!("position"),
                    position.serialize(env),
                );
            }
        }
    }
}

/// Check collisions between bird and pipes/ground
pub fn check_collisions(world: &mut SimpleWorld, env: &Env) -> bool {
    // Find bird position and state
    let bird_entities = world.get_entities_with_component(&symbol_short!("birdstate"), env);
    if bird_entities.is_empty() {
        return false;
    }

    let bird_id = bird_entities.get(0).unwrap();
    let bird_pos = match world.get_component(bird_id, &symbol_short!("position")) {
        Some(data) => match Position::deserialize(env, &data) {
            Some(pos) => pos,
            None => return false,
        },
        None => return false,
    };

    // Check ground collision
    if bird_pos.y >= GROUND_Y - BIRD_SIZE {
        if let Some(state_data) = world.get_component(bird_id, &symbol_short!("birdstate")) {
            if let Some(mut bird_state) = BirdState::deserialize(env, &state_data) {
                bird_state.is_alive = false;
                world.add_component(
                    bird_id,
                    symbol_short!("birdstate"),
                    bird_state.serialize(env),
                );
                return true;
            }
        }
    }

    // Check ceiling collision
    if bird_pos.y <= BIRD_SIZE {
        if let Some(state_data) = world.get_component(bird_id, &symbol_short!("birdstate")) {
            if let Some(mut bird_state) = BirdState::deserialize(env, &state_data) {
                bird_state.is_alive = false;
                world.add_component(
                    bird_id,
                    symbol_short!("birdstate"),
                    bird_state.serialize(env),
                );
                return true;
            }
        }
    }

    // Check pipe collisions
    let pipe_entities = world.get_entities_with_component(&symbol_short!("pipemark"), env);
    for i in 0..pipe_entities.len() {
        let pipe_id = pipe_entities.get(i).unwrap();

        if let (Some(pipe_pos_data), Some(pipe_config_data)) = (
            world.get_component(pipe_id, &symbol_short!("position")),
            world.get_component(pipe_id, &symbol_short!("pipeconf")),
        ) {
            if let (Some(pipe_pos), Some(pipe_config)) = (
                Position::deserialize(env, &pipe_pos_data),
                PipeConfig::deserialize(env, &pipe_config_data),
            ) {
                // Check if bird is within pipe's x range
                if bird_pos.x + BIRD_SIZE > pipe_pos.x
                    && bird_pos.x - BIRD_SIZE < pipe_pos.x + PIPE_WIDTH
                {
                    // Check if bird is outside the gap
                    let gap_top = pipe_config.gap_center_y - pipe_config.gap_size / 2;
                    let gap_bottom = pipe_config.gap_center_y + pipe_config.gap_size / 2;

                    if bird_pos.y - BIRD_SIZE < gap_top || bird_pos.y + BIRD_SIZE > gap_bottom {
                        // Collision with pipe
                        if let Some(state_data) =
                            world.get_component(bird_id, &symbol_short!("birdstate"))
                        {
                            if let Some(mut bird_state) = BirdState::deserialize(env, &state_data) {
                                bird_state.is_alive = false;
                                world.add_component(
                                    bird_id,
                                    symbol_short!("birdstate"),
                                    bird_state.serialize(env),
                                );
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

/// Update score when bird passes pipes
pub fn update_score(world: &mut SimpleWorld, env: &Env) -> u32 {
    let mut score_increase = 0;

    // Find bird position
    let bird_entities = world.get_entities_with_component(&symbol_short!("birdstate"), env);
    if bird_entities.is_empty() {
        return 0;
    }

    let bird_id = bird_entities.get(0).unwrap();
    let bird_x = match world.get_component(bird_id, &symbol_short!("position")) {
        Some(data) => match Position::deserialize(env, &data) {
            Some(pos) => pos.x,
            None => return 0,
        },
        None => return 0,
    };

    // Check each pipe
    let pipe_entities = world.get_entities_with_component(&symbol_short!("pipemark"), env);
    for i in 0..pipe_entities.len() {
        let pipe_id = pipe_entities.get(i).unwrap();

        if let (Some(marker_data), Some(pos_data)) = (
            world.get_component(pipe_id, &symbol_short!("pipemark")),
            world.get_component(pipe_id, &symbol_short!("position")),
        ) {
            if let (Some(mut marker), Some(pipe_pos)) = (
                PipeMarker::deserialize(env, &marker_data),
                Position::deserialize(env, &pos_data),
            ) {
                // If bird passed pipe and it wasn't marked yet
                if !marker.passed && bird_x > pipe_pos.x + PIPE_WIDTH {
                    marker.passed = true;
                    score_increase += 1;

                    // Update marker
                    world.add_component(pipe_id, symbol_short!("pipemark"), marker.serialize(env));
                }
            }
        }
    }

    score_increase
}
