#![no_std]
use cougr_core::component::ComponentTrait;
use cougr_core::*;
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env, Symbol};

mod components;
mod systems;
#[cfg(test)]
mod test;

pub use components::{
    BombComponent, CellType, ExplosionComponent, GameStateComponent, GridComponent,
    PlayerComponent, PowerUpComponent, PowerUpType, BOMB_TIMER, GRID_HEIGHT, GRID_WIDTH,
    INITIAL_LIVES,
};
use systems::{
    bomb_timer_and_chain_system, direction_delta, explosion_timer_system, pickup_system,
    player_death_system, win_condition_system,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    World,
}

#[contract]
pub struct BombermanContract;

#[allow(unused_variables)]
#[contractimpl]
impl BombermanContract {
    /// Initialize the game world.
    pub fn init_game(env: Env) -> Symbol {
        let mut world = SimpleWorld::new(&env);

        let grid = GridComponent::new(&env);
        let grid_entity = world.spawn_entity();
        world.set_typed(&env, grid_entity, &grid);

        let game_state = GameStateComponent::new();
        let game_state_entity = world.spawn_entity();
        world.set_typed(&env, game_state_entity, &game_state);

        // PowerUpSpawnSystem (init): place power-up entities at deterministic positions.
        for x in 1..GRID_WIDTH - 1 {
            for y in 1..GRID_HEIGHT - 1 {
                if (x + y) % 7 == 0 && grid.get_cell(x, y) == CellType::Empty {
                    let pu_type = match (x + y) % 3 {
                        0 => PowerUpType::Capacity,
                        1 => PowerUpType::Power,
                        _ => PowerUpType::Speed,
                    };
                    let pu = PowerUpComponent::new(x as i32, y as i32, pu_type);
                    let pu_entity = world.spawn_entity();
                    world.set_typed(&env, pu_entity, &pu);
                }
            }
        }

        env.storage().instance().set(&DataKey::World, &world);
        symbol_short!("init")
    }

    /// Spawn a new player at a given position.
    pub fn spawn_player(env: Env, player_id: u32, x: i32, y: i32) -> Symbol {
        let mut world = Self::get_world(&env);

        let player_entities =
            world.get_entities_with_component(&PlayerComponent::component_type(), &env);
        for entity_id in player_entities.iter() {
            if let Some(player) = world.get_typed::<PlayerComponent>(&env, entity_id) {
                if player.id == player_id {
                    return symbol_short!("exists");
                }
            }
        }

        let player = PlayerComponent::new(player_id, x, y);
        let player_entity = world.spawn_entity();
        world.set_typed(&env, player_entity, &player);

        env.storage().instance().set(&DataKey::World, &world);
        symbol_short!("spawned")
    }

    /// Move a player in the specified direction (0=up, 1=right, 2=down, 3=left).
    pub fn move_player(env: Env, player_id: u32, direction: u32) -> Symbol {
        let mut world = Self::get_world(&env);

        // Find player entity
        let player_entities =
            world.get_entities_with_component(&PlayerComponent::component_type(), &env);
        let mut player_entity_opt = None;
        for entity_id in player_entities.iter() {
            if let Some(player) = world.get_typed::<PlayerComponent>(&env, entity_id) {
                if player.id == player_id {
                    player_entity_opt = Some((entity_id, player));
                    break;
                }
            }
        }

        let (player_entity, mut player) = match player_entity_opt {
            Some(p) => p,
            None => return symbol_short!("no_player"),
        };

        // Find grid entity
        let grid_entities =
            world.get_entities_with_component(&GridComponent::component_type(), &env);
        let grid_entity = match grid_entities.get(0) {
            Some(e) => e,
            None => return symbol_short!("no_grid"),
        };
        let grid = world.get_typed::<GridComponent>(&env, grid_entity).unwrap();

        // Calculate new position
        let (dx, dy) = match direction_delta(direction) {
            Some(d) => d,
            None => return symbol_short!("inv_dir"),
        };
        let next_x = player.x + dx;
        let next_y = player.y + dy;

        if !grid.is_walkable(next_x, next_y) {
            return symbol_short!("blocked");
        }

        // Check for explosions at target position
        let explosion_entities =
            world.get_entities_with_component(&ExplosionComponent::component_type(), &env);
        for e_id in explosion_entities.iter() {
            if let Some(explosion) = world.get_typed::<ExplosionComponent>(&env, e_id) {
                if explosion.x == next_x && explosion.y == next_y && player.lives > 0 {
                    player.lives -= 1;
                }
            }
        }

        player.x = next_x;
        player.y = next_y;

        // PickupSystem
        pickup_system(&env, &mut world, &mut player, next_x, next_y);

        world.set_typed(&env, player_entity, &player);
        env.storage().instance().set(&DataKey::World, &world);
        symbol_short!("moved")
    }

