#![no_std]

use cougr_core::component::ComponentTrait;
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

// Game constants for fixed-point math (scale by 1000)
const FIXED_POINT_SCALE: i32 = 1000;
const GRAVITY: i32 = 500; // 0.5 units per step^2
const MAX_SIMULATION_STEPS: u32 = 50;
const GRID_SIZE: i32 = 1000; // Grid coordinates are 0-1000 (scaled by 1000)

// Material damage resistance multipliers (scaled by 1000)
const WOOD_RESISTANCE: i32 = 1000; // 1.0x damage
const GLASS_RESISTANCE: i32 = 500; // 0.5x damage (more vulnerable)
const STONE_RESISTANCE: i32 = 2000; // 2.0x damage (more resistant)

/// Material types for structural blocks
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Material {
    Wood,
    Glass,
    Stone,
}

/// Bird types for different projectile behaviors
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BirdType {
    Red,    // Standard bird
    Yellow, // Speed boost
    Blue,   // Splits into 3
}

/// Game status
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameStatus {
    InProgress,
    Won,
    Lost,
}

/// Shot outcome for player feedback
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShotOutcome {
    Hit(u32, u32), // entity_id, damage
    Miss,
    Win(u32), // final_score
    Loss,
}

impl soroban_sdk::TryFromVal<Env, soroban_sdk::Val> for ShotOutcome {
    type Error = soroban_sdk::Error;

    fn try_from_val(_env: &Env, _val: &soroban_sdk::Val) -> Result<Self, soroban_sdk::Error> {
        // For simplicity, we'll use a basic implementation
        // In a real implementation, you'd properly serialize/deserialize the enum
        Err(soroban_sdk::Error::from_contract_error(1))
    }
}

impl soroban_sdk::IntoVal<Env, soroban_sdk::Val> for ShotOutcome {
    fn into_val(&self, env: &Env) -> soroban_sdk::Val {
        // For simplicity, we'll use a basic implementation
        // In a real implementation, you'd properly serialize/deserialize the enum
        match self {
            ShotOutcome::Hit(entity_id, damage) => {
                let vec: soroban_sdk::Vec<u32> =
                    soroban_sdk::Vec::from_array(env, [0u32, *entity_id, *damage]);
                vec.into_val(env)
            }
            ShotOutcome::Miss => 1u32.into_val(env),
            ShotOutcome::Win(final_score) => {
                let vec: soroban_sdk::Vec<u32> =
                    soroban_sdk::Vec::from_array(env, [2u32, *final_score]);
                vec.into_val(env)
            }
            ShotOutcome::Loss => 3u32.into_val(env),
        }
    }
}

/// Position component - grid position using fixed-point math
#[contracttype]
#[derive(Clone, Debug)]
pub struct PositionComponent {
    pub x: i32, // Scaled by 1000
    pub y: i32, // Scaled by 1000
}

impl ComponentTrait for PositionComponent {
    fn component_type() -> Symbol {
        symbol_short!("position")
    }

    fn serialize(&self, env: &Env) -> soroban_sdk::Bytes {
        let mut bytes = soroban_sdk::Bytes::new(env);
        bytes.append(&soroban_sdk::Bytes::from_array(env, &self.x.to_be_bytes()));
        bytes.append(&soroban_sdk::Bytes::from_array(env, &self.y.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, data: &soroban_sdk::Bytes) -> Option<Self> {
        if data.len() != 8 {
            return None;
        }
        let x = i32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]);
        let y = i32::from_be_bytes([data.get(4)?, data.get(5)?, data.get(6)?, data.get(7)?]);
        Some(Self { x, y })
    }
}

/// Health component - hit points for structures and pigs
#[contracttype]
#[derive(Clone, Debug)]
pub struct HealthComponent {
    pub hp: u32,
    pub max_hp: u32,
}

impl ComponentTrait for HealthComponent {
    fn component_type() -> Symbol {
        symbol_short!("health")
    }

