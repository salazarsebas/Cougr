//! Tower Defense game systems
//!
//! This module implements the core game logic systems for:
//! - Wave spawning
//! - Path progression
//! - Tower targeting and attacks
//! - Base damage
//! - End condition checking

use crate::components::{
    BaseComponent, ComponentTrait, EnemyComponent, GameStatusComponent, Position, TowerComponent,
    TowerKind, WaveComponent, PATH, PATH_LENGTH,
};
use cougr_core::simple_world::{EntityId, SimpleWorld};
use soroban_sdk::{symbol_short, Env, Vec};

// ============================================================================
// Configuration
// ============================================================================

/// Number of waves in the game
pub const TOTAL_WAVES: u32 = 5;

/// Enemies per wave
pub const ENEMIES_PER_WAVE: u32 = 5;

/// Base starting health
pub const BASE_HEALTH: u32 = 100;

/// Damage dealt when enemy reaches base
pub const ENEMY_BASE_DAMAGE: u32 = 10;

// ============================================================================
// Initialization
// ============================================================================

/// Initialize the game state
pub fn init_game(world: &mut SimpleWorld, env: &Env) -> EntityId {
    // Create game entity to hold global state
    let game_id = world.spawn_entity();

    // Initialize wave component
    let wave = WaveComponent::new(TOTAL_WAVES, ENEMIES_PER_WAVE);
    world.add_component(game_id, symbol_short!("wave"), wave.serialize(env));

    // Initialize base component
    let base = BaseComponent::new(BASE_HEALTH);
    world.add_component(game_id, symbol_short!("base"), base.serialize(env));

    // Initialize game status
    let status = GameStatusComponent::new();
    world.add_component(game_id, symbol_short!("status"), status.serialize(env));

    game_id
}

// ============================================================================
// Tower Placement
// ============================================================================

/// Place a tower at the specified position
pub fn place_tower(
    world: &mut SimpleWorld,
    env: &Env,
    x: u32,
    y: u32,
    tower_kind: u32,
) -> Option<EntityId> {
    // Validate tower kind
    let kind = TowerKind::from_u8(tower_kind as u8)?;

    // Check position is valid and not on path
    let pos = Position::new(x, y);
    if !pos.is_valid() {
        return None;
    }

    // Don't allow placing on path
    for (px, py) in PATH.iter() {
        if x == *px && y == *py {
            return None;
        }
    }

    // Create tower entity
    let tower_id = world.spawn_entity();

    // Add position component
    world.add_component(tower_id, symbol_short!("position"), pos.serialize(env));

    // Add tower component
    let tower = TowerComponent::new(kind);
    world.add_component(tower_id, symbol_short!("tower"), tower.serialize(env));

    Some(tower_id)
}

// ============================================================================
// Wave Spawn System
// ============================================================================

/// Spawn enemies according to wave state
pub fn wave_spawn_system(world: &mut SimpleWorld, game_id: EntityId, env: &Env) {
    // Get wave component
    let wave_data = match world.get_component(game_id, &symbol_short!("wave")) {
        Some(data) => data,
        None => return,
    };
    let mut wave = match WaveComponent::deserialize(env, &wave_data) {
        Some(w) => w,
        None => return,
    };

    // Check if we should spawn
    if wave.should_spawn() {
        // Spawn enemy at path start
        let enemy_id = world.spawn_entity();

        // Position at path start
        let (start_x, start_y) = PATH[0];
        let pos = Position::new(start_x, start_y);
        world.add_component(enemy_id, symbol_short!("position"), pos.serialize(env));

        // Enemy component
        let enemy = EnemyComponent::for_wave(wave.current_wave);
        world.add_component(enemy_id, symbol_short!("enemy"), enemy.serialize(env));

        // Record spawn
        wave.record_spawn();
    } else {
        // Tick spawn timer
        wave.tick();
    }

    // Save wave state
    world.add_component(game_id, symbol_short!("wave"), wave.serialize(env));
}

// ============================================================================
// Path Progression System
// ============================================================================

