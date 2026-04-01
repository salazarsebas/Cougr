#![no_std]

//! # Tower Defense On-Chain Game
//!
//! This example demonstrates how to build a tower defense game using the
//! `cougr-core` ECS framework on the Stellar blockchain via Soroban.
//!
//! ## Game Features
//!
//! - **Wave Spawning**: Enemies spawn in waves with increasing difficulty
//! - **Path Movement**: Enemies follow a deterministic path toward the base
//! - **Tower Placement**: Players can place towers to defend the path
//! - **Attack Resolution**: Towers automatically target and damage enemies
//! - **Win/Loss Conditions**: Survive all waves or lose when base health reaches 0
//!
//! ## Architecture
//!
//! This implementation uses an Entity-Component-System (ECS) pattern:
//! - **Entities**: Game state, Enemies, Towers
//! - **Components**: Position, Enemy stats, Tower stats, Wave state, Base health
//! - **Systems**: Wave spawning, Path progression, Targeting, Attack resolution
//!
//! The `cougr-core` package simplifies on-chain game development by providing:
//! - Serialization-ready component patterns for on-chain storage
//! - Entity management optimized for Soroban's constraints
//! - A consistent architecture for game logic

mod components;
mod systems;

use components::{BaseComponent, WaveComponent};
use cougr_core::SimpleWorld;
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env};

// ============================================================================
// Game State
// ============================================================================

/// Main game state stored in contract storage
#[contracttype]
#[derive(Clone, Debug)]
pub struct GameState {
    pub game_id: u32,
    pub base_health: u32,
    pub current_wave: u32,
    pub total_waves: u32,
    pub tick_count: u32,
    pub enemies_killed: u32,
    pub enemies_alive: u32,
    pub status: u32, // 0=active, 1=won, 2=lost
}

// ============================================================================
// Contract
// ============================================================================

/// Tower Defense game contract
#[contract]
pub struct TowerDefenseContract;

#[contractimpl]
impl TowerDefenseContract {
    // ========================================================================
    // Initialization
    // ========================================================================

    /// Initialize a new game
    ///
    /// Creates the game state with:
    /// - Base health at 100
    /// - Wave 1 ready to start
    /// - Game status: Active
    pub fn init_game(env: Env) {
        let mut world = SimpleWorld::new(&env);
        let game_id = systems::init_game(&mut world, &env);

        env.storage()
            .persistent()
            .set(&symbol_short!("game_id"), &game_id);
        env.storage()
            .persistent()
            .set(&symbol_short!("world"), &world);
    }

    // ========================================================================
    // Tower Placement
    // ========================================================================

    /// Place a tower at the specified coordinates
    ///
    /// # Arguments
    /// * `x` - X coordinate on the map
    /// * `y` - Y coordinate on the map
    /// * `tower_kind` - Type of tower (0=Basic, 1=Sniper, 2=Splash)
    ///
    /// # Returns
    /// `true` if tower was placed successfully, `false` otherwise
    pub fn place_tower(env: Env, x: u32, y: u32, tower_kind: u32) -> bool {
        let mut world: SimpleWorld = env
            .storage()
            .persistent()
            .get(&symbol_short!("world"))
            .unwrap();

        let result = systems::place_tower(&mut world, &env, x, y, tower_kind);

        env.storage()
            .persistent()
            .set(&symbol_short!("world"), &world);

        result.is_some()
    }

    // ========================================================================
    // Game Loop
    // ========================================================================