    /// Place a bomb at the player's current position.
    pub fn place_bomb(env: Env, player_id: u32) -> Symbol {
        let mut world = Self::get_world(&env);

        let player_entities =
            world.get_entities_with_component(&PlayerComponent::component_type(), &env);
        let mut player_opt = None;
        for entity_id in player_entities.iter() {
            if let Some(player) = world.get_typed::<PlayerComponent>(&env, entity_id) {
                if player.id == player_id {
                    player_opt = Some(player);
                    break;
                }
            }
        }

        let player = match player_opt {
            Some(p) => p,
            None => return symbol_short!("no_player"),
        };

        let bomb_entities =
            world.get_entities_with_component(&BombComponent::component_type(), &env);
        let mut owned_bombs = 0;
        for b_id in bomb_entities.iter() {
            if let Some(bomb) = world.get_typed::<BombComponent>(&env, b_id) {
                if bomb.owner_id == player_id {
                    owned_bombs += 1;
                }
            }
        }

        if owned_bombs >= player.bomb_capacity {
            return symbol_short!("cap_full");
        }

        for b_id in bomb_entities.iter() {
            if let Some(bomb) = world.get_typed::<BombComponent>(&env, b_id) {
                if bomb.x == player.x && bomb.y == player.y {
                    return symbol_short!("exists");
                }
            }
        }

        let mut bomb = BombComponent::new(player.x, player.y, player_id);
        bomb.power = player.bomb_power;
        let bomb_entity = world.spawn_entity();
        world.set_typed(&env, bomb_entity, &bomb);

        env.storage().instance().set(&DataKey::World, &world);
        symbol_short!("bomb_plc")
    }

    /// Advance the game tick - handle timers, explosions, and collisions.
    pub fn update_tick(env: Env) -> Symbol {
        let mut world = Self::get_world(&env);

        let state_entities =
            world.get_entities_with_component(&GameStateComponent::component_type(), &env);
        let state_entity = state_entities.get(0).expect("No game state");
        let mut game_state = world
            .get_typed::<GameStateComponent>(&env, state_entity)
            .unwrap();

        if game_state.game_over {
            return symbol_short!("game_over");
        }

        game_state.current_tick += 1;

        let grid_entities =
            world.get_entities_with_component(&GridComponent::component_type(), &env);
        let grid_entity = grid_entities.get(0).expect("No grid");
        let mut grid = world.get_typed::<GridComponent>(&env, grid_entity).unwrap();

        // 1. ExplosionTimerSystem
        explosion_timer_system(&env, &mut world);

        // 2. BombTimerSystem + ChainReactionSystem
        bomb_timer_and_chain_system(&env, &mut world, &mut grid);

        // 3. PlayerDeathSystem
        player_death_system(&env, &mut world);

        // 4. Persist updated grid
        world.set_typed(&env, grid_entity, &grid);

        // 5. WinConditionSystem
        win_condition_system(&env, &mut world, &mut game_state);

        world.set_typed(&env, state_entity, &game_state);
        env.storage().instance().set(&DataKey::World, &world);
        symbol_short!("tick_upd")
    }

    /// Get the current score for a player.
    pub fn get_score(env: Env, player_id: u32) -> u32 {
        let world = Self::get_world(&env);
        let player_entities =
            world.get_entities_with_component(&PlayerComponent::component_type(), &env);
        for entity_id in player_entities.iter() {
            if let Some(player) = world.get_typed::<PlayerComponent>(&env, entity_id) {
                if player.id == player_id {
                    return player.score;
                }
            }
        }
        0
    }

    /// Get the current lives for a player.
    pub fn get_lives(env: Env, player_id: u32) -> u32 {
        let world = Self::get_world(&env);
        let player_entities =
            world.get_entities_with_component(&PlayerComponent::component_type(), &env);
        for entity_id in player_entities.iter() {
            if let Some(player) = world.get_typed::<PlayerComponent>(&env, entity_id) {
                if player.id == player_id {
                    return player.lives;
                }
            }
        }
        0
    }

    /// Check if the game is over and return winner status.
    pub fn check_game_over(env: Env) -> Symbol {
        let world = Self::get_world(&env);
        let state_entities =
            world.get_entities_with_component(&GameStateComponent::component_type(), &env);
        let state_entity = match state_entities.get(0) {
            Some(e) => e,
            None => return symbol_short!("no_state"),
        };
        let game_state = world
            .get_typed::<GameStateComponent>(&env, state_entity)
            .unwrap();

        if game_state.game_over {
            match game_state.winner_id {
                Some(_) => symbol_short!("winner"),
                None => symbol_short!("draw"),
            }
        } else {
            symbol_short!("ongoing")
        }
    }

    pub fn hello(_env: Env, to: Symbol) -> Symbol {
        to
    }

    // ── Storage helper ────────────────────────────────────────────────────────

    fn get_world(env: &Env) -> SimpleWorld {
        env.storage()
            .instance()
            .get(&DataKey::World)
            .expect("Game not initialized")
    }
}
