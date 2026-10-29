//! Pokémon Mini game systems
//!
//! This module implements the core game logic systems for:
//! - Map generation (deterministic)
//! - Movement with collision detection
//! - Encounter triggering (deterministic)
//! - Turn-based battle resolution

use crate::components::{
    BattleAction, BattlePhase, BattleResult, BattleState, ComponentTrait, Creature, Direction,
    DirectionComponent, Position, TileType, ENCOUNTER_MODULO, MAP_HEIGHT, MAP_WIDTH,
};
use cougr_core::simple_world::{EntityId, SimpleWorld};
use soroban_sdk::{symbol_short, Env};

// ============================================================================
// Map System
// ============================================================================

/// Get the tile type at a given position using deterministic generation
///
/// Map layout (8x8):
/// - Spawn point at (1, 1)
/// - Walls around the edges and some interior
/// - TallGrass zones for encounters
/// - Water obstacles
pub fn get_tile_at(x: i32, y: i32) -> TileType {
    // Out of bounds is Wall
    if !(0..MAP_WIDTH).contains(&x) || !(0..MAP_HEIGHT).contains(&y) {
        return TileType::Wall;
    }

    // Border walls
    if x == 0 || x == MAP_WIDTH - 1 || y == 0 || y == MAP_HEIGHT - 1 {
        return TileType::Wall;
    }

    // Spawn point
    if x == 1 && y == 1 {
        return TileType::Spawn;
    }

    // Water obstacle (small pond in corner)
    if (x == 5 || x == 6) && (y == 5 || y == 6) {
        return TileType::Water;
    }

    // Interior wall obstacles
    if x == 4 && (y == 2 || y == 3) {
        return TileType::Wall;
    }

    // TallGrass zones (where encounters happen)
    // Zone 1: Top right area
    if (5..=6).contains(&x) && (1..=3).contains(&y) {
        return TileType::TallGrass;
    }
    // Zone 2: Bottom left area
    if (1..=3).contains(&x) && (5..=6).contains(&y) {
        return TileType::TallGrass;
    }
    // Zone 3: Center grass
    if x == 3 && y == 3 {
        return TileType::TallGrass;
    }

    // Default is regular grass
    TileType::Grass
}

/// Check if a position is valid for movement
pub fn can_move_to(x: i32, y: i32) -> bool {
    let tile = get_tile_at(x, y);
    !tile.is_blocked()
}

// ============================================================================
// Player System
// ============================================================================

/// Initialize a new player at spawn point
pub fn init_player(world: &mut SimpleWorld, env: &Env) -> EntityId {
    let player_id = world.spawn_entity();

    // Set position at spawn (1, 1)
    let position = Position::new(1, 1);
    world.add_component(
        player_id,
        symbol_short!("position"),
        position.serialize(env),
    );

    // Set initial facing direction
    let direction = DirectionComponent::new(Direction::Right);
    world.add_component(player_id, symbol_short!("facing"), direction.serialize(env));

    // Set player marker
    world.add_component(
        player_id,
        symbol_short!("player"),
        soroban_sdk::Bytes::from_array(env, &[1]),
    );

    // Set starter creature
    let creature = Creature::starter();
    world.add_component(
        player_id,
        symbol_short!("creature"),
        creature.serialize(env),
    );

    player_id
}

/// Get player position
pub fn get_player_position(
    world: &SimpleWorld,
    player_id: EntityId,
    env: &Env,
) -> Option<Position> {
    let pos_data = world.get_component(player_id, &symbol_short!("position"))?;
    Position::deserialize(env, &pos_data)
}

/// Get player creature
pub fn get_player_creature(
    world: &SimpleWorld,
    player_id: EntityId,
    env: &Env,
) -> Option<Creature> {
    let creature_data = world.get_component(player_id, &symbol_short!("creature"))?;
    Creature::deserialize(env, &creature_data)
}

/// Update player creature
pub fn update_player_creature(
    world: &mut SimpleWorld,
    player_id: EntityId,
    creature: &Creature,
    env: &Env,
) {
    world.add_component(
        player_id,
        symbol_short!("creature"),
        creature.serialize(env),
    );
}