    fn serialize(&self, env: &Env) -> soroban_sdk::Bytes {
        let mut bytes = soroban_sdk::Bytes::new(env);
        bytes.append(&soroban_sdk::Bytes::from_array(env, &self.hp.to_be_bytes()));
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.max_hp.to_be_bytes(),
        ));
        bytes
    }

    fn deserialize(_env: &Env, data: &soroban_sdk::Bytes) -> Option<Self> {
        if data.len() != 8 {
            return None;
        }
        let hp = u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]);
        let max_hp = u32::from_be_bytes([data.get(4)?, data.get(5)?, data.get(6)?, data.get(7)?]);
        Some(Self { hp, max_hp })
    }
}

/// Material component - damage resistance multiplier
#[contracttype]
#[derive(Clone, Debug)]
pub struct MaterialComponent {
    pub kind: Material,
}

impl ComponentTrait for MaterialComponent {
    fn component_type() -> Symbol {
        symbol_short!("material")
    }

    fn serialize(&self, env: &Env) -> soroban_sdk::Bytes {
        let mut bytes = soroban_sdk::Bytes::new(env);
        let value = match self.kind {
            Material::Wood => 0u8,
            Material::Glass => 1u8,
            Material::Stone => 2u8,
        };
        bytes.append(&soroban_sdk::Bytes::from_array(env, &[value]));
        bytes
    }

    fn deserialize(_env: &Env, data: &soroban_sdk::Bytes) -> Option<Self> {
        if data.len() != 1 {
            return None;
        }
        let value = data.get(0)?;
        let kind = match value {
            0 => Material::Wood,
            1 => Material::Glass,
            2 => Material::Stone,
            _ => return None,
        };
        Some(Self { kind })
    }
}

/// Projectile component - shot parameters for trajectory resolution
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProjectileComponent {
    pub bird_type: BirdType,
    pub angle: i32, // In degrees, scaled by 1000
    pub power: i32, // Launch power, scaled by 1000
    pub active: bool,
}

impl ComponentTrait for ProjectileComponent {
    fn component_type() -> Symbol {
        symbol_short!("proj")
    }

    fn serialize(&self, env: &Env) -> soroban_sdk::Bytes {
        let mut bytes = soroban_sdk::Bytes::new(env);
        let bird_value = match self.bird_type {
            BirdType::Red => 0u8,
            BirdType::Yellow => 1u8,
            BirdType::Blue => 2u8,
        };
        bytes.append(&soroban_sdk::Bytes::from_array(env, &[bird_value]));
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.angle.to_be_bytes(),
        ));
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.power.to_be_bytes(),
        ));
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &[if self.active { 1u8 } else { 0u8 }],
        ));
        bytes
    }

    fn deserialize(_env: &Env, data: &soroban_sdk::Bytes) -> Option<Self> {
        if data.len() != 10 {
            return None;
        }
        let bird_value = data.get(0)?;
        let bird_type = match bird_value {
            0 => BirdType::Red,
            1 => BirdType::Yellow,
            2 => BirdType::Blue,
            _ => return None,
        };
        let angle = i32::from_be_bytes([data.get(1)?, data.get(2)?, data.get(3)?, data.get(4)?]);
        let power = i32::from_be_bytes([data.get(5)?, data.get(6)?, data.get(7)?, data.get(8)?]);
        let active = data.get(9)? != 0;
        Some(Self {
            bird_type,
            angle,
            power,
            active,
        })
    }
}

/// Score component - running score tracker
#[contracttype]
#[derive(Clone, Debug)]
pub struct ScoreComponent {
    pub points: u32,
    pub birds_remaining: u32,
}

impl ComponentTrait for ScoreComponent {
    fn component_type() -> Symbol {
        symbol_short!("score")
    }