    /// Advance the game by one tick
    ///
    /// This executes all game systems in order:
    /// 1. Wave spawn system - spawns enemies
    /// 2. Path progression system - moves enemies
    /// 3. Attack resolution system - towers attack enemies
    /// 4. Base damage system - enemies reaching base deal damage
    /// 5. Wave advancement - check if wave is complete
    /// 6. End condition system - check win/loss
    pub fn advance_tick(env: Env) {
        let mut world: SimpleWorld = env
            .storage()
            .persistent()
            .get(&symbol_short!("world"))
            .unwrap();

        let game_id: u32 = env
            .storage()
            .persistent()
            .get(&symbol_short!("game_id"))
            .unwrap();

        // Check if game is still active
        if let Some(status) = systems::get_status(&world, game_id, &env) {
            if !status.is_active() {
                return;
            }
        }

        // Execute game systems
        systems::tick_system(&mut world, game_id, &env);
        systems::wave_spawn_system(&mut world, game_id, &env);
        systems::path_progression_system(&mut world, &env);
        systems::attack_resolution_system(&mut world, game_id, &env);
        systems::base_damage_system(&mut world, game_id, &env);
        systems::check_wave_advancement(&mut world, game_id, &env);
        systems::end_condition_system(&mut world, game_id, &env);

        env.storage()
            .persistent()
            .set(&symbol_short!("world"), &world);
    }

    // ========================================================================
    // State Queries
    // ========================================================================

    /// Get the current game state
    pub fn get_state(env: Env) -> GameState {
        let world: SimpleWorld = env
            .storage()
            .persistent()
            .get(&symbol_short!("world"))
            .unwrap();

        let game_id: u32 = env
            .storage()
            .persistent()
            .get(&symbol_short!("game_id"))
            .unwrap();

        let base = systems::get_base(&world, game_id, &env).unwrap_or(BaseComponent::new(0));
        let wave = systems::get_wave(&world, game_id, &env).unwrap_or(WaveComponent::new(0, 0));
        let status = systems::get_status(&world, game_id, &env).unwrap_or_default();
        let enemies_alive = systems::count_alive_enemies(&world, &env);

        GameState {
            game_id,
            base_health: base.health,
            current_wave: wave.current_wave,
            total_waves: wave.total_waves,
            tick_count: status.tick_count,
            enemies_killed: status.enemies_killed,
            enemies_alive,
            status: status.status.to_u8() as u32,
        }
    }

    /// Check if the game is finished (won or lost)
    pub fn is_finished(env: Env) -> bool {
        let world: SimpleWorld = env
            .storage()
            .persistent()
            .get(&symbol_short!("world"))
            .unwrap();

        let game_id: u32 = env
            .storage()
            .persistent()
            .get(&symbol_short!("game_id"))
            .unwrap();

        let status = systems::get_status(&world, game_id, &env).unwrap_or_default();

        !status.is_active()
    }

