use crate::components::{Direction, DirectionComponent, Food, Position, SnakeSegment};
use crate::simple_world::{EntityId, SimpleWorld};
use cougr_core::component::ComponentTrait;
use soroban_sdk::{symbol_short, Env, Vec};

/// Segment data for sorting (entity_id, index, x, y)
type SegmentData = (EntityId, u32, i32, i32);

/// Move the snake in the current direction
/// Returns the new head position
pub fn move_snake(world: &mut SimpleWorld, env: &Env, grid_size: i32) -> Option<Position> {
    // Find snake head
    let head_entities = world.get_entities_with_component(&symbol_short!("snkhead"), env);
    if head_entities.is_empty() {
        return None;
    }
    let head_id = head_entities.get(0).unwrap();

    // Get current head position and direction
    let head_pos = get_position(world, head_id, env)?;
    let direction = get_direction(world, head_id, env)?;

    // Calculate new head position
    let (dx, dy) = direction.delta();
    let new_x = head_pos.x + dx;
    let new_y = head_pos.y + dy;

    // Check wall collision (out of bounds)
    if new_x < 0 || new_x >= grid_size || new_y < 0 || new_y >= grid_size {
        return None; // Hit wall
    }

    let new_head_pos = Position::new(new_x, new_y);

    // Move body segments (from tail to head)
    move_body_segments(world, env, head_pos.x, head_pos.y);

    // Update head position
    world.add_component(
        head_id,
        symbol_short!("position"),
        new_head_pos.serialize(env),
    );

    Some(new_head_pos)
}

/// Move all body segments to follow the head
fn move_body_segments(world: &mut SimpleWorld, env: &Env, old_head_x: i32, old_head_y: i32) {
    let segment_entities = world.get_entities_with_component(&symbol_short!("snkseg"), env);

    // Collect segments with their indices as tuples of primitives
    let mut segments: Vec<SegmentData> = Vec::new(env);

    for i in 0..segment_entities.len() {
        let entity_id = segment_entities.get(i).unwrap();
        if let (Some(seg), Some(pos)) = (
            get_segment(world, entity_id, env),
            get_position(world, entity_id, env),
        ) {
            segments.push_back((entity_id, seg.index, pos.x, pos.y));
        }
    }

    // Sort by index descending (move tail first)
    // Simple bubble sort since we can't use standard sort in no_std
    let len = segments.len();
    for i in 0..len {
        for j in 0..(len - 1 - i) {
            let (_, idx_j, _, _) = segments.get(j).unwrap();
            let (_, idx_next, _, _) = segments.get(j + 1).unwrap();
            if idx_j < idx_next {
                // Swap
                let temp = segments.get(j).unwrap();
                let next = segments.get(j + 1).unwrap();
                segments.set(j, next);
                segments.set(j + 1, temp);
            }
        }
    }

    // Move each segment to the position of the segment ahead of it
    for i in 0..segments.len() {
        let (entity_id, index, _, _) = segments.get(i).unwrap();

        // Find the position this segment should move to
        let (target_x, target_y) = if index == 0 {
            // First segment follows the head
            (old_head_x, old_head_y)
        } else {
            // Find the segment with index - 1
            let mut found_pos = None;
            for j in 0..segments.len() {
                let (_, other_idx, other_x, other_y) = segments.get(j).unwrap();
                if other_idx == index - 1 {
                    found_pos = Some((other_x, other_y));
                    break;
                }
            }
            match found_pos {
                Some(pos) => pos,
                None => continue,
            }
        };

        let target_pos = Position::new(target_x, target_y);
        world.add_component(
            entity_id,
            symbol_short!("position"),
            target_pos.serialize(env),
        );
    }
}

/// Check if the snake collides with itself
pub fn check_self_collision(world: &SimpleWorld, env: &Env) -> bool {
    // Get head position
    let head_entities = world.get_entities_with_component(&symbol_short!("snkhead"), env);
    if head_entities.is_empty() {
        return false;
    }
    let head_id = head_entities.get(0).unwrap();
    let head_pos = match get_position(world, head_id, env) {
        Some(pos) => pos,
        None => return false,
    };

    // Check against all body segments
    let segment_entities = world.get_entities_with_component(&symbol_short!("snkseg"), env);
    for i in 0..segment_entities.len() {
        let entity_id = segment_entities.get(i).unwrap();
        if let Some(seg_pos) = get_position(world, entity_id, env) {
            if seg_pos.x == head_pos.x && seg_pos.y == head_pos.y {
                return true;
            }
        }
    }

    false
}

/// Check if the snake head is on food
pub fn check_food_collision(world: &SimpleWorld, env: &Env) -> Option<EntityId> {
    // Get head position
    let head_entities = world.get_entities_with_component(&symbol_short!("snkhead"), env);
    if head_entities.is_empty() {
        return None;
    }
    let head_id = head_entities.get(0).unwrap();
    let head_pos = get_position(world, head_id, env)?;

    // Check against food entities
    let food_entities = world.get_entities_with_component(&symbol_short!("food"), env);
    for i in 0..food_entities.len() {
        let food_id = food_entities.get(i).unwrap();
        if let Some(food_pos) = get_position(world, food_id, env) {
            if food_pos.x == head_pos.x && food_pos.y == head_pos.y {
                return Some(food_id);
            }
        }
    }

    None
}