/// Move enemies along the path
pub fn path_progression_system(world: &mut SimpleWorld, env: &Env) {
    // Get all entities with enemy component
    let entities = world.get_entities_with_component(&symbol_short!("enemy"), env);

    for entity_id in entities.iter() {
        // Get enemy component
        let enemy_data = match world.get_component(entity_id, &symbol_short!("enemy")) {
            Some(data) => data,
            None => continue,
        };
        let mut enemy = match EnemyComponent::deserialize(env, &enemy_data) {
            Some(e) => e,
            None => continue,
        };

        // Skip dead enemies
        if !enemy.is_alive() {
            continue;
        }

        // Move enemy along path
        for _ in 0..enemy.speed {
            if enemy.path_index < (PATH_LENGTH as u32 - 1) {
                enemy.path_index += 1;
            }
        }

        // Update position based on path index
        let path_idx = enemy.path_index as usize;
        if path_idx < PATH_LENGTH {
            let (new_x, new_y) = PATH[path_idx];
            let new_pos = Position::new(new_x, new_y);
            world.add_component(entity_id, symbol_short!("position"), new_pos.serialize(env));
        }

        // Save enemy state
        world.add_component(entity_id, symbol_short!("enemy"), enemy.serialize(env));
    }
}

// ============================================================================
// Targeting System
// ============================================================================

/// Find enemies in range for each tower
pub fn get_targets_in_range(world: &SimpleWorld, tower_id: EntityId, env: &Env) -> Vec<EntityId> {
    let mut targets = Vec::new(env);

    // Get tower position and component
    let tower_pos_data = match world.get_component(tower_id, &symbol_short!("position")) {
        Some(data) => data,
        None => return targets,
    };
    let tower_pos = match Position::deserialize(env, &tower_pos_data) {
        Some(p) => p,
        None => return targets,
    };

    let tower_data = match world.get_component(tower_id, &symbol_short!("tower")) {
        Some(data) => data,
        None => return targets,
    };
    let tower = match TowerComponent::deserialize(env, &tower_data) {
        Some(t) => t,
        None => return targets,
    };

    // Find all enemies in range
    let enemies = world.get_entities_with_component(&symbol_short!("enemy"), env);
    for enemy_id in enemies.iter() {
        // Get enemy position
        let enemy_pos_data = match world.get_component(enemy_id, &symbol_short!("position")) {
            Some(data) => data,
            None => continue,
        };
        let enemy_pos = match Position::deserialize(env, &enemy_pos_data) {
            Some(p) => p,
            None => continue,
        };

        // Check enemy is alive
        let enemy_data = match world.get_component(enemy_id, &symbol_short!("enemy")) {
            Some(data) => data,
            None => continue,
        };
        let enemy = match EnemyComponent::deserialize(env, &enemy_data) {
            Some(e) => e,
            None => continue,
        };
        if !enemy.is_alive() {
            continue;
        }

        // Check range
        let distance = tower_pos.distance_to(&enemy_pos);
        if distance <= tower.range {
            targets.push_back(enemy_id);
        }
    }

    targets
}

// ============================================================================
// Attack Resolution System
// ============================================================================

