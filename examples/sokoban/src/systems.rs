//! Pure gameplay logic for the Sokoban puzzle, kept out of the
//! `#[contractimpl]` entrypoints per this repo's "entrypoints stay thin"
//! guidance (see CLAUDE.md § Soroban and contract code).

use cougr_core::component::ComponentTrait;
use cougr_core::prelude::EntityId;
use cougr_core::{ArchetypeQueryBuilder, ArchetypeWorld};
use soroban_sdk::Env;

use crate::components::{Crate, Goal, LevelMeta, OnGoal, Position, Wall};

/// `init_level` always spawns the player first, and `ArchetypeWorld` hands
/// out entity ids starting at 1, so the player's id never needs a query.
pub const PLAYER_ID: EntityId = 1;

fn position_of(world: &ArchetypeWorld, env: &Env, entity: EntityId) -> Position {
    world
        .get_typed::<Position>(env, entity)
        .unwrap_or_else(|| panic!("entity has no position"))
}

/// Finds the entity carrying component `T` whose `Position` is `(x, y)`,
/// if any. Cougr does not index positions spatially, so a tile lookup is a
/// query over that component's archetypes followed by a linear scan.
fn entity_at<T: ComponentTrait>(
    world: &ArchetypeWorld,
    env: &Env,
    x: i32,
    y: i32,
) -> Option<EntityId> {
    let candidates = ArchetypeQueryBuilder::new()
        .with_component(T::component_type())
        .build()
        .execute(world, env);

    for i in 0..candidates.len() {
        let candidate = candidates.get(i).unwrap();
        let pos = position_of(world, env, candidate);
        if pos.x == x && pos.y == y {
            return Some(candidate);
        }
    }
    None
}

/// True if `(x, y)` lies within the declared `width x height` grid.
fn in_bounds(x: i32, y: i32, width: u32, height: u32) -> bool {
    x >= 0 && y >= 0 && (x as u32) < width && (y as u32) < height
}

/// True once every `Crate` entity also carries `OnGoal`: crates that are not
/// yet parked on a goal are exactly the entities keeping the level unsolved.
pub fn is_solved(world: &ArchetypeWorld, env: &Env) -> bool {
    let unsolved = ArchetypeQueryBuilder::new()
        .with_component(Crate::component_type())
        .without_component(OnGoal::component_type())
        .build()
        .execute(world, env);
    unsolved.is_empty()
}

/// Moves the player by one tile `(dx, dy)`, pushing at most one crate ahead
/// of it. Panics on every blocked case: the level is already solved, the
/// destination tile is a wall, or a crate would be pushed into a wall or
/// another crate. Auth is the caller's responsibility; this function only
/// knows about tiles and crates.
pub fn try_push(world: &mut ArchetypeWorld, env: &Env, dx: i32, dy: i32) {
    let mut meta: LevelMeta = world
        .get_typed(env, PLAYER_ID)
        .unwrap_or_else(|| panic!("player has no level metadata"));

    if meta.solved {
        panic!("level already solved");
    }

    let player_pos = position_of(world, env, PLAYER_ID);
    let dest = (player_pos.x + dx, player_pos.y + dy);

    if !in_bounds(dest.0, dest.1, meta.width, meta.height) {
        panic!("out of bounds");
    }

    if entity_at::<Wall>(world, env, dest.0, dest.1).is_some() {
        panic!("blocked by wall");
    }

    if let Some(crate_id) = entity_at::<Crate>(world, env, dest.0, dest.1) {
        let crate_dest = (dest.0 + dx, dest.1 + dy);

        if !in_bounds(crate_dest.0, crate_dest.1, meta.width, meta.height) {
            panic!("out of bounds");
        }

        if entity_at::<Wall>(world, env, crate_dest.0, crate_dest.1).is_some() {
            panic!("crate blocked by wall");
        }
        if entity_at::<Crate>(world, env, crate_dest.0, crate_dest.1).is_some() {
            panic!("crate blocked by another crate");
        }

        world.set_typed(env, crate_id, &Position::new(crate_dest.0, crate_dest.1));

        if entity_at::<Goal>(world, env, crate_dest.0, crate_dest.1).is_some() {
            world.set_typed(env, crate_id, &OnGoal);
        } else {
            world.remove_typed::<OnGoal>(env, crate_id);
        }
    }

    world.set_typed(env, PLAYER_ID, &Position::new(dest.0, dest.1));

    meta.move_count += 1;
    meta.solved = is_solved(world, env);
    world.set_typed(env, PLAYER_ID, &meta);
}