    /// Get the game result (0=active, 1=won, 2=lost)
    pub fn get_result(env: Env) -> u32 {
        let world: SimpleWorld = env
            .storage()
            .persistent()
            .get(&symbol_short!("world"))
            .unwrap();

        let game_id: u32 = env
            .storage()
            .persistent()
            .get(&symbol_short!("game_id"))
            .unwrap();

        let status = systems::get_status(&world, game_id, &env).unwrap_or_default();

        status.status.to_u8() as u32
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_init_game() {
        let env = Env::default();
        let contract_id = env.register(TowerDefenseContract, ());
        let client = TowerDefenseContractClient::new(&env, &contract_id);

        client.init_game();

        let state = client.get_state();
        assert_eq!(state.base_health, 100);
        assert_eq!(state.current_wave, 1);
        assert_eq!(state.total_waves, 5);
        assert_eq!(state.status, 0); // Active
        assert!(!client.is_finished());
    }

    #[test]
    fn test_place_tower() {
        let env = Env::default();
        let contract_id = env.register(TowerDefenseContract, ());
        let client = TowerDefenseContractClient::new(&env, &contract_id);

        client.init_game();

        // Place a basic tower
        let result = client.place_tower(&3, &3, &0);
        assert!(result);

        // Try to place on path (should fail)
        let result = client.place_tower(&0, &5, &0);
        assert!(!result);
    }

    #[test]
    fn test_enemy_spawning() {
        let env = Env::default();
        let contract_id = env.register(TowerDefenseContract, ());
        let client = TowerDefenseContractClient::new(&env, &contract_id);

        client.init_game();

        // Initial state - no enemies yet
        let state = client.get_state();
        assert_eq!(state.enemies_alive, 0);

        // Advance tick to spawn first enemy
        client.advance_tick();

        let state = client.get_state();
        assert_eq!(state.enemies_alive, 1);
        assert_eq!(state.tick_count, 1);
    }

    #[test]
    fn test_path_progression() {
        let env = Env::default();
        let contract_id = env.register(TowerDefenseContract, ());
        let client = TowerDefenseContractClient::new(&env, &contract_id);

        client.init_game();

        // Spawn an enemy
        client.advance_tick();

        // Enemy should start at path index 0 and move each tick
        // After multiple ticks, enemy should progress along path
        for _ in 0..5 {
            client.advance_tick();
        }

        let state = client.get_state();
        // Should have spawned more enemies and some may have progressed
        assert!(state.tick_count >= 5);
    }

    #[test]
    fn test_tower_attacks() {
        let env = Env::default();
        let contract_id = env.register(TowerDefenseContract, ());
        let client = TowerDefenseContractClient::new(&env, &contract_id);

        client.init_game();

        // Place a tower near the path start
        client.place_tower(&1, &5, &0);

        // Spawn enemy and let tower attack
        client.advance_tick();
        client.advance_tick();
        client.advance_tick();

        // Tower should have attacked the enemy
        let state = client.get_state();
        assert!(state.tick_count >= 3);
    }

    #[test]
    fn test_base_damage_and_loss() {
        let env = Env::default();
        let contract_id = env.register(TowerDefenseContract, ());
        let client = TowerDefenseContractClient::new(&env, &contract_id);

        client.init_game();

        // Run many ticks without towers - enemies will reach base
        for _ in 0..100 {
            client.advance_tick();
            if client.is_finished() {
                break;
            }
        }

        let state = client.get_state();
        // Either lost (status=2) or base health reduced
        assert!(state.base_health < 100 || state.status == 2);
    }

    #[test]
    fn test_win_condition() {
        let env = Env::default();
        let contract_id = env.register(TowerDefenseContract, ());
        let client = TowerDefenseContractClient::new(&env, &contract_id);

        client.init_game();

        // Place powerful towers along the path
        client.place_tower(&1, &4, &1); // Sniper
        client.place_tower(&3, &1, &1); // Sniper
        client.place_tower(&4, &6, &1); // Sniper
        client.place_tower(&7, &6, &1); // Sniper
        client.place_tower(&7, &4, &1); // Sniper

        // Run game until finished
        for _ in 0..500 {
            client.advance_tick();
            if client.is_finished() {
                break;
            }
        }

        let state = client.get_state();
        // With good tower placement, should win (status=1)
        // or at least kill many enemies
        assert!(state.enemies_killed > 0 || state.status == 1);
    }

    #[test]
    fn test_multiple_tower_types() {
        let env = Env::default();
        let contract_id = env.register(TowerDefenseContract, ());
        let client = TowerDefenseContractClient::new(&env, &contract_id);

        client.init_game();

        // Place different tower types
        let basic = client.place_tower(&1, &4, &0);
        let sniper = client.place_tower(&3, &1, &1);
        let splash = client.place_tower(&4, &6, &2);

        assert!(basic);
        assert!(sniper);
        assert!(splash);

        // Run a few ticks
        for _ in 0..10 {
            client.advance_tick();
        }

        let state = client.get_state();
        assert!(state.tick_count >= 10);
    }

    #[test]
    fn test_invalid_tower_placement() {
        let env = Env::default();
        let contract_id = env.register(TowerDefenseContract, ());
        let client = TowerDefenseContractClient::new(&env, &contract_id);

        client.init_game();

        // Invalid tower kind
        let result = client.place_tower(&3, &3, &99);
        assert!(!result);

        // Out of bounds
        let result = client.place_tower(&100, &100, &0);
        assert!(!result);
    }

    #[test]
    fn test_game_state_persistence() {
        let env = Env::default();
        let contract_id = env.register(TowerDefenseContract, ());
        let client = TowerDefenseContractClient::new(&env, &contract_id);

        client.init_game();

        client.place_tower(&3, &3, &0);
        client.advance_tick();

        let state1 = client.get_state();

        client.advance_tick();

        let state2 = client.get_state();

        assert!(state2.tick_count > state1.tick_count);
    }
}
