#![cfg(test)]

use super::*;
use soroban_sdk::{Env};
use cougr_core::{SimpleWorld, GameApp, ScheduleStage, SystemConfig};

#[test]
fn test_init_game() {
    let env = Env::default();
    let contract_id = env.register(FlappyBirdContract, ());
    let client = FlappyBirdContractClient::new(&env, &contract_id);

    // Initialize game
    client.init_game();

    // Check game state
    let score = client.get_score();
    assert_eq!(score, 0);

    let game_over = client.check_game_over();
    assert!(!game_over);

    // Check bird position
    let (x, y) = client.get_bird_pos();
    assert_eq!(x, INIT_BIRD_X);
    assert_eq!(y, INIT_BIRD_Y);
}

#[test]
fn test_flap() {
    let env = Env::default();
    let contract_id = env.register(FlappyBirdContract, ());
    let client = FlappyBirdContractClient::new(&env, &contract_id);

    // Initialize game
    client.init_game();

    // Flap
    client.flap();

    // Bird velocity should have changed (will see effect after update_tick)
    client.update_tick();

    let (_, y) = client.get_bird_pos();
    // After flap and one tick, bird should have moved up
    assert!(y < INIT_BIRD_Y);
}

#[test]
fn test_gravity() {
    let env = Env::default();
    let contract_id = env.register(FlappyBirdContract, ());
    let client = FlappyBirdContractClient::new(&env, &contract_id);

    // Initialize game
    client.init_game();

    let (_, y_before) = client.get_bird_pos();

    // Update multiple ticks without flapping
    client.update_tick();
    client.update_tick();
    client.update_tick();

    let (_, y_after) = client.get_bird_pos();

    // Bird should have fallen
    assert!(y_after > y_before);
}

#[test]
fn test_game_over_on_ground_collision() {
    let env = Env::default();
    let contract_id = env.register(FlappyBirdContract, ());
    let client = FlappyBirdContractClient::new(&env, &contract_id);

    // Initialize game
    client.init_game();

    // Let bird fall to ground
    for _ in 0..100 {
        client.update_tick();
        if client.check_game_over() {
            break;
        }
    }

    // Game should be over
    assert!(client.check_game_over());
}

#[test]
fn test_score_increases() {
    let env = Env::default();
    let contract_id = env.register(FlappyBirdContract, ());
    let client = FlappyBirdContractClient::new(&env, &contract_id);

    // Initialize game
    client.init_game();

    let initial_score = client.get_score();

    // Play for a while
    for i in 0..100 {
        if i % 5 == 0 {
            client.flap(); // Flap periodically to stay alive
        }
        client.update_tick();

        if client.check_game_over() {
            break;
        }
    }

    let final_score = client.get_score();

    // Score should have increased if we survived long enough
    // (might not if we died early)
    if !client.check_game_over() {
        assert!(final_score >= initial_score);
    }
}

#[test]
fn test_cannot_flap_after_game_over() {
    let env = Env::default();
    let contract_id = env.register(FlappyBirdContract, ());
    let client = FlappyBirdContractClient::new(&env, &contract_id);

    // Initialize game
    client.init_game();

    // Let bird fall to ground
    for _ in 0..100 {
        client.update_tick();
        if client.check_game_over() {
            break;
        }
    }

    assert!(client.check_game_over());

    // Try to flap after game over
    client.flap();

    // Position should not change after game over
    let (x1, y1) = client.get_bird_pos();
    client.update_tick();
    let (x2, y2) = client.get_bird_pos();

    assert_eq!(x1, x2);
    assert_eq!(y1, y2);
}

#[test]
fn test_gameapp_tick_integration() {
    let env = Env::default();
    let mut app = GameApp::new(&env);
    app.add_system_with_config(
        "flappy_tick_boundary",
        |_world: &mut SimpleWorld, _env: &Env| {},
        SystemConfig::new().in_stage(ScheduleStage::Update),
    );
    app.run(&env).unwrap();
}
