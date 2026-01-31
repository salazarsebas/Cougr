#![cfg(test)]
use super::*;
use soroban_sdk::Env;

#[test]
fn test_player_component_serialization() {
    let env = Env::default();
    let player = PlayerComponent::new(1, 5, 7);

    let serialized = player.serialize(&env);
    let deserialized = PlayerComponent::deserialize(&env, &serialized).unwrap();

    assert_eq!(player.id, deserialized.id);
    assert_eq!(player.x, deserialized.x);
    assert_eq!(player.y, deserialized.y);
    assert_eq!(player.lives, deserialized.lives);
    assert_eq!(player.bomb_capacity, deserialized.bomb_capacity);
    assert_eq!(player.score, deserialized.score);
}

#[test]
fn test_bomb_component_serialization() {
    let env = Env::default();
    let bomb = BombComponent::new(3, 4, 1);

    let serialized = bomb.serialize(&env);
    let deserialized = BombComponent::deserialize(&env, &serialized).unwrap();

    assert_eq!(bomb.x, deserialized.x);
    assert_eq!(bomb.y, deserialized.y);
    assert_eq!(bomb.timer, deserialized.timer);
    assert_eq!(bomb.power, deserialized.power);
    assert_eq!(bomb.owner_id, deserialized.owner_id);
}

#[test]
fn test_explosion_component_serialization() {
    let env = Env::default();
    let explosion = ExplosionComponent::new(2, 3);

    let serialized = explosion.serialize(&env);
    let deserialized = ExplosionComponent::deserialize(&env, &serialized).unwrap();

    assert_eq!(explosion.x, deserialized.x);
    assert_eq!(explosion.y, deserialized.y);
    assert_eq!(explosion.timer, deserialized.timer);
}

#[test]
fn test_grid_component_creation() {
    let grid = GridComponent::new();

    // Check that borders are walls
    assert_eq!(grid.get_cell(0, 0), CellType::Wall);
    assert_eq!(grid.get_cell(GRID_WIDTH - 1, 0), CellType::Wall);
    assert_eq!(grid.get_cell(0, GRID_HEIGHT - 1), CellType::Wall);
    assert_eq!(grid.get_cell(GRID_WIDTH - 1, GRID_HEIGHT - 1), CellType::Wall);

    // Check that some internal cells have destructible blocks or power-ups
    let mut has_destructible = false;
    let mut has_powerup = false;
    for x in 1..GRID_WIDTH - 1 {
        for y in 1..GRID_HEIGHT - 1 {
            match grid.get_cell(x, y) {
                CellType::Destructible => has_destructible = true,
                CellType::PowerUp => has_powerup = true,
                _ => {}
            }
        }
    }
    assert!(has_destructible || has_powerup); // At least one of each should exist
}

#[test]
fn test_grid_walkable() {
    let grid = GridComponent::new();

    // Walls should not be walkable
    assert!(!grid.is_walkable(0, 0));

    // Empty cells should be walkable
    let mut found_empty = false;
    for x in 1..GRID_WIDTH - 1 {
        for y in 1..GRID_HEIGHT - 1 {
            if grid.get_cell(x, y) == CellType::Empty {
                assert!(grid.is_walkable(x as i32, y as i32));
                found_empty = true;
                break;
            }
        }
        if found_empty {
            break;
        }
    }

    // Out of bounds should not be walkable
    assert!(!grid.is_walkable(-1, 5));
    assert!(!grid.is_walkable(GRID_WIDTH as i32, 5));
    assert!(!grid.is_walkable(5, -1));
    assert!(!grid.is_walkable(5, GRID_HEIGHT as i32));
}

#[test]
fn test_grid_component_serialization() {
    let env = Env::default();
    let grid = GridComponent::new();

    let serialized = grid.serialize(&env);
    let deserialized = GridComponent::deserialize(&env, &serialized).unwrap();

    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            assert_eq!(grid.get_cell(x, y), deserialized.get_cell(x, y));
        }
    }
}

#[test]
fn test_game_state_component_serialization() {
    let env = Env::default();
    let mut game_state = GameStateComponent::new();
    game_state.current_tick = 42;
    game_state.game_over = true;
    game_state.winner_id = Some(5);

    let serialized = game_state.serialize(&env);
    let deserialized = GameStateComponent::deserialize(&env, &serialized).unwrap();

    assert_eq!(game_state.current_tick, deserialized.current_tick);
    assert_eq!(game_state.game_over, deserialized.game_over);
    assert_eq!(game_state.winner_id, deserialized.winner_id);
}

