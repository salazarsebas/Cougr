#![no_std]

mod components;
mod systems;

#[cfg(test)]
mod test;

use components::{
    Health, InvaderTag, InvaderType, Position, ShipTag,
    INVADER_COLS, INVADER_ROWS, SHIP_Y,
};
use cougr_core::{ComponentTrait, SimpleWorld};
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env};
use systems::{
    collision_system, enemy_shoot_system, invader_movement_system, movement_system, spawn_bullet,
};

// Re-export constants used by tests
pub use components::{GAME_WIDTH, GAME_HEIGHT, SHOOT_COOLDOWN};

// ── Lightweight metadata stored alongside the world ───────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct GameMeta {
    pub score: u32,
    pub game_over: bool,
    pub tick: u32,
    pub shoot_cooldown: u32,
    pub invader_direction: i32,
    pub ship_entity: u32,
}

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct SpaceInvadersContract;

#[contractimpl]
impl SpaceInvadersContract {
    /// Initialize a new game.
    pub fn init_game(env: Env) {
        let mut world = SimpleWorld::new(&env);

        // Spawn player ship
        let ship_id = world.spawn_entity();
        world.set_typed(&env, ship_id, &Position::new(GAME_WIDTH / 2, SHIP_Y));
        world.set_typed(&env, ship_id, &Health::new(3));
        world.set_typed(&env, ship_id, &ShipTag);

        // Spawn invader grid
        for row in 0..INVADER_ROWS {
            let inv_type = match row {
                0 => InvaderType::Squid,
                1 | 2 => InvaderType::Crab,
                _ => InvaderType::Octopus,
            };
            for col in 0..INVADER_COLS {
                let id = world.spawn_entity();
                let x = (col as i32 * 4) + 4;
                let y = (row as i32 * 3) + 2;
                world.set_typed(&env, id, &Position::new(x, y));
                world.set_typed(&env, id, &InvaderTag { points: inv_type.points(), active: true });
            }
        }

        let meta = GameMeta {
            score: 0,
            game_over: false,
            tick: 0,
            shoot_cooldown: 0,
            invader_direction: 1,
            ship_entity: ship_id,
        };

        save_world(&env, &world);
        save_meta(&env, &meta);
    }

    /// Move the player ship. direction: -1 = left, 1 = right. Returns new X.
    pub fn move_ship(env: Env, direction: i32) -> i32 {
        let meta = load_meta(&env);
        if meta.game_over { return Self::get_ship_position(env); }

        let mut world = load_world(&env);
        let sid = meta.ship_entity;
        let pos_data = world.get_component(sid, &symbol_short!("position")).unwrap();
        let mut pos = Position::deserialize(&env, &pos_data).unwrap();

        let nx = pos.x + direction;
        if (1..GAME_WIDTH - 1).contains(&nx) {
            pos.x = nx;
            world.set_typed(&env, sid, &pos);
            save_world(&env, &world);
        }
        pos.x
    }

    /// Fire a player bullet. Returns true if fired, false if on cooldown.
    pub fn shoot(env: Env) -> bool {
        let mut meta = load_meta(&env);
        if meta.game_over || meta.shoot_cooldown > 0 { return false; }

        let mut world = load_world(&env);
        let sid = meta.ship_entity;
        let pos_data = world.get_component(sid, &symbol_short!("position")).unwrap();
        let pos = Position::deserialize(&env, &pos_data).unwrap();

        spawn_bullet(&mut world, &env, pos.x, SHIP_Y - 1, 0);
        meta.shoot_cooldown = SHOOT_COOLDOWN;

        save_world(&env, &world);
        save_meta(&env, &meta);
        true
    }

    /// Advance the game by one tick. Returns true while the game is running.
    pub fn update_tick(env: Env) -> bool {
        let mut meta = load_meta(&env);
        if meta.game_over { return false; }

        meta.tick += 1;
        if meta.shoot_cooldown > 0 { meta.shoot_cooldown -= 1; }

        let mut world = load_world(&env);

        // Movement system
        movement_system(&mut world, &env);

        // Collision system
        let (gained, ship_hit) = collision_system(&mut world, &env);
        meta.score += gained;

        if ship_hit {
            let hp_data = world.get_component(meta.ship_entity, &symbol_short!("health")).unwrap();
            let hp = Health::deserialize(&env, &hp_data).unwrap();
            if !hp.is_alive() {
                meta.game_over = true;
            }
        }

        // Invader movement system
        if invader_movement_system(&mut world, &env, meta.tick, &mut meta.invader_direction) {
            meta.game_over = true;
        }

        // Enemy shoot system
        enemy_shoot_system(&mut world, &env, meta.tick);

        // Win condition: all invaders destroyed
        if Self::count_active_invaders(&world, &env) == 0 {
            meta.game_over = true;
        }

        save_world(&env, &world);
        save_meta(&env, &meta);
        !meta.game_over
    }

    /// Current score.
    pub fn get_score(env: Env) -> u32 { load_meta(&env).score }

    /// Remaining lives (ship health).
    pub fn get_lives(env: Env) -> u32 {
        let meta = load_meta(&env);
        let world = load_world(&env);
        world.get_component(meta.ship_entity, &symbol_short!("health"))
            .and_then(|d| Health::deserialize(&env, &d))
            .map(|h| h.current)
            .unwrap_or(0)
    }

    /// Ship X position.
    pub fn get_ship_position(env: Env) -> i32 {
        let meta = load_meta(&env);
        let world = load_world(&env);
        world.get_component(meta.ship_entity, &symbol_short!("position"))
            .and_then(|d| Position::deserialize(&env, &d))
            .map(|p| p.x)
            .unwrap_or(GAME_WIDTH / 2)
    }

    /// Whether the game is over.
    pub fn check_game_over(env: Env) -> bool { load_meta(&env).game_over }

    /// Number of active invaders remaining.
    pub fn get_active_invaders(env: Env) -> u32 {
        let world = load_world(&env);
        Self::count_active_invaders(&world, &env)
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn count_active_invaders(world: &SimpleWorld, env: &Env) -> u32 {
        let invaders = world.get_entities_with_component(&symbol_short!("invader"), env);
        let mut count = 0u32;
        for i in 0..invaders.len() {
            let id = invaders.get(i).unwrap();
            if let Some(d) = world.get_component(id, &symbol_short!("invader")) {
                if let Some(tag) = InvaderTag::deserialize(env, &d) {
                    if tag.active { count += 1; }
                }
            }
        }
        count
    }
}

// ── Storage helpers ───────────────────────────────────────────────────────────

fn save_world(env: &Env, world: &SimpleWorld) {
    env.storage().instance().set(&symbol_short!("world"), world);
}

fn load_world(env: &Env) -> SimpleWorld {
    env.storage()
        .instance()
        .get(&symbol_short!("world"))
        .expect("Game not initialized")
}

fn save_meta(env: &Env, meta: &GameMeta) {
    env.storage().instance().set(&symbol_short!("meta"), meta);
}

fn load_meta(env: &Env) -> GameMeta {
    env.storage()
        .instance()
        .get(&symbol_short!("meta"))
        .expect("Game not initialized")
}