/// Move the player in a direction
///
/// Returns:
/// - Ok(true) if movement successful and encounter triggered
/// - Ok(false) if movement successful, no encounter
/// - Err if movement blocked (wall/water/out of bounds)
pub fn move_player(
    world: &mut SimpleWorld,
    env: &Env,
    player_id: EntityId,
    direction: Direction,
    move_count: u32,
) -> Result<bool, ()> {
    // Get current position
    let current_pos = get_player_position(world, player_id, env).ok_or(())?;

    // Calculate new position
    let new_pos = current_pos.apply_direction(direction);

    // Check if valid
    if !new_pos.is_valid() || !can_move_to(new_pos.x, new_pos.y) {
        return Err(());
    }

    // Update position
    world.add_component(player_id, symbol_short!("position"), new_pos.serialize(env));

    // Update facing direction
    let dir_component = DirectionComponent::new(direction);
    world.add_component(
        player_id,
        symbol_short!("facing"),
        dir_component.serialize(env),
    );

    // Check for encounter
    let tile = get_tile_at(new_pos.x, new_pos.y);
    if tile.can_trigger_encounter() {
        let encounter = check_encounter_trigger(new_pos.x, new_pos.y, move_count);
        Ok(encounter)
    } else {
        Ok(false)
    }
}

/// Deterministic encounter check
///
/// Formula: (x + y + move_count) % ENCOUNTER_MODULO == 0
pub fn check_encounter_trigger(x: i32, y: i32, move_count: u32) -> bool {
    let sum = (x as u32) + (y as u32) + move_count;
    sum.is_multiple_of(ENCOUNTER_MODULO)
}

// ============================================================================
// Battle System
// ============================================================================

/// Start a new battle
pub fn start_battle(battle_id: u32, player_creature: Creature, move_count: u32) -> BattleState {
    let enemy = Creature::wild_from_seed(move_count);
    BattleState::new(battle_id, player_creature, enemy)
}

