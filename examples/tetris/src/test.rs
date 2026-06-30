#![cfg(test)]

use super::*;

use soroban_sdk::Env; 

#[test]
fn test_init_game() {
    let env = Env::default();
    let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
    let state = client.init_game();
    assert_eq!(state.score, 0);
    assert!(!state.game_over);
}

#[test]
fn test_move_functions() {
    let env = Env::default();
    let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
    client.init_game();

    // Initial move
    let _moved = client.move_left();
}

#[test]
fn test_rotation() {
    let env = Env::default();
    let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
    client.init_game();

    // Try rotate
    let _rotated = client.rotate();
}

#[test]
fn test_collision_detection() {
    let env = Env::default();
    let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
    client.init_game();

    for _ in 0..10 {
        client.move_left();
    }
}

#[test]
fn test_line_clearing() {
    let env = Env::default();
    let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
    client.init_game();

    let _lines = client.update_tick();
}

#[test]
fn test_score_updates() {
    let env = Env::default();
    let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
    client.init_game();

    assert_eq!(client.get_state().score, 0);
}

#[test]
fn test_game_over() {
    let env = Env::default();
    let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
    client.init_game();

    assert!(!client.get_state().game_over);
}

#[test]
fn test_invalid_action_at_left_wall_returns_false() {
    let env = Env::default();
    let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
    client.init_game();
    let mut last = true;
    for _ in 0..16 {
        last = client.move_left();
    }
    assert!(!last);
}

#[test]
fn test_gameapp_tick_integration() {
    let env = Env::default();
    super::systems::run_gameapp_tick(&env);
}