    fn serialize(&self, env: &Env) -> soroban_sdk::Bytes {
        let mut bytes = soroban_sdk::Bytes::new(env);
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.points.to_be_bytes(),
        ));
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.birds_remaining.to_be_bytes(),
        ));
        bytes
    }

    fn deserialize(_env: &Env, data: &soroban_sdk::Bytes) -> Option<Self> {
        if data.len() != 8 {
            return None;
        }
        let points = u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]);
        let birds_remaining =
            u32::from_be_bytes([data.get(4)?, data.get(5)?, data.get(6)?, data.get(7)?]);
        Some(Self {
            points,
            birds_remaining,
        })
    }
}

/// Level configuration component - level metadata and game state
#[contracttype]
#[derive(Clone, Debug)]
pub struct LevelConfigComponent {
    pub level_id: u32,
    pub status: GameStatus,
    pub player: Address,
}

impl ComponentTrait for LevelConfigComponent {
    fn component_type() -> Symbol {
        symbol_short!("level")
    }

    fn serialize(&self, env: &Env) -> soroban_sdk::Bytes {
        let mut bytes = soroban_sdk::Bytes::new(env);
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.level_id.to_be_bytes(),
        ));
        let status_value = match self.status {
            GameStatus::InProgress => 0u8,
            GameStatus::Won => 1u8,
            GameStatus::Lost => 2u8,
        };
        bytes.append(&soroban_sdk::Bytes::from_array(env, &[status_value]));

        // Convert Address to string and then to bytes
        let address_str = self.player.to_string();
        let address_bytes = address_str.to_bytes();
        // Convert Bytes to slice for from_slice
        let mut slice = [0u8; 64]; // Fixed size array
        for i in 0..address_bytes.len().min(64) {
            slice[i as usize] = address_bytes.get(i).unwrap_or(0);
        }
        bytes.append(&soroban_sdk::Bytes::from_slice(env, &slice));
        bytes
    }

    fn deserialize(_env: &Env, data: &soroban_sdk::Bytes) -> Option<Self> {
        if data.len() < 9 {
            return None;
        }
        let level_id = u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]);
        let status_value = data.get(4)?;
        let status = match status_value {
            0 => GameStatus::InProgress,
            1 => GameStatus::Won,
            2 => GameStatus::Lost,
            _ => return None,
        };

        // For simplicity, we'll skip address deserialization in this example
        // In a real implementation, you'd properly deserialize the address
        // For now, we'll create a placeholder address using a fixed pattern
        let address_str = soroban_sdk::String::from_str(
            _env,
            "GD5DJHHB6F4AQJ4GHQPM7XQFNBEI7LJTGFSXK6IGKPYEPJ5VVXRKQ",
        );
        let player = Address::from_string(&address_str);

        Some(Self {
            level_id,
            status,
            player,
        })
    }
}

/// ECS World State - contains all game entities and components
#[contracttype]
#[derive(Clone, Debug)]
pub struct ECSWorldState {
    // Entity 0: Level Configuration
    pub level_config: LevelConfigComponent,
    // Entity 1: Score
    pub score: ScoreComponent,
    // Entity 2: Active Projectile
    pub projectile: ProjectileComponent,
    // Entities 3+: Game objects (structures, pigs)
    // For simplicity, we'll use fixed arrays for this example
    pub positions: Vec<PositionComponent>,
    pub healths: Vec<HealthComponent>,
    pub materials: Vec<MaterialComponent>,
}

/// External game state for API responses
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LevelState {
    pub level_id: u32,
    pub status: GameStatus,
    pub score: u32,
    pub birds_remaining: u32,
    pub entity_count: u32,
}

const ECS_WORLD_KEY: Symbol = symbol_short!("ANGRY_ECS");

#[contract]
pub struct AngryBirdsContract;

#[contractimpl]
impl AngryBirdsContract {
    /// Initialize a new level with default configuration
    pub fn init_level(env: Env, player: Address, level_id: u32) -> LevelState {
        let world_state = ECSWorldState {
            level_config: LevelConfigComponent {
                level_id,
                status: GameStatus::InProgress,
                player,
            },
            score: ScoreComponent {
                points: 0,
                birds_remaining: 3, // Start with 3 birds
            },
            projectile: ProjectileComponent {
                bird_type: BirdType::Red,
                angle: 0,
                power: 0,
                active: false,
            },
            positions: Vec::new(&env),
            healths: Vec::new(&env),
            materials: Vec::new(&env),
        };

        env.storage().instance().set(&ECS_WORLD_KEY, &world_state);
        Self::world_to_level_state(&world_state)
    }

