#![cfg(test)]

use super::*;
use soroban_sdk::Env;

fn get_state(env: &Env, contract_id: &soroban_sdk::Address) -> GameState {
    env.as_contract(contract_id, || {
        env.storage()
            .instance()
            .get(&state_key())
            .expect("state missing")
    })
}

fn set_state(env: &Env, contract_id: &soroban_sdk::Address, state: &GameState) {
    env.as_contract(contract_id, || {
        env.storage().instance().set(&state_key(), state);
    });
}

#[test]
fn test_smoke_and_init() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let count = client.cougr_smoke();
    assert_eq!(count, 1);

    client.init_game();
    let state = get_state(&env, &contract_id);
    assert_eq!(state.score, 0);
    assert_eq!(state.lives, 3);
    assert_eq!(state.asteroids.len(), 2);
    assert_eq!(state.bullets.len(), 0);
    assert_eq!(state.game_over, false);
}

#[test]
fn test_tick_progression() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.init_game();
    client.rotate_ship(&1);
    client.thrust_ship();
    client.shoot();
    client.update_tick();
}

#[test]
fn test_rotation_wraps() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.init_game();
    client.rotate_ship(&-1);
    let state = get_state(&env, &contract_id);
    assert_eq!(state.ship.rotation, DIRECTIONS - 1);
}

#[test]
fn test_thrust_changes_velocity() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.init_game();
    let before = get_state(&env, &contract_id).ship.velocity;
    client.thrust_ship();
    let after = get_state(&env, &contract_id).ship.velocity;
    assert!(before.x != after.x || before.y != after.y);
}

#[test]
fn test_shoot_adds_bullet() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.init_game();
    client.shoot();
    let state = get_state(&env, &contract_id);
    assert_eq!(state.bullets.len(), 1);
}

#[test]
fn test_asteroid_split_and_score() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.init_game();
    let mut state = get_state(&env, &contract_id);
    let asteroid = Asteroid {
        position: Vec2 { x: 100 * SCALE, y: 100 * SCALE },
        velocity: Vec2 { x: 0, y: 0 },
        size: 2,
    };
    state.asteroids = Vec::new(&env);
    state.asteroids.push_back(asteroid.clone());
    state.bullets = Vec::new(&env);
    state.bullets.push_back(Bullet {
        position: asteroid.position,
        velocity: Vec2 { x: 0, y: 0 },
        ttl: BULLET_TTL,
    });
    set_state(&env, &contract_id, &state);

    client.update_tick();
    let state = get_state(&env, &contract_id);
    assert_eq!(state.score, 10);
    assert_eq!(state.asteroids.len(), 2);
    assert_eq!(state.asteroids.get(0).unwrap().size, 1);
    assert_eq!(state.asteroids.get(1).unwrap().size, 1);
}

#[test]
fn test_collision_reduces_lives() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.init_game();
    let mut state = get_state(&env, &contract_id);
    let ship_pos = state.ship.position;
    state.asteroids = Vec::new(&env);
    state.asteroids.push_back(Asteroid {
        position: ship_pos,
        velocity: Vec2 { x: 0, y: 0 },
        size: 3,
    });
    set_state(&env, &contract_id, &state);

    client.update_tick();
    let state = get_state(&env, &contract_id);
    assert_eq!(state.lives, 2);
}

#[test]
fn test_game_over_when_no_asteroids() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.init_game();
    let mut state = get_state(&env, &contract_id);
    state.asteroids = Vec::new(&env);
    set_state(&env, &contract_id, &state);

    client.update_tick();
    let state = get_state(&env, &contract_id);
    assert!(state.game_over);
}
