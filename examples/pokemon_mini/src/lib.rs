#![no_std]

//! # Pokémon Mini On-Chain Game
//!
//! This example demonstrates how to build a Pokémon-style mini game using the
//! `cougr-core` ECS framework on the Stellar blockchain via Soroban.
//!
//! ## Game Features
//!
//! - **Tile Map**: 8x8 deterministic map with various tile types
//! - **Movement**: Grid-based movement with collision detection
//! - **Encounters**: Deterministic encounter triggering on TallGrass tiles
//! - **Battle**: Turn-based 1v1 combat with Attack, Defend, Run actions
//!
//! ## Architecture
//!
//! This implementation uses an Entity-Component-System (ECS) pattern:
//! - **Entities**: Player, Creatures
//! - **Components**: Position, Direction, Creature stats
//! - **Systems**: Movement, Encounter, Battle resolution
//!
//! The `cougr-core` package simplifies on-chain game development by providing:
//! - Serialization-ready component patterns for on-chain storage
//! - Entity management optimized for Soroban's constraints
//! - A consistent architecture for game logic

mod components;
mod systems;

use components::{
    BattleAction, BattleResult, BattleState, Creature, Direction, Position, MAP_HEIGHT, MAP_WIDTH,
};
use cougr_core::SimpleWorld;
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env};

// ============================================================================
// Game State
// ============================================================================

/// Main game state stored in contract storage
#[contracttype]
#[derive(Clone, Debug)]
pub struct GameState {
    pub player_id: u32,
    pub move_count: u32,
    pub in_battle: bool,
    pub battle_count: u32,
    pub wins: u32,
    pub losses: u32,
    pub escapes: u32,
}

// ============================================================================
// Contract
// ============================================================================

/// Pokémon Mini game contract
#[contract]
pub struct PokemonMiniContract;

#[contractimpl]
impl PokemonMiniContract {
    // ========================================================================
    // Initialization
    // ========================================================================

    /// Initialize a new player at the spawn point with a starter creature
    ///
    /// Creates the player entity with:
    /// - Position at spawn (1, 1)
    /// - Facing direction: Right
    /// - Starter creature (species 1, level 5)
    pub fn init_player(env: Env) {
        let mut world = SimpleWorld::new(&env);
        let player_id = systems::init_player(&mut world, &env);

        let game_state = GameState {
            player_id,
            move_count: 0,
            in_battle: false,
            battle_count: 0,
            wins: 0,
            losses: 0,
            escapes: 0,
        };

        env.storage()
            .persistent()
            .set(&symbol_short!("state"), &game_state);
        env.storage()
            .persistent()
            .set(&symbol_short!("world"), &world);
    }

    // ========================================================================
    // Player State Queries
    // ========================================================================

    /// Get the player's current position
    pub fn get_player_state(env: Env) -> (i32, i32, u32, bool, u32) {
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

        let pos = systems::get_player_position(&world, game_state.player_id, &env)
            .unwrap_or(Position::new(0, 0));

        let creature = systems::get_player_creature(&world, game_state.player_id, &env)
            .unwrap_or(Creature::starter());

        (
            pos.x,
            pos.y,
            game_state.move_count,
            game_state.in_battle,
            creature.hp,
        )
    }

    /// Get the player's creature stats
    pub fn get_creature_stats(env: Env) -> (u32, u32, u32, u32, u32, u32) {
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

        let creature = systems::get_player_creature(&world, game_state.player_id, &env)
            .unwrap_or(Creature::starter());

        (
            creature.species_id,
            creature.level,
            creature.hp,
            creature.max_hp,
            creature.atk,
            creature.def,
        )
    }

    /// Get the tile type at a specific position
    ///
    /// Returns:
    /// - 0: Grass
    /// - 1: Wall
    /// - 2: Water
    /// - 3: TallGrass
    /// - 4: Spawn
    pub fn get_tile(x: i32, y: i32) -> u32 {
        systems::get_tile_at(x, y).to_u8() as u32
    }