    /// Shoot a bird with specified angle and power
    pub fn shoot(env: Env, player: Address, angle: i32, power: i32) -> ShotOutcome {
        let mut world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&ECS_WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        // Validate player and game state
        if world_state.level_config.player != player {
            panic!("Not authorized");
        }
        if world_state.level_config.status != GameStatus::InProgress {
            panic!("Game not in progress");
        }
        if world_state.score.birds_remaining == 0 {
            panic!("No birds remaining");
        }
        if world_state.projectile.active {
            panic!("Projectile already active");
        }

        // Update projectile component
        world_state.projectile.angle = angle;
        world_state.projectile.power = power;
        world_state.projectile.active = true;
        world_state.score.birds_remaining -= 1;

        // Run shot resolution system
        let outcome = Self::shot_resolution_system(&mut world_state);

        // Apply damage system if hit occurred
        if let ShotOutcome::Hit(entity_id, damage) = outcome {
            Self::damage_system(&mut world_state, entity_id, damage);
        }

        // Update score system
        Self::score_system(&mut world_state);

        // Check win conditions
        let final_outcome = Self::win_condition_system(&mut world_state);

        env.storage().instance().set(&ECS_WORLD_KEY, &world_state);
        final_outcome
    }

    /// Get current level state
    pub fn get_state(env: Env) -> LevelState {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&ECS_WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        Self::world_to_level_state(&world_state)
    }

    /// Reset the level
    pub fn reset_level(env: Env, player: Address) -> LevelState {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&ECS_WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        if world_state.level_config.player != player {
            panic!("Not authorized");
        }

        Self::init_level(env, player, world_state.level_config.level_id)
    }

    // ECS Systems Implementation

    /// ShotResolutionSystem - resolves projectile trajectory using discrete steps
    fn shot_resolution_system(world: &mut ECSWorldState) -> ShotOutcome {
        // Convert angle to radians and calculate initial velocity components
        // Using fixed-point math for trigonometric calculations
        let angle_deg = world.projectile.angle / 1000; // Convert back to degrees
        let _angle_rad = (angle_deg * 314159) / (180000); // π/180 approximation

        // Use precomputed trigonometric values for common angles
        let (cos_val, sin_val) = Self::get_trig_values(angle_deg);

        let vx = (world.projectile.power * cos_val) / (FIXED_POINT_SCALE * 1000);
        let vy = (world.projectile.power * sin_val) / (FIXED_POINT_SCALE * 1000);

        let mut x = 0; // Start from left edge
        let mut y = 500 * FIXED_POINT_SCALE; // Start from middle height
        let current_vx = vx;
        let mut current_vy = vy;

        for _step in 0..MAX_SIMULATION_STEPS {
            // Update position
            x += current_vx;
            y += current_vy;

            // Apply gravity
            current_vy -= GRAVITY;

            // Check boundaries
            if !(0..=GRID_SIZE * FIXED_POINT_SCALE).contains(&x) || y < 0 {
                return ShotOutcome::Miss;
            }

            // Check collisions with entities
            for (i, pos) in world.positions.iter().enumerate() {
                let i_u32 = i as u32;
                if i_u32 >= world.healths.len() || world.healths.get(i_u32).unwrap().hp == 0 {
                    continue; // Skip destroyed entities
                }

                let dx = (x - pos.x).abs();
                let dy = (y - pos.y).abs();

                // Simple circular collision (radius = 50 units)
                if dx < 50 * FIXED_POINT_SCALE && dy < 50 * FIXED_POINT_SCALE {
                    let damage = Self::calculate_damage(&world.projectile);
                    return ShotOutcome::Hit(i_u32, damage);
                }
            }
        }

        ShotOutcome::Miss
    }