/// Process tower attacks
pub fn attack_resolution_system(world: &mut SimpleWorld, game_id: EntityId, env: &Env) {
    // Get game status for tracking kills
    let status_data = match world.get_component(game_id, &symbol_short!("status")) {
        Some(data) => data,
        None => return,
    };
    let mut status = match GameStatusComponent::deserialize(env, &status_data) {
        Some(s) => s,
        None => return,
    };

    // Get all towers
    let towers = world.get_entities_with_component(&symbol_short!("tower"), env);

    for tower_id in towers.iter() {
        // Get tower component
        let tower_data = match world.get_component(tower_id, &symbol_short!("tower")) {
            Some(data) => data,
            None => continue,
        };
        let mut tower = match TowerComponent::deserialize(env, &tower_data) {
            Some(t) => t,
            None => continue,
        };

        // Tick cooldown
        tower.tick_cooldown();

        // Check if can attack
        if !tower.can_attack() {
            world.add_component(tower_id, symbol_short!("tower"), tower.serialize(env));
            continue;
        }

        // Find targets
        let targets = get_targets_in_range(world, tower_id, env);
        if targets.is_empty() {
            world.add_component(tower_id, symbol_short!("tower"), tower.serialize(env));
            continue;
        }

        // Attack first target (closest to base - highest path_index)
        let mut best_target: Option<EntityId> = None;
        let mut best_path_index: u32 = 0;

        for target_id in targets.iter() {
            let enemy_data = match world.get_component(target_id, &symbol_short!("enemy")) {
                Some(data) => data,
                None => continue,
            };
            let enemy = match EnemyComponent::deserialize(env, &enemy_data) {
                Some(e) => e,
                None => continue,
            };
            if enemy.path_index >= best_path_index {
                best_path_index = enemy.path_index;
                best_target = Some(target_id);
            }
        }

        // Deal damage to target
        if let Some(target_id) = best_target {
            let enemy_data = match world.get_component(target_id, &symbol_short!("enemy")) {
                Some(data) => data,
                None => continue,
            };
            let mut enemy = match EnemyComponent::deserialize(env, &enemy_data) {
                Some(e) => e,
                None => continue,
            };

            enemy.take_damage(tower.damage);

            // Record kill if enemy died
            if !enemy.is_alive() {
                status.record_kill();
            }

            world.add_component(target_id, symbol_short!("enemy"), enemy.serialize(env));
            tower.reset_cooldown();
        }

        world.add_component(tower_id, symbol_short!("tower"), tower.serialize(env));
    }

    // Save status
    world.add_component(game_id, symbol_short!("status"), status.serialize(env));
}

// ============================================================================
// Base Damage System
// ============================================================================

/// Check for enemies that reached the base and deal damage
pub fn base_damage_system(world: &mut SimpleWorld, game_id: EntityId, env: &Env) {
    // Get base component
    let base_data = match world.get_component(game_id, &symbol_short!("base")) {
        Some(data) => data,
        None => return,
    };
    let mut base = match BaseComponent::deserialize(env, &base_data) {
        Some(b) => b,
        None => return,
    };

    // Check all enemies
    let enemies = world.get_entities_with_component(&symbol_short!("enemy"), env);
    let mut enemies_to_remove: Vec<EntityId> = Vec::new(env);

    for enemy_id in enemies.iter() {
        let enemy_data = match world.get_component(enemy_id, &symbol_short!("enemy")) {
            Some(data) => data,
            None => continue,
        };
        let enemy = match EnemyComponent::deserialize(env, &enemy_data) {
            Some(e) => e,
            None => continue,
        };

        // Check if enemy reached base
        if enemy.is_alive() && enemy.reached_base() {
            base.take_damage(ENEMY_BASE_DAMAGE);
            enemies_to_remove.push_back(enemy_id);
        }
    }

    // Remove enemies that reached base (mark as dead)
    for enemy_id in enemies_to_remove.iter() {
        let enemy_data = match world.get_component(enemy_id, &symbol_short!("enemy")) {
            Some(data) => data,
            None => continue,
        };
        let mut enemy = match EnemyComponent::deserialize(env, &enemy_data) {
            Some(e) => e,
            None => continue,
        };
        enemy.hp = 0; // Mark as dead
        world.add_component(enemy_id, symbol_short!("enemy"), enemy.serialize(env));
    }

    // Save base state
    world.add_component(game_id, symbol_short!("base"), base.serialize(env));
}

// ============================================================================
// End Condition System
// ============================================================================

/// Check for win/loss conditions
pub fn end_condition_system(world: &mut SimpleWorld, game_id: EntityId, env: &Env) {
    // Get components
    let base_data = match world.get_component(game_id, &symbol_short!("base")) {
        Some(data) => data,
        None => return,
    };
    let base = match BaseComponent::deserialize(env, &base_data) {
        Some(b) => b,
        None => return,
    };

    let wave_data = match world.get_component(game_id, &symbol_short!("wave")) {
        Some(data) => data,
        None => return,
    };
    let wave = match WaveComponent::deserialize(env, &wave_data) {
        Some(w) => w,
        None => return,
    };

    let status_data = match world.get_component(game_id, &symbol_short!("status")) {
        Some(data) => data,
        None => return,
    };
    let mut status = match GameStatusComponent::deserialize(env, &status_data) {
        Some(s) => s,
        None => return,
    };

    // Already ended?
    if !status.is_active() {
        return;
    }

    // Check loss condition: base destroyed
    if base.is_destroyed() {
        status.set_lost();
        world.add_component(game_id, symbol_short!("status"), status.serialize(env));
        return;
    }

    // Check win condition: all waves complete and no enemies alive
    if wave.all_waves_complete() {
        let enemies = world.get_entities_with_component(&symbol_short!("enemy"), env);
        let mut any_alive = false;
        for enemy_id in enemies.iter() {
            let enemy_data = match world.get_component(enemy_id, &symbol_short!("enemy")) {
                Some(data) => data,
                None => continue,
            };
            let enemy = match EnemyComponent::deserialize(env, &enemy_data) {
                Some(e) => e,
                None => continue,
            };
            if enemy.is_alive() {
                any_alive = true;
                break;
            }
        }

        if !any_alive {
            status.set_won();
            world.add_component(game_id, symbol_short!("status"), status.serialize(env));
        }
    }
}

