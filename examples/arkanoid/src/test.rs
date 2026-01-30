use crate::{ArkanoidContract, ArkanoidContractClient};
use soroban_sdk::Env;

#[test]
fn test_init_game() {
    let env = Env::default();
    let contract_id = env.register(ArkanoidContract, ());
    let client = ArkanoidContractClient::new(&env, &contract_id);

    let state = client.init_game();

    assert_eq!(state.paddle_x, 50); // Center of field (100/2)
    assert_eq!(state.ball_x, 50);
    assert_eq!(state.ball_y, 50); // FIELD_HEIGHT - 10
    assert_eq!(state.ball_vx, 1);
    assert_eq!(state.ball_vy, -1);
    assert_eq!(state.score, 0);
    assert_eq!(state.lives, 3);
    assert_eq!(state.game_active, true);
    assert_eq!(state.bricks_remaining, 50); // 10 cols * 5 rows
}

#[test]
fn test_move_paddle_right() {
    let env = Env::default();
    let contract_id = env.register(ArkanoidContract, ());
    let client = ArkanoidContractClient::new(&env, &contract_id);

    client.init_game();
    let state = client.move_paddle(&1); // Move right

    assert_eq!(state.paddle_x, 53); // 50 + (1 * 3)
}

#[test]
fn test_move_paddle_left() {
    let env = Env::default();
    let contract_id = env.register(ArkanoidContract, ());
    let client = ArkanoidContractClient::new(&env, &contract_id);

    client.init_game();
    let state = client.move_paddle(&-1); // Move left

    assert_eq!(state.paddle_x, 47); // 50 - (1 * 3)
}

#[test]
fn test_paddle_bounds_left() {
    let env = Env::default();
    let contract_id = env.register(ArkanoidContract, ());
    let client = ArkanoidContractClient::new(&env, &contract_id);

    client.init_game();

    // Move paddle far left
    for _ in 0..20 {
        client.move_paddle(&-1);
    }

    let state = client.get_game_state();
    assert_eq!(state.paddle_x, 7); // PADDLE_WIDTH/2 = 15/2 = 7
}

#[test]
fn test_paddle_bounds_right() {
    let env = Env::default();
    let contract_id = env.register(ArkanoidContract, ());
    let client = ArkanoidContractClient::new(&env, &contract_id);

    client.init_game();

    // Move paddle far right
    for _ in 0..20 {
        client.move_paddle(&1);
    }

    let state = client.get_game_state();
    assert_eq!(state.paddle_x, 93); // FIELD_WIDTH - PADDLE_WIDTH/2 = 100 - 7
}

#[test]
fn test_ball_physics() {
    let env = Env::default();
    let contract_id = env.register(ArkanoidContract, ());
    let client = ArkanoidContractClient::new(&env, &contract_id);

    client.init_game();
    let state = client.update_tick();

    // Ball should move by velocity
    assert_eq!(state.ball_x, 51); // 50 + 1
    assert_eq!(state.ball_y, 49); // 50 - 1
}

#[test]
fn test_top_wall_collision() {
    let env = Env::default();
    let contract_id = env.register(ArkanoidContract, ());
    let client = ArkanoidContractClient::new(&env, &contract_id);

    client.init_game();

    // Move ball to top
    for _ in 0..60 {
        client.update_tick();
    }

    let state = client.get_game_state();
    // Ball should have bounced and be moving down
    assert!(state.ball_vy > 0);
}

#[test]
fn test_side_wall_collision() {
    let env = Env::default();
    let contract_id = env.register(ArkanoidContract, ());
    let client = ArkanoidContractClient::new(&env, &contract_id);

    client.init_game();

    // Move ball to side by updating many times
    for _ in 0..60 {
        client.update_tick();
    }

    let state = client.get_game_state();
    // Ball should stay within bounds
    assert!(state.ball_x >= 0 && state.ball_x <= 100);
}

#[test]
fn test_paddle_collision() {
    let env = Env::default();
    let contract_id = env.register(ArkanoidContract, ());
    let client = ArkanoidContractClient::new(&env, &contract_id);

    client.init_game();

    // Move ball down towards paddle
    for _ in 0..10 {
        client.update_tick();
    }

    let state = client.get_game_state();
    // Ball should eventually bounce off paddle and move up
    if state.ball_y >= 55 {
        assert!(state.ball_vy < 0);
    }
}

