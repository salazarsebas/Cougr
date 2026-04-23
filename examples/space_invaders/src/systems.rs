use crate::components::{
    BulletTag, Health, InvaderTag, Position, Velocity,
    BULLET_SPEED, GAME_HEIGHT, GAME_WIDTH, INVADER_COLS, INVADER_MOVE_INTERVAL,
    INVADER_WIN_Y,
};
use cougr_core::{ComponentTrait, SimpleWorld};
use soroban_sdk::{symbol_short, Env};

/// Movement system: applies each bullet's Velocity to its Position and
/// despawns out-of-bounds bullets.
pub fn movement_system(world: &mut SimpleWorld, env: &Env) {
    let bullets = world.get_entities_with_component(&symbol_short!("bullet"), env);
    for i in 0..bullets.len() {
        let id = bullets.get(i).unwrap();
        let pos_data = match world.get_component(id, &symbol_short!("position")) {
            Some(d) => d,
            None => continue,
        };
        let vel_data = match world.get_component(id, &symbol_short!("velocity")) {
            Some(d) => d,
            None => continue,
        };
        let mut pos = Position::deserialize(env, &pos_data).unwrap();
        let vel = Velocity::deserialize(env, &vel_data).unwrap();
        pos.x += vel.dx;
        pos.y += vel.dy;

        if pos.y <= 0 || pos.y >= GAME_HEIGHT {
            world.despawn_entity(id);
        } else {
            world.set_typed(env, id, &pos);
        }
    }
}

/// Collision system: checks player bullets vs invaders and enemy bullets vs ship.
/// Returns (score_gained, ship_was_hit).
pub fn collision_system(world: &mut SimpleWorld, env: &Env) -> (u32, bool) {
    let mut score = 0u32;
    let mut ship_hit = false;

    let bullets = world.get_entities_with_component(&symbol_short!("bullet"), env);
    let invaders = world.get_entities_with_component(&symbol_short!("invader"), env);
    let ships = world.get_entities_with_component(&symbol_short!("ship"), env);

    let ship_id = if !ships.is_empty() { Some(ships.get(0).unwrap()) } else { None };
    let ship_pos: Option<Position> = ship_id.and_then(|id| {
        world.get_component(id, &symbol_short!("position"))
            .and_then(|d| Position::deserialize(env, &d))
    });

    for bi in 0..bullets.len() {
        let bid = bullets.get(bi).unwrap();
        let btag_data = match world.get_component(bid, &symbol_short!("bullet")) {
            Some(d) => d,
            None => continue,
        };
        let btag = BulletTag::deserialize(env, &btag_data).unwrap();
        let bpos_data = match world.get_component(bid, &symbol_short!("position")) {
            Some(d) => d,
            None => continue,
        };
        let bpos = Position::deserialize(env, &bpos_data).unwrap();

        if btag.owner == 0 {
            // Player bullet — check against invaders
            for ii in 0..invaders.len() {
                let iid = invaders.get(ii).unwrap();
                let itag_data = match world.get_component(iid, &symbol_short!("invader")) {
                    Some(d) => d,
                    None => continue,
                };
                let mut itag = InvaderTag::deserialize(env, &itag_data).unwrap();
                if !itag.active { continue; }
                let ipos_data = match world.get_component(iid, &symbol_short!("position")) {
                    Some(d) => d,
                    None => continue,
                };
                let ipos = Position::deserialize(env, &ipos_data).unwrap();
                if hits(&bpos, &ipos, 2) {
                    score += itag.points;
                    itag.active = false;
                    world.set_typed(env, iid, &itag);
                    world.despawn_entity(bid);
                    break;
                }
            }
        } else {
            // Enemy bullet — check against ship
            if let (Some(sid), Some(ref sp)) = (ship_id, &ship_pos) {
                if hits(&bpos, sp, 2) {
                    let health_data = world.get_component(sid, &symbol_short!("health")).unwrap();
                    let mut hp = Health::deserialize(env, &health_data).unwrap();
                    hp.take_damage();
                    world.set_typed(env, sid, &hp);
                    world.despawn_entity(bid);
                    ship_hit = true;
                }
            }
        }
    }

    (score, ship_hit)
}

/// Invader movement system: moves the formation and descends when hitting a wall.
/// Returns true if any invader reached the player's row (game over).
pub fn invader_movement_system(
    world: &mut SimpleWorld,
    env: &Env,
    tick: u32,
    direction: &mut i32,
) -> bool {
    if tick % INVADER_MOVE_INTERVAL != 0 {
        return false;
    }

    let invaders = world.get_entities_with_component(&symbol_short!("invader"), env);
    let mut should_descend = false;

    for i in 0..invaders.len() {
        let id = invaders.get(i).unwrap();
        let itag_data = world.get_component(id, &symbol_short!("invader")).unwrap();
        let itag = InvaderTag::deserialize(env, &itag_data).unwrap();
        if !itag.active { continue; }
        let pos_data = world.get_component(id, &symbol_short!("position")).unwrap();
        let pos = Position::deserialize(env, &pos_data).unwrap();
        let nx = pos.x + *direction;
        if nx <= 0 || nx >= GAME_WIDTH - 1 {
            should_descend = true;
            break;
        }
    }

    let mut game_over = false;
    for i in 0..invaders.len() {
        let id = invaders.get(i).unwrap();
        let itag_data = world.get_component(id, &symbol_short!("invader")).unwrap();
        let itag = InvaderTag::deserialize(env, &itag_data).unwrap();
        if !itag.active { continue; }
        let pos_data = world.get_component(id, &symbol_short!("position")).unwrap();
        let mut pos = Position::deserialize(env, &pos_data).unwrap();
        if should_descend {
            pos.y += 1;
        } else {
            pos.x += *direction;
        }
        if pos.y >= INVADER_WIN_Y {
            game_over = true;
        }
        world.set_typed(env, id, &pos);
    }

    if should_descend {
        *direction *= -1;
    }

    game_over
}

/// Enemy shoot system: one active invader fires a bullet each interval.
pub fn enemy_shoot_system(world: &mut SimpleWorld, env: &Env, tick: u32) {
    if tick % 7 != 0 { return; }

    let invaders = world.get_entities_with_component(&symbol_short!("invader"), env);
    let col_target = (tick / 7) % INVADER_COLS;

    for i in 0..invaders.len() {
        let id = invaders.get(i).unwrap();
        let itag_data = world.get_component(id, &symbol_short!("invader")).unwrap();
        let itag = InvaderTag::deserialize(env, &itag_data).unwrap();
        if !itag.active { continue; }
        if i % INVADER_COLS != col_target { continue; }
        let pos_data = world.get_component(id, &symbol_short!("position")).unwrap();
        let pos = Position::deserialize(env, &pos_data).unwrap();
        spawn_bullet(world, env, pos.x, pos.y + 1, 1);
        break;
    }
}

// ── Spawn helpers ─────────────────────────────────────────────────────────────

pub fn spawn_bullet(world: &mut SimpleWorld, env: &Env, x: i32, y: i32, owner: u8) {
    let id = world.spawn_entity();
    let dy = if owner == 0 { -BULLET_SPEED } else { BULLET_SPEED };
    world.set_typed(env, id, &Position::new(x, y));
    world.set_typed(env, id, &Velocity::new(0, dy));
    world.set_typed(env, id, &BulletTag { owner });
}

// ── Utility ───────────────────────────────────────────────────────────────────

fn hits(a: &Position, b: &Position, tol: i32) -> bool {
    (a.x - b.x).abs() < tol && (a.y - b.y).abs() < tol
}