/// Process a battle action
///
/// Returns the updated battle state
pub fn process_battle_action(mut battle: BattleState, action: BattleAction) -> BattleState {
    // Check if battle is already finished
    if battle.is_finished() {
        return battle;
    }

    // Reset defending status
    battle.player_defending = false;

    match action {
        BattleAction::Attack => {
            // Player attacks enemy
            let damage = BattleState::calculate_damage(
                battle.player_creature.atk,
                battle.enemy_creature.def,
            );
            let enemy_alive = battle.enemy_creature.take_damage(damage);

            if !enemy_alive {
                // Player wins!
                battle.phase = BattlePhase::Finished;
                battle.result = BattleResult::Win;
                return battle;
            }
        }
        BattleAction::Defend => {
            // Player defends, reducing damage taken this turn
            battle.player_defending = true;
        }
        BattleAction::Run => {
            // Player escapes battle
            battle.phase = BattlePhase::Finished;
            battle.result = BattleResult::Escaped;
            return battle;
        }
    }

    // Enemy turn (always attacks)
    let player_def = if battle.player_defending {
        battle.player_creature.def + 3 // Defending bonus
    } else {
        battle.player_creature.def
    };

    let enemy_damage = BattleState::calculate_damage(battle.enemy_creature.atk, player_def);
    let player_alive = battle.player_creature.take_damage(enemy_damage);

    if !player_alive {
        // Player loses
        battle.phase = BattlePhase::Finished;
        battle.result = BattleResult::Lose;
        return battle;
    }

    // Continue to next turn
    battle.turn += 1;
    battle.phase = BattlePhase::WaitingPlayerAction;

    battle
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_spawn() {
        assert_eq!(get_tile_at(1, 1), TileType::Spawn);
    }

    #[test]
    fn test_map_walls() {
        // Border walls
        assert_eq!(get_tile_at(0, 0), TileType::Wall);
        assert_eq!(get_tile_at(7, 7), TileType::Wall);
        assert_eq!(get_tile_at(0, 4), TileType::Wall);
        assert_eq!(get_tile_at(4, 0), TileType::Wall);

        // Interior walls
        assert_eq!(get_tile_at(4, 2), TileType::Wall);
        assert_eq!(get_tile_at(4, 3), TileType::Wall);
    }

    #[test]
    fn test_map_water() {
        assert_eq!(get_tile_at(5, 5), TileType::Water);
        assert_eq!(get_tile_at(6, 6), TileType::Water);
    }

    #[test]
    fn test_map_tallgrass() {
        // Zone 1
        assert_eq!(get_tile_at(5, 1), TileType::TallGrass);
        assert_eq!(get_tile_at(6, 2), TileType::TallGrass);
        // Zone 2
        assert_eq!(get_tile_at(2, 5), TileType::TallGrass);
        assert_eq!(get_tile_at(3, 6), TileType::TallGrass);
    }

    #[test]
    fn test_map_grass() {
        assert_eq!(get_tile_at(2, 2), TileType::Grass);
        assert_eq!(get_tile_at(3, 4), TileType::Grass);
    }

    #[test]
    fn test_can_move_to() {
        assert!(can_move_to(2, 2)); // Grass
        assert!(can_move_to(1, 1)); // Spawn
        assert!(can_move_to(5, 1)); // TallGrass
        assert!(!can_move_to(0, 0)); // Wall
        assert!(!can_move_to(5, 5)); // Water
        assert!(!can_move_to(-1, 0)); // Out of bounds
    }

    #[test]
    fn test_encounter_trigger() {
        // (x + y + move_count) % 5 == 0
        assert!(check_encounter_trigger(2, 3, 0)); // 5 % 5 = 0
        assert!(check_encounter_trigger(1, 1, 3)); // 5 % 5 = 0
        assert!(!check_encounter_trigger(1, 1, 1)); // 3 % 5 != 0
        assert!(check_encounter_trigger(0, 0, 5)); // 5 % 5 = 0
    }

    #[test]
    fn test_init_player() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        let player_id = init_player(&mut world, &env);

        let pos = get_player_position(&world, player_id, &env).unwrap();
        assert_eq!(pos.x, 1);
        assert_eq!(pos.y, 1);

        let creature = get_player_creature(&world, player_id, &env).unwrap();
        assert_eq!(creature.species_id, 1);
        assert_eq!(creature.level, 5);
    }

    #[test]
    fn test_move_player() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        let player_id = init_player(&mut world, &env);

        // Move right (should succeed)
        let result = move_player(&mut world, &env, player_id, Direction::Right, 1);
        assert!(result.is_ok());

        let pos = get_player_position(&world, player_id, &env).unwrap();
        assert_eq!(pos.x, 2);
        assert_eq!(pos.y, 1);

        // Move down
        let result = move_player(&mut world, &env, player_id, Direction::Down, 2);
        assert!(result.is_ok());

        let pos = get_player_position(&world, player_id, &env).unwrap();
        assert_eq!(pos.x, 2);
        assert_eq!(pos.y, 2);
    }

    #[test]
    fn test_move_blocked() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        let player_id = init_player(&mut world, &env);

        // Try to move up into wall (should fail)
        let result = move_player(&mut world, &env, player_id, Direction::Up, 1);
        assert!(result.is_err());

        // Position should be unchanged
        let pos = get_player_position(&world, player_id, &env).unwrap();
        assert_eq!(pos.x, 1);
        assert_eq!(pos.y, 1);
    }

    #[test]
    fn test_battle_attack_win() {
        let player = Creature::new(1, 10, 30, 15, 8);
        let enemy = Creature::new(2, 5, 10, 5, 3);

        let mut battle = BattleState::new(1, player, enemy);

        // Attack until enemy is defeated
        for _ in 0..5 {
            if battle.is_finished() {
                break;
            }
            battle = process_battle_action(battle, BattleAction::Attack);
        }

        assert!(battle.is_finished());
        assert_eq!(battle.result, BattleResult::Win);
    }

    #[test]
    fn test_battle_run() {
        let player = Creature::starter();
        let enemy = Creature::wild_from_seed(10);

        let battle = BattleState::new(1, player, enemy);
        let battle = process_battle_action(battle, BattleAction::Run);

        assert!(battle.is_finished());
        assert_eq!(battle.result, BattleResult::Escaped);
    }

    #[test]
    fn test_battle_defend() {
        let player = Creature::new(1, 5, 50, 8, 5);
        let enemy = Creature::new(2, 5, 20, 10, 3);

        let battle = BattleState::new(1, player.clone(), enemy);

        // First attack without defending
        let battle1 = process_battle_action(battle, BattleAction::Attack);
        let hp_after_no_defend = battle1.player_creature.hp;

        // Reset and defend
        let player2 = player.clone();
        let enemy2 = Creature::new(2, 5, 20, 10, 3);
        let battle2 = BattleState::new(2, player2, enemy2);
        let battle2 = process_battle_action(battle2, BattleAction::Defend);
        let hp_after_defend = battle2.player_creature.hp;

        // Defending should result in less damage taken
        assert!(hp_after_defend >= hp_after_no_defend);
    }
}