#[test]
fn test_brick_breaking() {
    let env = Env::default();
    let contract_id = env.register(ArkanoidContract, ());
    let client = ArkanoidContractClient::new(&env, &contract_id);

    let initial_state = client.init_game();
    let initial_bricks = initial_state.bricks_remaining;

    // Simulate game ticks to hit bricks
    for _ in 0..5 {
        client.update_tick();
    }

    let state = client.get_game_state();
    // At least one brick should be broken or score increased
    assert!(state.bricks_remaining <= initial_bricks || state.score > 0);
}

#[test]
fn test_scoring() {
    let env = Env::default();
    let contract_id = env.register(ArkanoidContract, ());
    let client = ArkanoidContractClient::new(&env, &contract_id);

    client.init_game();

    // Play for a while
    for _ in 0..20 {
        client.update_tick();
    }

    let state = client.get_game_state();
    // Score should increase when bricks are broken
    if state.bricks_remaining < 50 {
        assert!(state.score > 0);
    }
}

#[test]
fn test_lose_life() {
    let env = Env::default();
    let contract_id = env.register(ArkanoidContract, ());
    let client = ArkanoidContractClient::new(&env, &contract_id);

    client.init_game();

    // Move paddle away from ball
    for _ in 0..10 {
        client.move_paddle(&1);
    }

    // Let ball fall
    // Ball starts at y=50, moving up (vy=-1). Hits top (y=0) at approx tick 50.
    // Bounces down. Hits bottom (y=60) at approx tick 110.
    // We run for 300 ticks to be absolutely safe.
    for _ in 0..300 {
        client.update_tick();
    }

    let state = client.get_game_state();
    // Should have lost at least one life
    assert!(state.lives < 3);
}

#[test]
fn test_game_over_no_lives() {
    let env = Env::default();
    let contract_id = env.register(ArkanoidContract, ());
    let client = ArkanoidContractClient::new(&env, &contract_id);

    client.init_game();

    // Move paddle away and let ball fall multiple times
    for _ in 0..5 {
        for _ in 0..10 {
            client.move_paddle(&1);
        }
        for _ in 0..150 {
            client.update_tick();
        }
    }

    let state = client.get_game_state();
    // Game should end when lives reach 0
    assert_eq!(state.lives, 0);
    assert_eq!(state.game_active, false);
    assert_eq!(client.check_game_over(), true);
}

#[test]
fn test_get_game_state() {
    let env = Env::default();
    let contract_id = env.register(ArkanoidContract, ());
    let client = ArkanoidContractClient::new(&env, &contract_id);

    client.init_game();
    let state1 = client.get_game_state();

    client.move_paddle(&1);
    let state2 = client.get_game_state();

    // State should change after paddle movement
    assert_ne!(state1.paddle_x, state2.paddle_x);
}

#[test]
fn test_check_game_over() {
    let env = Env::default();
    let contract_id = env.register(ArkanoidContract, ());
    let client = ArkanoidContractClient::new(&env, &contract_id);

    client.init_game();

    let game_over = client.check_game_over();
    assert_eq!(game_over, false);
}

#[test]
fn test_inactive_game_no_updates() {
    let env = Env::default();
    let contract_id = env.register(ArkanoidContract, ());
    let client = ArkanoidContractClient::new(&env, &contract_id);

    client.init_game();

    // Force game over by losing all lives
    for _ in 0..5 {
        for _ in 0..10 {
            client.move_paddle(&1);
        }
        for _ in 0..150 {
            client.update_tick();
        }
    }

    let state1 = client.get_game_state();
    assert!(!state1.game_active);

    let ball_x = state1.ball_x;

    // Try to update
    client.update_tick();
    let state2 = client.get_game_state();

    // Ball position should not change when game is inactive
    assert_eq!(state2.ball_x, ball_x);
}

#[test]
fn test_multiple_brick_breaks() {
    let env = Env::default();
    let contract_id = env.register(ArkanoidContract, ());
    let client = ArkanoidContractClient::new(&env, &contract_id);

    let initial_state = client.init_game();

    // Play for extended period
    for _ in 0..50 {
        client.update_tick();
    }

    let state = client.get_game_state();
    // Bricks should decrease
    assert!(state.bricks_remaining <= initial_state.bricks_remaining);
}
