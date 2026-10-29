use cougr_core::component::ComponentTrait;
use cougr_core::SimpleWorld;
use soroban_sdk::{Env, Vec};

use crate::components::{
    BombComponent, CellType, ExplosionComponent, GameStateComponent, GridComponent,
    PlayerComponent, PowerUpComponent, PowerUpType, GRID_HEIGHT, GRID_WIDTH,
};

// ── MovementSystem ────────────────────────────────────────────────────────────

/// Translates a direction code into a (dx, dy) delta.
/// Returns `None` for invalid direction codes.
pub(crate) fn direction_delta(direction: u32) -> Option<(i32, i32)> {
    match direction {
        0 => Some((0, -1)), // Up
        1 => Some((1, 0)),  // Right
        2 => Some((0, 1)),  // Down
        3 => Some((-1, 0)), // Left
        _ => None,
    }
}

// ── PickupSystem ──────────────────────────────────────────────────────────────

/// Checks whether a power-up entity exists at `(x, y)`.  If so, applies the
/// buff to `player`, despawns the entity, and returns `true`.
pub(crate) fn pickup_system(
    env: &Env,
    world: &mut SimpleWorld,
    player: &mut PlayerComponent,
    x: i32,
    y: i32,
) {
    let powerup_entities =
        world.get_entities_with_component(&PowerUpComponent::component_type(), env);
    let mut pickup_id_opt = None;
    let mut pickup_type_opt = None;
    for pu_id in powerup_entities.iter() {
        if let Some(pu) = world.get_typed::<PowerUpComponent>(env, pu_id) {
            if pu.x == x && pu.y == y {
                pickup_id_opt = Some(pu_id);
                pickup_type_opt = Some(pu.power_up_type);
                break;
            }
        }
    }
    if let (Some(pu_id), Some(pu_type)) = (pickup_id_opt, pickup_type_opt) {
        match pu_type {
            PowerUpType::Capacity => player.bomb_capacity += 1,
            PowerUpType::Power => player.bomb_power += 1,
            PowerUpType::Speed => player.speed += 1,
        }
        world.despawn_entity(pu_id);
    }
}

// ── ExplosionPropagationSystem ────────────────────────────────────────────────

/// Detonates a bomb: spawns explosion entities at the bomb's position and in
/// all four cardinal directions up to `bomb.power` cells, stopping at walls
/// and destroying destructible blocks.  Also runs `PowerUpSpawnSystem` on
/// destroyed blocks.
pub(crate) fn detonate_bomb(
    env: &Env,
    world: &mut SimpleWorld,
    grid: &mut GridComponent,
    bomb: &BombComponent,
) {
    // Spawn center explosion
    let center_exp = ExplosionComponent::new(bomb.x, bomb.y);
    let center_id = world.spawn_entity();
    world.set_typed(env, center_id, &center_exp);

    let dirs = [(0i32, -1i32), (1, 0), (0, 1), (-1, 0)];
    for (dx, dy) in dirs {
        for dist in 1..=bomb.power {
            let x = bomb.x + dx * dist as i32;
            let y = bomb.y + dy * dist as i32;

            if x < 0 || y < 0 || x >= GRID_WIDTH as i32 || y >= GRID_HEIGHT as i32 {
                break;
            }

            let cell = grid.get_cell(x as usize, y as usize);
            if cell == CellType::Wall {
                break;
            }

            let exp = ExplosionComponent::new(x, y);
            let exp_id = world.spawn_entity();
            world.set_typed(env, exp_id, &exp);

            if cell == CellType::Destructible {
                grid.set_cell(x as usize, y as usize, CellType::Empty);
                // PowerUpSpawnSystem: ~25% drop rate on destroyed blocks
                powerup_spawn_system(env, world, x, y);
                break;
            }

            if cell == CellType::PowerUp {
                grid.set_cell(x as usize, y as usize, CellType::Empty);
            }
        }
    }
}

// ── PowerUpSpawnSystem ────────────────────────────────────────────────────────

/// Deterministically spawns a power-up entity at `(x, y)` when `(x+y)%4==0`.
pub(crate) fn powerup_spawn_system(env: &Env, world: &mut SimpleWorld, x: i32, y: i32) {
    if (x + y) % 4 == 0 {
        let pu_type = match (x + y) % 3 {
            0 => PowerUpType::Capacity,
            1 => PowerUpType::Power,
            _ => PowerUpType::Speed,
        };
        let pu = PowerUpComponent::new(x, y, pu_type);
        let pu_entity = world.spawn_entity();
        world.set_typed(env, pu_entity, &pu);
    }
}