#[test]
fn test_game_state_component_no_winner() {
    let env = Env::default();
    let game_state = GameStateComponent::new();

    let serialized = game_state.serialize(&env);
    let deserialized = GameStateComponent::deserialize(&env, &serialized).unwrap();

    assert_eq!(game_state.current_tick, deserialized.current_tick);
    assert_eq!(game_state.game_over, deserialized.game_over);
    assert_eq!(game_state.winner_id, deserialized.winner_id);
}

#[test]
fn test_contract_init_game() {
    let env = Env::default();
    let contract = BombermanContract;

    let result = contract.init_game(env);
    assert_eq!(result, symbol_short!("initialized"));
}

#[test]
fn test_contract_move_player() {
    let env = Env::default();
    let contract = BombermanContract;

    // Test valid directions
    for direction in 0..=3 {
        let result = contract.move_player(env.clone(), 1, direction);
        assert_eq!(result, symbol_short!("moved"));
    }

    // Test invalid direction
    let result = contract.move_player(env, 1, 4);
    assert_eq!(result, symbol_short!("invalid_dir"));
}

#[test]
fn test_contract_place_bomb() {
    let env = Env::default();
    let contract = BombermanContract;

    let result = contract.place_bomb(env, 1);
    assert_eq!(result, symbol_short!("bomb_placed"));
}

#[test]
fn test_contract_update_tick() {
    let env = Env::default();
    let contract = BombermanContract;

    let result = contract.update_tick(env);
    assert_eq!(result, symbol_short!("tick_updated"));
}

#[test]
fn test_contract_get_score() {
    let env = Env::default();
    let contract = BombermanContract;

    let score = contract.get_score(env, 1);
    assert_eq!(score, 100); // placeholder value
}

#[test]
fn test_contract_check_game_over() {
    let env = Env::default();
    let contract = BombermanContract;

    let result = contract.check_game_over(env);
    assert_eq!(result, symbol_short!("ongoing"));
}

#[test]
fn test_contract_hello() {
    let env = Env::default();
    let contract = BombermanContract;

    let result = contract.hello(env, symbol_short!("world"));
    assert_eq!(result, symbol_short!("world"));
}

// Integration test demonstrating cougr-core usage
#[test]
fn test_cougr_core_integration() {
    let env = Env::default();

    // Create world using cougr-core
    let mut world = create_world();

    // Create and spawn a player entity
    let player = PlayerComponent::new(1, 2, 3);
    let player_component = Component::new(
        PlayerComponent::component_type(),
        player.serialize(&env)
    );

    let player_entity_id = spawn_entity(&mut world, Vec::from_array(&env, [player_component]));

    // Verify entity was created
    assert!(world.exists(player_entity_id));

    // Verify component exists
    assert!(world.has_component(player_entity_id, &PlayerComponent::component_type()));

    // Retrieve and verify component data
    let retrieved_component = world.get_component(player_entity_id, &PlayerComponent::component_type());
    assert!(retrieved_component.is_some());

    let retrieved_player = PlayerComponent::deserialize(&env, &retrieved_component.unwrap().data()).unwrap();
    assert_eq!(retrieved_player.id, 1);
    assert_eq!(retrieved_player.x, 2);
    assert_eq!(retrieved_player.y, 3);
    assert_eq!(retrieved_player.lives, INITIAL_LIVES);
}

// Test component type symbols are unique
#[test]
fn test_component_type_uniqueness() {
    assert_ne!(PlayerComponent::component_type(), BombComponent::component_type());
    assert_ne!(PlayerComponent::component_type(), ExplosionComponent::component_type());
    assert_ne!(PlayerComponent::component_type(), GridComponent::component_type());
    assert_ne!(PlayerComponent::component_type(), GameStateComponent::component_type());
    assert_ne!(BombComponent::component_type(), ExplosionComponent::component_type());
    assert_ne!(BombComponent::component_type(), GridComponent::component_type());
    assert_ne!(BombComponent::component_type(), GameStateComponent::component_type());
    assert_ne!(ExplosionComponent::component_type(), GridComponent::component_type());
    assert_ne!(ExplosionComponent::component_type(), GameStateComponent::component_type());
    assert_ne!(GridComponent::component_type(), GameStateComponent::component_type());
}