/// Grow the snake by adding a new segment at the tail
pub fn grow_snake(world: &mut SimpleWorld, env: &Env) {
    let segment_entities = world.get_entities_with_component(&symbol_short!("snkseg"), env);

    // Find the highest segment index and tail position
    let mut max_index: u32 = 0;
    let mut tail_pos = None;

    // If no segments exist, find head position
    if segment_entities.is_empty() {
        let head_entities = world.get_entities_with_component(&symbol_short!("snkhead"), env);
        if !head_entities.is_empty() {
            let head_id = head_entities.get(0).unwrap();
            tail_pos = get_position(world, head_id, env);
        }
    } else {
        for i in 0..segment_entities.len() {
            let entity_id = segment_entities.get(i).unwrap();
            if let Some(seg) = get_segment(world, entity_id, env) {
                if seg.index >= max_index {
                    max_index = seg.index;
                    tail_pos = get_position(world, entity_id, env);
                }
            }
        }
        max_index += 1;
    }

    // Create new segment at tail position (will be pushed back on next move)
    if let Some(pos) = tail_pos {
        let new_segment_id = world.spawn_entity();
        let segment = SnakeSegment::new(max_index);
        world.add_component(
            new_segment_id,
            symbol_short!("position"),
            pos.serialize(env),
        );
        world.add_component(
            new_segment_id,
            symbol_short!("snkseg"),
            segment.serialize(env),
        );
    }
}

/// Spawn food at a position that doesn't overlap with the snake
pub fn spawn_food(world: &mut SimpleWorld, env: &Env, tick: u32, grid_size: i32) {
    // Remove any existing food
    let food_entities = world.get_entities_with_component(&symbol_short!("food"), env);
    for i in 0..food_entities.len() {
        let food_id = food_entities.get(i).unwrap();
        world.despawn_entity(food_id);
    }

    // Collect all occupied positions
    let mut occupied_count = 0;
    let head_entities = world.get_entities_with_component(&symbol_short!("snkhead"), env);
    let segment_entities = world.get_entities_with_component(&symbol_short!("snkseg"), env);

    occupied_count += head_entities.len() + segment_entities.len();

    // Generate pseudo-random position based on tick
    // This is deterministic for testing and on-chain reproducibility
    let mut attempts = 0;
    let max_attempts = (grid_size * grid_size) as u32;

    while attempts < max_attempts {
        let hash = ((tick.wrapping_mul(17)) ^ (attempts.wrapping_mul(31))) as i32;
        let x = (hash.abs() % grid_size).abs();
        let y = ((hash.wrapping_mul(13)).abs() % grid_size).abs();

        // Check if position is occupied
        let mut is_occupied = false;

        // Check head
        for i in 0..head_entities.len() {
            let entity_id = head_entities.get(i).unwrap();
            if let Some(pos) = get_position(world, entity_id, env) {
                if pos.x == x && pos.y == y {
                    is_occupied = true;
                    break;
                }
            }
        }

        // Check segments
        if !is_occupied {
            for i in 0..segment_entities.len() {
                let entity_id = segment_entities.get(i).unwrap();
                if let Some(pos) = get_position(world, entity_id, env) {
                    if pos.x == x && pos.y == y {
                        is_occupied = true;
                        break;
                    }
                }
            }
        }

        if !is_occupied {
            // Spawn food at this position
            let food_id = world.spawn_entity();
            let food_pos = Position::new(x, y);
            let food = Food;

            world.add_component(food_id, symbol_short!("position"), food_pos.serialize(env));
            world.add_component(food_id, symbol_short!("food"), food.serialize(env));
            return;
        }

        attempts += 1;
    }

    // If all positions are occupied (snake filled the grid), no food spawned
    // This means the player essentially won!
    let _ = occupied_count; // Suppress unused warning
}

/// Update the snake's direction (with validation)
pub fn update_direction(world: &mut SimpleWorld, env: &Env, new_direction: Direction) -> bool {
    let head_entities = world.get_entities_with_component(&symbol_short!("snkhead"), env);
    if head_entities.is_empty() {
        return false;
    }
    let head_id = head_entities.get(0).unwrap();

    // Get current direction
    if let Some(current_dir) = get_direction(world, head_id, env) {
        // Can't reverse direction (would cause immediate self-collision)
        if current_dir.is_opposite(&new_direction) {
            return false;
        }
    }

    // Update direction
    let dir_component = DirectionComponent::new(new_direction);
    world.add_component(
        head_id,
        symbol_short!("direction"),
        dir_component.serialize(env),
    );

    true
}

// Helper functions

