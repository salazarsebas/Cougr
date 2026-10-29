#![cfg(test)]

use super::*;
use cougr_core::component::ComponentTrait;
use soroban_sdk::Env;

fn get_world(env: &Env, contract_id: &soroban_sdk::Address) -> ECSWorldState {
    env.as_contract(contract_id, || {
        env.storage()
            .instance()
            .get(&state_key())
            .expect("world missing")
    })
}

fn set_world(env: &Env, contract_id: &soroban_sdk::Address, world: &ECSWorldState) {
    env.as_contract(contract_id, || {
        env.storage().instance().set(&state_key(), world);
    });
}

#[test]
fn test_init() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.init_game();
    let world = get_world(&env, &contract_id);
    assert_eq!(world.score.points, 0);
    assert_eq!(world.score.lives, 3);
    assert_eq!(world.asteroids.len(), 2);
    assert_eq!(world.bullets.len(), 0);
    assert!(!world.game_over);
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
    let world = get_world(&env, &contract_id);
    assert_eq!(world.ship.angle, DIRECTIONS - 1);
}

#[test]
fn test_thrust_changes_velocity() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.init_game();
    let before = get_world(&env, &contract_id).ship;
    client.thrust_ship();
    let after = get_world(&env, &contract_id).ship;
    assert!(before.vx != after.vx || before.vy != after.vy);
}

#[test]
fn test_shoot_adds_bullet() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.init_game();
    client.shoot();
    let world = get_world(&env, &contract_id);
    assert_eq!(world.bullets.len(), 1);
}

#[test]
fn test_asteroid_split_and_score() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.init_game();
    let mut world = get_world(&env, &contract_id);
    let asteroid = AsteroidComponent {
        x: 100 * SCALE,
        y: 100 * SCALE,
        vx: 0,
        vy: 0,
        size: 2,
    };
    world.asteroids = Vec::new(&env);
    world.asteroids.push_back(asteroid.clone());
    world.bullets = Vec::new(&env);
    world.bullets.push_back(BulletComponent {
        x: asteroid.x,
        y: asteroid.y,
        vx: 0,
        vy: 0,
        lifetime: BULLET_TTL,
    });
    set_world(&env, &contract_id, &world);

    client.update_tick();
    let world = get_world(&env, &contract_id);
    assert_eq!(world.score.points, 10);
    assert_eq!(world.asteroids.len(), 2);
    assert_eq!(world.asteroids.get(0).unwrap().size, 1);
    assert_eq!(world.asteroids.get(1).unwrap().size, 1);
}

#[test]
fn test_collision_reduces_lives() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.init_game();
    let mut world = get_world(&env, &contract_id);
    let ship_x = world.ship.x;
    let ship_y = world.ship.y;
    world.asteroids = Vec::new(&env);
    world.asteroids.push_back(AsteroidComponent {
        x: ship_x,
        y: ship_y,
        vx: 0,
        vy: 0,
        size: 3,
    });
    set_world(&env, &contract_id, &world);

    client.update_tick();
    let world = get_world(&env, &contract_id);
    assert_eq!(world.score.lives, 2);
}

#[test]
fn test_game_over_when_no_asteroids() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.init_game();
    let mut world = get_world(&env, &contract_id);
    world.asteroids = Vec::new(&env);
    set_world(&env, &contract_id, &world);

    client.update_tick();
    let world = get_world(&env, &contract_id);
    assert!(world.game_over);
}

#[test]
fn test_bullet_lifetime_cleanup() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.init_game();
    let mut world = get_world(&env, &contract_id);
    world.bullets = Vec::new(&env);
    world.bullets.push_back(BulletComponent {
        x: 100 * SCALE,
        y: 100 * SCALE,
        vx: 0,
        vy: 0,
        lifetime: 1,
    });
    set_world(&env, &contract_id, &world);

    client.update_tick();
    let world = get_world(&env, &contract_id);
    assert_eq!(world.bullets.len(), 0);
}

#[test]
fn test_component_serialization() {
    let env = Env::default();

    let ship = ShipComponent {
        x: 100,
        y: 200,
        vx: 10,
        vy: 20,
        angle: 3,
    };
    let data = ship.serialize(&env);
    let deserialized = ShipComponent::deserialize(&env, &data).unwrap();
    assert_eq!(ship.x, deserialized.x);
    assert_eq!(ship.y, deserialized.y);
    assert_eq!(ship.vx, deserialized.vx);
    assert_eq!(ship.vy, deserialized.vy);
    assert_eq!(ship.angle, deserialized.angle);
}

#[test]
fn test_get_game_state() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.init_game();
    let state = client.get_game_state();
    assert_eq!(state.score, 0);
    assert_eq!(state.lives, 3);
    assert_eq!(state.asteroid_count, 2);
    assert_eq!(state.bullet_count, 0);
    assert!(!state.game_over);
}