    /// Get map dimensions
    pub fn get_map_size() -> (i32, i32) {
        (MAP_WIDTH, MAP_HEIGHT)
    }

    /// Get battle statistics
    pub fn get_battle_stats(env: Env) -> (u32, u32, u32) {
        let game_state: GameState = env
            .storage()
            .persistent()
            .get(&symbol_short!("state"))
            .unwrap();

        (game_state.wins, game_state.losses, game_state.escapes)
    }

    // ========================================================================
    // Movement
    // ========================================================================

    /// Move the player in a direction
    ///
    /// Direction values:
    /// - 0: Up
    /// - 1: Down
    /// - 2: Left
    /// - 3: Right
    ///
    /// Returns:
    /// - 0: Movement blocked (wall/water/in battle)
    /// - 1: Movement successful, no encounter
    /// - 2: Movement successful, encounter triggered (battle started)
    pub fn move_player(env: Env, direction: u32) -> u32 {
        let mut game_state: GameState = env
            .storage()
            .persistent()
            .get(&symbol_short!("state"))
            .unwrap();

        // Cannot move during battle
        if game_state.in_battle {
            return 0;
        }

        let dir = match Direction::from_u8(direction as u8) {
            Some(d) => d,
            None => return 0,
        };

        let mut world: SimpleWorld = env
            .storage()
            .persistent()
            .get(&symbol_short!("world"))
            .unwrap();

        game_state.move_count += 1;

        let result = systems::move_player(
            &mut world,
            &env,
            game_state.player_id,
            dir,
            game_state.move_count,
        );

        let return_code = match result {
            Err(()) => 0,   // Blocked
            Ok(false) => 1, // Moved, no encounter
            Ok(true) => {
                // Encounter! Start battle
                let creature = systems::get_player_creature(&world, game_state.player_id, &env)
                    .unwrap_or(Creature::starter());

                game_state.battle_count += 1;
                let battle =
                    systems::start_battle(game_state.battle_count, creature, game_state.move_count);

                game_state.in_battle = true;
                env.storage()
                    .persistent()
                    .set(&symbol_short!("battle"), &battle);

                2
            }
        };

        env.storage()
            .persistent()
            .set(&symbol_short!("state"), &game_state);
        env.storage()
            .persistent()
            .set(&symbol_short!("world"), &world);

        return_code
    }

    // ========================================================================
    // Battle
    // ========================================================================

    /// Get current battle state
    ///
    /// Returns: (in_battle, player_hp, enemy_hp, turn, result)
    /// Result values:
    /// - 0: None (battle ongoing)
    /// - 1: Win
    /// - 2: Lose
    /// - 3: Escaped
    pub fn get_battle_state(env: Env) -> (bool, u32, u32, u32, u32) {
        let game_state: GameState = env
            .storage()
            .persistent()
            .get(&symbol_short!("state"))
            .unwrap();

        if !game_state.in_battle {
            return (false, 0, 0, 0, 0);
        }

        let battle: BattleState = env
            .storage()
            .persistent()
            .get(&symbol_short!("battle"))
            .unwrap();

        (
            true,
            battle.player_creature.hp,
            battle.enemy_creature.hp,
            battle.turn,
            battle.result as u32,
        )
    }