// ============================================================================
// Wave Advancement
// ============================================================================

/// Check if current wave is complete and advance to next
pub fn check_wave_advancement(world: &mut SimpleWorld, game_id: EntityId, env: &Env) {
    let wave_data = match world.get_component(game_id, &symbol_short!("wave")) {
        Some(data) => data,
        None => return,
    };
    let mut wave = match WaveComponent::deserialize(env, &wave_data) {
        Some(w) => w,
        None => return,
    };

    // Skip if waves are done or spawns remaining
    if wave.all_waves_complete() || wave.has_spawns_remaining() {
        return;
    }

    // Check if all enemies from this wave are dead
    let enemies = world.get_entities_with_component(&symbol_short!("enemy"), env);
    for enemy_id in enemies.iter() {
        let enemy_data = match world.get_component(enemy_id, &symbol_short!("enemy")) {
            Some(data) => data,
            None => continue,
        };
        let enemy = match EnemyComponent::deserialize(env, &enemy_data) {
            Some(e) => e,
            None => continue,
        };
        if enemy.is_alive() {
            return; // Still enemies alive
        }
    }

    // Advance to next wave
    wave.next_wave(ENEMIES_PER_WAVE);
    world.add_component(game_id, symbol_short!("wave"), wave.serialize(env));
}

// ============================================================================
// Tick System
// ============================================================================

/// Increment game tick counter
pub fn tick_system(world: &mut SimpleWorld, game_id: EntityId, env: &Env) {
    let status_data = match world.get_component(game_id, &symbol_short!("status")) {
        Some(data) => data,
        None => return,
    };
    let mut status = match GameStatusComponent::deserialize(env, &status_data) {
        Some(s) => s,
        None => return,
    };

    status.increment_tick();
    world.add_component(game_id, symbol_short!("status"), status.serialize(env));
}

// ============================================================================
// Query Helpers
// ============================================================================

/// Get wave component
pub fn get_wave(world: &SimpleWorld, game_id: EntityId, env: &Env) -> Option<WaveComponent> {
    let data = world.get_component(game_id, &symbol_short!("wave"))?;
    WaveComponent::deserialize(env, &data)
}

/// Get base component
pub fn get_base(world: &SimpleWorld, game_id: EntityId, env: &Env) -> Option<BaseComponent> {
    let data = world.get_component(game_id, &symbol_short!("base"))?;
    BaseComponent::deserialize(env, &data)
}

/// Get game status component
pub fn get_status(
    world: &SimpleWorld,
    game_id: EntityId,
    env: &Env,
) -> Option<GameStatusComponent> {
    let data = world.get_component(game_id, &symbol_short!("status"))?;
    GameStatusComponent::deserialize(env, &data)
}

/// Count alive enemies
pub fn count_alive_enemies(world: &SimpleWorld, env: &Env) -> u32 {
    let enemies = world.get_entities_with_component(&symbol_short!("enemy"), env);
    let mut count = 0u32;
    for enemy_id in enemies.iter() {
        let enemy_data = match world.get_component(enemy_id, &symbol_short!("enemy")) {
            Some(data) => data,
            None => continue,
        };
        let enemy = match EnemyComponent::deserialize(env, &enemy_data) {
            Some(e) => e,
            None => continue,
        };
        if enemy.is_alive() {
            count += 1;
        }
    }
    count
}