// ── BombTimerSystem + ChainReactionSystem ─────────────────────────────────────

/// Decrements all bomb timers.  Bombs that reach zero are detonated.
/// After each detonation, any remaining live bomb overlapping a fresh explosion
/// is immediately queued for chain detonation in the same tick.
pub(crate) fn bomb_timer_and_chain_system(
    env: &Env,
    world: &mut SimpleWorld,
    grid: &mut GridComponent,
) {
    let bomb_entities = world.get_entities_with_component(&BombComponent::component_type(), env);
    let mut detonation_queue: Vec<BombComponent> = Vec::new(env);

    for b_id in bomb_entities.iter() {
        if let Some(mut bomb) = world.get_typed::<BombComponent>(env, b_id) {
            if bomb.timer > 0 {
                bomb.timer -= 1;
            }
            if bomb.timer == 0 {
                detonation_queue.push_back(bomb);
                world.despawn_entity(b_id);
            } else {
                world.set_typed(env, b_id, &bomb);
            }
        }
    }

    let mut head = 0u32;
    while head < detonation_queue.len() {
        let bomb = detonation_queue.get(head).unwrap();
        head += 1;

        detonate_bomb(env, world, grid, &bomb);

        // ChainReactionSystem: check remaining live bombs for overlap with new explosions
        let remaining = world.get_entities_with_component(&BombComponent::component_type(), env);
        let active_expl =
            world.get_entities_with_component(&ExplosionComponent::component_type(), env);

        for r_id in remaining.iter() {
            if let Some(remaining_bomb) = world.get_typed::<BombComponent>(env, r_id) {
                let mut chain = false;
                for e_id in active_expl.iter() {
                    if let Some(expl) = world.get_typed::<ExplosionComponent>(env, e_id) {
                        if expl.x == remaining_bomb.x && expl.y == remaining_bomb.y {
                            chain = true;
                            break;
                        }
                    }
                }
                if chain {
                    detonation_queue.push_back(remaining_bomb);
                    world.despawn_entity(r_id);
                }
            }
        }
    }
}

// ── ExplosionTimerSystem ──────────────────────────────────────────────────────

/// Decrements explosion timers and despawns expired explosions.
pub(crate) fn explosion_timer_system(env: &Env, world: &mut SimpleWorld) {
    let explosion_entities =
        world.get_entities_with_component(&ExplosionComponent::component_type(), env);
    for e_id in explosion_entities.iter() {
        if let Some(mut explosion) = world.get_typed::<ExplosionComponent>(env, e_id) {
            if explosion.timer > 0 {
                explosion.timer -= 1;
            }
            if explosion.timer == 0 {
                world.despawn_entity(e_id);
            } else {
                world.set_typed(env, e_id, &explosion);
            }
        }
    }
}

// ── PlayerDeathSystem ─────────────────────────────────────────────────────────

/// Checks each player against active explosions and decrements lives on hit.
pub(crate) fn player_death_system(env: &Env, world: &mut SimpleWorld) {
    let player_entities =
        world.get_entities_with_component(&PlayerComponent::component_type(), env);
    let active_explosions =
        world.get_entities_with_component(&ExplosionComponent::component_type(), env);

    for p_id in player_entities.iter() {
        if let Some(mut player) = world.get_typed::<PlayerComponent>(env, p_id) {
            if player.lives == 0 {
                continue;
            }
            let mut hit = false;
            for e_id in active_explosions.iter() {
                if let Some(explosion) = world.get_typed::<ExplosionComponent>(env, e_id) {
                    if explosion.x == player.x && explosion.y == player.y {
                        hit = true;
                        break;
                    }
                }
            }
            if hit {
                player.lives -= 1;
                world.set_typed(env, p_id, &player);
            }
        }
    }
}

// ── WinConditionSystem ────────────────────────────────────────────────────────

/// Checks alive player count and updates `GameStateComponent` accordingly.
pub(crate) fn win_condition_system(
    env: &Env,
    world: &mut SimpleWorld,
    game_state: &mut GameStateComponent,
) {
    let player_entities =
        world.get_entities_with_component(&PlayerComponent::component_type(), env);
    let mut alive_players = 0u32;
    let mut last_alive_id = 0u32;
    for p_id in player_entities.iter() {
        if let Some(player) = world.get_typed::<PlayerComponent>(env, p_id) {
            if player.lives > 0 {
                alive_players += 1;
                last_alive_id = player.id;
            }
        }
    }
    if alive_players <= 1 {
        game_state.game_over = true;
        if alive_players == 1 {
            game_state.winner_id = Some(last_alive_id);
        }
    }
}