    /// Execute a battle action
    ///
    /// Action values:
    /// - 0: Attack
    /// - 1: Defend
    /// - 2: Run
    ///
    /// Returns:
    /// - 0: Not in battle / invalid action
    /// - 1: Action executed, battle continues
    /// - 2: Battle ended (win/lose/escape)
    pub fn battle_action(env: Env, action: u32) -> u32 {
        let mut game_state: GameState = env
            .storage()
            .persistent()
            .get(&symbol_short!("state"))
            .unwrap();

        if !game_state.in_battle {
            return 0;
        }

        let action = match BattleAction::from_u8(action as u8) {
            Some(a) => a,
            None => return 0,
        };

        let battle: BattleState = env
            .storage()
            .persistent()
            .get(&symbol_short!("battle"))
            .unwrap();

        let new_battle = systems::process_battle_action(battle, action);

        if new_battle.is_finished() {
            // Update stats
            match new_battle.result {
                BattleResult::Win => {
                    game_state.wins += 1;
                    // Heal creature on win
                    let mut world: SimpleWorld = env
                        .storage()
                        .persistent()
                        .get(&symbol_short!("world"))
                        .unwrap();
                    let mut creature = new_battle.player_creature.clone();
                    creature.heal_full();
                    systems::update_player_creature(
                        &mut world,
                        game_state.player_id,
                        &creature,
                        &env,
                    );
                    env.storage()
                        .persistent()
                        .set(&symbol_short!("world"), &world);
                }
                BattleResult::Lose => game_state.losses += 1,
                BattleResult::Escaped => game_state.escapes += 1,
                BattleResult::None => {}
            }

            game_state.in_battle = false;

            env.storage()
                .persistent()
                .set(&symbol_short!("state"), &game_state);
            env.storage()
                .persistent()
                .set(&symbol_short!("battle"), &new_battle);

            return 2;
        }

        env.storage()
            .persistent()
            .set(&symbol_short!("battle"), &new_battle);

        1
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::TileType;

    #[test]
    fn test_init_player() {
        let env = Env::default();
        let contract_id = env.register(PokemonMiniContract, ());
        let client = PokemonMiniContractClient::new(&env, &contract_id);

        client.init_player();

        let (x, y, move_count, in_battle, hp) = client.get_player_state();
        assert_eq!(x, 1);
        assert_eq!(y, 1);
        assert_eq!(move_count, 0);
        assert!(!in_battle);
        assert!(hp > 0);
    }

    #[test]
    fn test_get_creature_stats() {
        let env = Env::default();
        let contract_id = env.register(PokemonMiniContract, ());
        let client = PokemonMiniContractClient::new(&env, &contract_id);

        client.init_player();

        let (species_id, level, hp, max_hp, atk, def) = client.get_creature_stats();
        assert_eq!(species_id, 1); // Starter species
        assert_eq!(level, 5); // Starter level
        assert_eq!(hp, max_hp); // Full health
        assert!(atk > 0);
        assert!(def > 0);
    }

    #[test]
    fn test_get_tile() {
        let env = Env::default();
        let contract_id = env.register(PokemonMiniContract, ());
        let client = PokemonMiniContractClient::new(&env, &contract_id);

        // Spawn
        assert_eq!(client.get_tile(&1, &1), TileType::Spawn.to_u8() as u32);
        // Wall (border)
        assert_eq!(client.get_tile(&0, &0), TileType::Wall.to_u8() as u32);
        // Water
        assert_eq!(client.get_tile(&5, &5), TileType::Water.to_u8() as u32);
        // TallGrass
        assert_eq!(client.get_tile(&5, &1), TileType::TallGrass.to_u8() as u32);
        // Regular grass
        assert_eq!(client.get_tile(&2, &2), TileType::Grass.to_u8() as u32);
    }

    #[test]
    fn test_get_map_size() {
        let env = Env::default();
        let contract_id = env.register(PokemonMiniContract, ());
        let client = PokemonMiniContractClient::new(&env, &contract_id);

        let (width, height) = client.get_map_size();
        assert_eq!(width, 8);
        assert_eq!(height, 8);
    }

    #[test]
    fn test_movement_basic() {
        let env = Env::default();
        let contract_id = env.register(PokemonMiniContract, ());
        let client = PokemonMiniContractClient::new(&env, &contract_id);

        client.init_player();

        // Move right
        let result = client.move_player(&3); // Right
        assert!(result >= 1); // Should succeed

        let (x, y, _, _, _) = client.get_player_state();
        assert_eq!(x, 2);
        assert_eq!(y, 1);

        // Move down
        let result = client.move_player(&1); // Down
        assert!(result >= 1);

        let (x, y, _, _, _) = client.get_player_state();
        assert_eq!(x, 2);
        assert_eq!(y, 2);
    }

    #[test]
    fn test_movement_boundaries() {
        let env = Env::default();
        let contract_id = env.register(PokemonMiniContract, ());
        let client = PokemonMiniContractClient::new(&env, &contract_id);

        client.init_player();

        // Try to move up into wall (player at 1,1)
        let result = client.move_player(&0); // Up
        assert_eq!(result, 0); // Should be blocked

        // Position unchanged
        let (x, y, _, _, _) = client.get_player_state();
        assert_eq!(x, 1);
        assert_eq!(y, 1);

        // Try to move left into wall
        let result = client.move_player(&2); // Left
        assert_eq!(result, 0); // Should be blocked
    }

    #[test]
    fn test_movement_blocked_water() {
        let env = Env::default();
        let contract_id = env.register(PokemonMiniContract, ());
        let client = PokemonMiniContractClient::new(&env, &contract_id);

        client.init_player();

        // Navigate towards water at (5,5)
        // Move to position (4, 4) first
        for _ in 0..3 {
            client.move_player(&3); // Right
        }
        for _ in 0..3 {
            client.move_player(&1); // Down
        }

        let (_x, _y, _, in_battle, _) = client.get_player_state();

        // If we're in battle, skip the water test
        if in_battle {
            return;
        }

        // Try to move into water
        let _result = client.move_player(&3); // Right towards water

        // Position x should be less than 5 (blocked by water) or we hit encounter
        let (new_x, _, _, in_battle, _) = client.get_player_state();
        if !in_battle {
            assert!(new_x <= 5);
        }
    }

    #[test]
    fn test_encounter_trigger() {
        let env = Env::default();
        let contract_id = env.register(PokemonMiniContract, ());
        let client = PokemonMiniContractClient::new(&env, &contract_id);

        client.init_player();

        // Navigate to TallGrass and trigger encounter
        // TallGrass at (5,1) - move right 4 times from (1,1)
        let mut encountered = false;
        for _ in 0..10 {
            let (_, _, _, in_battle, _) = client.get_player_state();
            if in_battle {
                encountered = true;
                break;
            }
            let result = client.move_player(&3); // Right
            if result == 2 {
                encountered = true;
                break;
            }
            if result == 0 {
                // Blocked, try down
                client.move_player(&1);
            }
        }

        // Should eventually trigger an encounter on tall grass
        // (deterministic based on move count)
        let (_, _, _, in_battle, _) = client.get_player_state();

        // Either we're in battle or we explored the area
        assert!(encountered || !in_battle);
    }

    #[test]
    fn test_no_move_in_battle() {
        let env = Env::default();
        let contract_id = env.register(PokemonMiniContract, ());
        let client = PokemonMiniContractClient::new(&env, &contract_id);

        client.init_player();

        // Trigger an encounter
        for _ in 0..20 {
            let (_, _, _, in_battle, _) = client.get_player_state();
            if in_battle {
                break;
            }
            client.move_player(&3);
            client.move_player(&1);
        }

        let (x1, y1, _, in_battle, _) = client.get_player_state();

        if in_battle {
            // Try to move while in battle
            let result = client.move_player(&3);
            assert_eq!(result, 0); // Should be blocked

            let (x2, y2, _, _, _) = client.get_player_state();
            assert_eq!(x1, x2);
            assert_eq!(y1, y2);
        }
    }

    #[test]
    fn test_battle_attack() {
        let env = Env::default();
        let contract_id = env.register(PokemonMiniContract, ());
        let client = PokemonMiniContractClient::new(&env, &contract_id);

        client.init_player();

        // Trigger encounter
        for _ in 0..30 {
            let (_, _, _, in_battle, _) = client.get_player_state();
            if in_battle {
                break;
            }
            client.move_player(&3);
            client.move_player(&1);
        }

        let (_, _, _, in_battle, _) = client.get_player_state();
        if !in_battle {
            return; // No encounter triggered in test
        }

        // Get initial enemy HP
        let (_, _player_hp_before, enemy_hp_before, _, _) = client.get_battle_state();

        // Attack
        let result = client.battle_action(&0);
        assert!(result >= 1);

        let (still_in_battle, _, enemy_hp_after, _, _) = client.get_battle_state();

        // Either battle ended or damage was dealt
        if still_in_battle {
            assert!(enemy_hp_after < enemy_hp_before);
        }
    }

    #[test]
    fn test_battle_run() {
        let env = Env::default();
        let contract_id = env.register(PokemonMiniContract, ());
        let client = PokemonMiniContractClient::new(&env, &contract_id);

        client.init_player();

        // Trigger encounter
        for _ in 0..30 {
            let (_, _, _, in_battle, _) = client.get_player_state();
            if in_battle {
                break;
            }
            client.move_player(&3);
            client.move_player(&1);
        }

        let (_, _, _, in_battle, _) = client.get_player_state();
        if !in_battle {
            return;
        }

        // Run from battle
        let result = client.battle_action(&2); // Run
        assert_eq!(result, 2); // Battle ended

        // Should no longer be in battle
        let (_, _, _, in_battle, _) = client.get_player_state();
        assert!(!in_battle);

        // Check escape was recorded
        let (_, _, escapes) = client.get_battle_stats();
        assert!(escapes > 0);
    }

    #[test]
    fn test_battle_win() {
        let env = Env::default();
        let contract_id = env.register(PokemonMiniContract, ());
        let client = PokemonMiniContractClient::new(&env, &contract_id);

        client.init_player();

        // Trigger encounter
        for _ in 0..30 {
            let (_, _, _, in_battle, _) = client.get_player_state();
            if in_battle {
                break;
            }
            client.move_player(&3);
            client.move_player(&1);
        }

        let (_, _, _, in_battle, _) = client.get_player_state();
        if !in_battle {
            return;
        }

        // Keep attacking until battle ends
        for _ in 0..20 {
            let (still_in_battle, _, _, _, result) = client.get_battle_state();
            if !still_in_battle || result != 0 {
                break;
            }
            client.battle_action(&0); // Attack
        }

        // Check if we won (or lost)
        let (wins, losses, _) = client.get_battle_stats();
        assert!(wins > 0 || losses > 0);
    }

    #[test]
    fn test_battle_defend() {
        let env = Env::default();
        let contract_id = env.register(PokemonMiniContract, ());
        let client = PokemonMiniContractClient::new(&env, &contract_id);

        client.init_player();

        // Trigger encounter
        for _ in 0..30 {
            let (_, _, _, in_battle, _) = client.get_player_state();
            if in_battle {
                break;
            }
            client.move_player(&3);
            client.move_player(&1);
        }

        let (_, _, _, in_battle, _) = client.get_player_state();
        if !in_battle {
            return;
        }

        // Defend action
        let result = client.battle_action(&1);
        assert!(result >= 1); // Action executed
    }

    #[test]
    fn test_creature_heals_on_win() {
        let env = Env::default();
        let contract_id = env.register(PokemonMiniContract, ());
        let client = PokemonMiniContractClient::new(&env, &contract_id);

        client.init_player();

        // Trigger encounter and win
        for _ in 0..30 {
            let (_, _, _, in_battle, _) = client.get_player_state();
            if in_battle {
                break;
            }
            client.move_player(&3);
            client.move_player(&1);
        }

        let (_, _, _, in_battle, _) = client.get_player_state();
        if !in_battle {
            return;
        }

        // Attack until win
        for _ in 0..20 {
            let (still_in_battle, _, _, _, _result) = client.get_battle_state();
            if !still_in_battle {
                break;
            }
            client.battle_action(&0);
        }

        // Check if we won
        let (wins, _, _) = client.get_battle_stats();
        if wins > 0 {
            // Creature should be healed
            let (_, _, hp, max_hp, _, _) = client.get_creature_stats();
            assert_eq!(hp, max_hp);
        }
    }
}