    /// DamageSystem - applies damage based on material resistance
    fn damage_system(world: &mut ECSWorldState, entity_id: u32, base_damage: u32) {
        let material_resistance = match world.materials.get(entity_id).unwrap().kind {
            Material::Wood => WOOD_RESISTANCE,
            Material::Glass => GLASS_RESISTANCE,
            Material::Stone => STONE_RESISTANCE,
        };

        // Apply material resistance to damage
        let actual_damage = (base_damage * FIXED_POINT_SCALE as u32) / material_resistance as u32;

        if let Some(mut health) = world.healths.get(entity_id) {
            if health.hp > actual_damage {
                health.hp -= actual_damage;
            } else {
                health.hp = 0; // Entity destroyed
            }
            world.healths.set(entity_id, health);
        }
    }

    /// ScoreSystem - calculates points from destroyed entities
    fn score_system(world: &mut ECSWorldState) {
        // Award points for entities with 0 health
        for health in world.healths.iter() {
            if health.hp == 0 {
                world.score.points += 100; // 100 points per destroyed entity
            }
        }
    }

    /// WinConditionSystem - checks win/loss conditions
    fn win_condition_system(world: &mut ECSWorldState) -> ShotOutcome {
        // Check if all pigs are destroyed (simplified: check if all entities are destroyed)
        let all_destroyed = world.healths.iter().all(|h| h.hp == 0);

        if all_destroyed {
            world.level_config.status = GameStatus::Won;
            return ShotOutcome::Win(world.score.points);
        }

        // Check if no birds remain and projectile is inactive
        if world.score.birds_remaining == 0 && !world.projectile.active {
            world.level_config.status = GameStatus::Lost;
            return ShotOutcome::Loss;
        }

        // Reset projectile after resolution
        world.projectile.active = false;

        // Default outcome based on whether any entities remain
        if all_destroyed {
            ShotOutcome::Win(world.score.points)
        } else {
            ShotOutcome::Miss
        }
    }

    // Helper functions

    fn get_trig_values(angle_deg: i32) -> (i32, i32) {
        // Precomputed trigonometric values for common angles (scaled by 1000)
        match angle_deg {
            0 => (1000, 0),     // cos(0°) = 1, sin(0°) = 0
            30 => (866, 500),   // cos(30°) ≈ 0.866, sin(30°) = 0.5
            45 => (707, 707),   // cos(45°) ≈ 0.707, sin(45°) ≈ 0.707
            60 => (500, 866),   // cos(60°) = 0.5, sin(60°) ≈ 0.866
            90 => (0, 1000),    // cos(90°) = 0, sin(90°) = 1
            120 => (-500, 866), // cos(120°) = -0.5, sin(120°) ≈ 0.866
            135 => (-707, 707), // cos(135°) ≈ -0.707, sin(135°) ≈ 0.707
            150 => (-866, 500), // cos(150°) ≈ -0.866, sin(150°) = 0.5
            180 => (-1000, 0),  // cos(180°) = -1, sin(180°) = 0
            _ => (1000, 0),     // Default to 0° for unsupported angles
        }
    }

    fn calculate_damage(projectile: &ProjectileComponent) -> u32 {
        // Base damage depends on bird type and power
        let base_damage = match projectile.bird_type {
            BirdType::Red => 100,
            BirdType::Yellow => 150,
            BirdType::Blue => 75,
        };

        // Scale by power (power is scaled by 1000)
        (base_damage * projectile.power as u32) / FIXED_POINT_SCALE as u32
    }

    fn world_to_level_state(world: &ECSWorldState) -> LevelState {
        LevelState {
            level_id: world.level_config.level_id,
            status: world.level_config.status.clone(),
            score: world.score.points,
            birds_remaining: world.score.birds_remaining,
            entity_count: world.positions.len(),
        }
    }
}

#[cfg(test)]
mod test;