fn get_position(world: &SimpleWorld, entity_id: EntityId, env: &Env) -> Option<Position> {
    let data = world.get_component(entity_id, &symbol_short!("position"))?;
    Position::deserialize(env, &data)
}

fn get_direction(world: &SimpleWorld, entity_id: EntityId, env: &Env) -> Option<Direction> {
    let data = world.get_component(entity_id, &symbol_short!("direction"))?;
    let dir_component = DirectionComponent::deserialize(env, &data)?;
    Some(dir_component.direction)
}

fn get_segment(world: &SimpleWorld, entity_id: EntityId, env: &Env) -> Option<SnakeSegment> {
    let data = world.get_component(entity_id, &symbol_short!("snkseg"))?;
    SnakeSegment::deserialize(env, &data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::SnakeHead;

    fn setup_snake(env: &Env) -> SimpleWorld {
        let mut world = SimpleWorld::new(env);

        // Spawn head at center
        let head_id = world.spawn_entity();
        let head_pos = Position::new(5, 5);
        let head = SnakeHead;
        let direction = DirectionComponent::new(Direction::Right);

        world.add_component(head_id, symbol_short!("position"), head_pos.serialize(env));
        world.add_component(head_id, symbol_short!("snkhead"), head.serialize(env));
        world.add_component(
            head_id,
            symbol_short!("direction"),
            direction.serialize(env),
        );

        // Add one body segment
        let seg_id = world.spawn_entity();
        let seg_pos = Position::new(4, 5);
        let segment = SnakeSegment::new(0);

        world.add_component(seg_id, symbol_short!("position"), seg_pos.serialize(env));
        world.add_component(seg_id, symbol_short!("snkseg"), segment.serialize(env));

        world
    }

    #[test]
    fn test_move_snake() {
        let env = Env::default();
        let mut world = setup_snake(&env);

        let new_pos = move_snake(&mut world, &env, 10);
        assert!(new_pos.is_some());
        assert_eq!(new_pos.unwrap(), Position::new(6, 5));
    }

    #[test]
    fn test_wall_collision() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        // Place snake at edge
        let head_id = world.spawn_entity();
        let head_pos = Position::new(9, 5);
        let head = SnakeHead;
        let direction = DirectionComponent::new(Direction::Right);

        world.add_component(head_id, symbol_short!("position"), head_pos.serialize(&env));
        world.add_component(head_id, symbol_short!("snkhead"), head.serialize(&env));
        world.add_component(
            head_id,
            symbol_short!("direction"),
            direction.serialize(&env),
        );

        // Move should return None (hit wall)
        let result = move_snake(&mut world, &env, 10);
        assert!(result.is_none());
    }

    #[test]
    fn test_self_collision() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        // Create snake that collides with itself
        let head_id = world.spawn_entity();
        let collision_pos = Position::new(5, 5);
        let head = SnakeHead;

        world.add_component(
            head_id,
            symbol_short!("position"),
            collision_pos.serialize(&env),
        );
        world.add_component(head_id, symbol_short!("snkhead"), head.serialize(&env));

        // Add segment at same position
        let seg_id = world.spawn_entity();
        let segment = SnakeSegment::new(0);
        world.add_component(
            seg_id,
            symbol_short!("position"),
            collision_pos.serialize(&env),
        );
        world.add_component(seg_id, symbol_short!("snkseg"), segment.serialize(&env));

        assert!(check_self_collision(&world, &env));
    }

    #[test]
    fn test_food_collision() {
        let env = Env::default();
        let mut world = setup_snake(&env);

        // Place food at snake head position
        let food_id = world.spawn_entity();
        let food_pos = Position::new(5, 5);
        let food = Food;

        world.add_component(food_id, symbol_short!("position"), food_pos.serialize(&env));
        world.add_component(food_id, symbol_short!("food"), food.serialize(&env));

        let result = check_food_collision(&world, &env);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), food_id);
    }

    #[test]
    fn test_grow_snake() {
        let env = Env::default();
        let mut world = setup_snake(&env);

        let initial_segments = world
            .get_entities_with_component(&symbol_short!("snkseg"), &env)
            .len();

        grow_snake(&mut world, &env);

        let final_segments = world
            .get_entities_with_component(&symbol_short!("snkseg"), &env)
            .len();

        assert_eq!(final_segments, initial_segments + 1);
    }

    #[test]
    fn test_update_direction() {
        let env = Env::default();
        let mut world = setup_snake(&env);

        // Can change to perpendicular direction
        assert!(update_direction(&mut world, &env, Direction::Up));

        // Cannot reverse direction
        let result = update_direction(&mut world, &env, Direction::Down);
        assert!(!result);
    }

    #[test]
    fn test_spawn_food() {
        let env = Env::default();
        let mut world = setup_snake(&env);

        spawn_food(&mut world, &env, 1, 10);

        let food_entities = world.get_entities_with_component(&symbol_short!("food"), &env);
        assert_eq!(food_entities.len(), 1);
    }
}
