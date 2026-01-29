use super::*;
use soroban_sdk::Env;

#[test]
fn test_init_game() {
    let env = Env::default();
    let contract_id = env.register(PongContract, ());
    let client = PongContractClient::new(&env, &contract_id);

    let game_state = client.init_game();

    // Verify initial positions
    assert_eq!(game_state.player1_paddle_y, FIELD_HEIGHT / 2);
    assert_eq!(game_state.player2_paddle_y, FIELD_HEIGHT / 2);
    assert_eq!(game_state.ball_x, FIELD_WIDTH / 2);
    assert_eq!(game_state.ball_y, FIELD_HEIGHT / 2);

    // Verify initial velocities
    assert_eq!(game_state.ball_vx, BALL_SPEED);
    assert_eq!(game_state.ball_vy, BALL_SPEED);

    // Verify initial scores
    assert_eq!(game_state.player1_score, 0);
    assert_eq!(game_state.player2_score, 0);

    // Verify game is active
    assert!(game_state.game_active);
}

#[test]
fn test_move_paddle_player1_up() {
    let env = Env::default();
    let contract_id = env.register(PongContract, ());
    let client = PongContractClient::new(&env, &contract_id);

    client.init_game();
    let initial_state = client.get_game_state();
    let initial_y = initial_state.player1_paddle_y;

    let new_state = client.move_paddle(&1u32, &-1i32);

    assert_eq!(new_state.player1_paddle_y, initial_y - PADDLE_SPEED);
}

#[test]
fn test_move_paddle_player1_down() {
    let env = Env::default();
    let contract_id = env.register(PongContract, ());
    let client = PongContractClient::new(&env, &contract_id);

    client.init_game();
    let initial_state = client.get_game_state();
    let initial_y = initial_state.player1_paddle_y;

    let new_state = client.move_paddle(&1u32, &1i32);

    assert_eq!(new_state.player1_paddle_y, initial_y + PADDLE_SPEED);
}

#[test]
fn test_move_paddle_player2() {
    let env = Env::default();
    let contract_id = env.register(PongContract, ());
    let client = PongContractClient::new(&env, &contract_id);

    client.init_game();
    let initial_state = client.get_game_state();
    let initial_y = initial_state.player2_paddle_y;

    let new_state = client.move_paddle(&2u32, &1i32);

    assert_eq!(new_state.player2_paddle_y, initial_y + PADDLE_SPEED);
}

#[test]
fn test_paddle_boundary_top() {
    let env = Env::default();
    let contract_id = env.register(PongContract, ());
    let client = PongContractClient::new(&env, &contract_id);

    client.init_game();

    // Move paddle up many times to hit boundary
    for _ in 0..50 {
        client.move_paddle(&1u32, &-1i32);
    }

    let state = client.get_game_state();
    let min_y = PADDLE_HEIGHT / 2;
    assert_eq!(state.player1_paddle_y, min_y);
}

#[test]
fn test_paddle_boundary_bottom() {
    let env = Env::default();
    let contract_id = env.register(PongContract, ());
    let client = PongContractClient::new(&env, &contract_id);

    client.init_game();

    // Move paddle down many times to hit boundary
    for _ in 0..50 {
        client.move_paddle(&1u32, &1i32);
    }

    let state = client.get_game_state();
    let max_y = FIELD_HEIGHT - (PADDLE_HEIGHT / 2);
    assert_eq!(state.player1_paddle_y, max_y);
}

#[test]
fn test_ball_movement() {
    let env = Env::default();
    let contract_id = env.register(PongContract, ());
    let client = PongContractClient::new(&env, &contract_id);

    let initial_state = client.init_game();
    let initial_x = initial_state.ball_x;
    let initial_y = initial_state.ball_y;

    let new_state = client.update_tick();

    // Ball should have moved
    assert_eq!(new_state.ball_x, initial_x + BALL_SPEED);
    assert_eq!(new_state.ball_y, initial_y + BALL_SPEED);
}

#[test]
fn test_ball_wall_bounce_top() {
    let env = Env::default();
    let contract_id = env.register(PongContract, ());
    let client = PongContractClient::new(&env, &contract_id);

    client.init_game();

    // Move ball to top wall
    for _ in 0..100 {
        let state = client.update_tick();
        if state.ball_y <= 0 {
            // Ball should bounce (velocity should reverse)
            let next_state = client.update_tick();
            assert!(next_state.ball_vy > 0);
            return;
        }
    }

    panic!("Ball never reached top wall");
}

#[test]
fn test_ball_wall_bounce_bottom() {
    let env = Env::default();
    let contract_id = env.register(PongContract, ());
    let client = PongContractClient::new(&env, &contract_id);

    client.init_game();

    // Move ball to bottom wall by reversing initial velocity
    for _ in 0..100 {
        let state = client.update_tick();
        if state.ball_y >= FIELD_HEIGHT {
            // Ball should bounce (velocity should reverse)
            let next_state = client.update_tick();
            assert!(next_state.ball_vy < 0);
            return;
        }
    }
}

#[test]
fn test_scoring_player2() {
    let env = Env::default();
    let contract_id = env.register(PongContract, ());
    let client = PongContractClient::new(&env, &contract_id);

    client.init_game();

    // Move player 2 paddle out of the way (ball starts moving right)
    for _ in 0..20 {
        client.move_paddle(&2u32, &1i32);
    }

    // Let ball pass player 2's paddle (ball goes off right side, player 2 scores)
    for _ in 0..200 {
        let state = client.update_tick();
        if state.player1_score > 0 {
            assert_eq!(state.player1_score, 1);
            assert_eq!(state.player2_score, 0);
            // Ball should be reset to center
            assert_eq!(state.ball_x, FIELD_WIDTH / 2);
            assert_eq!(state.ball_y, FIELD_HEIGHT / 2);
            return;
        }
    }

    panic!("Player 1 never scored");
}

#[test]
fn test_scoring_player1() {
    let env = Env::default();
    let contract_id = env.register(PongContract, ());
    let client = PongContractClient::new(&env, &contract_id);

    client.init_game();

    // Move player 2 paddle out of the way
    for _ in 0..20 {
        client.move_paddle(&2u32, &1i32);
    }

    // Let ball pass player 2's paddle
    for _ in 0..200 {
        let state = client.update_tick();
        if state.player1_score > 0 {
            assert_eq!(state.player1_score, 1);
            assert_eq!(state.player2_score, 0);
            return;
        }
    }

    panic!("Player 1 never scored");
}

#[test]
fn test_game_ends_at_winning_score() {
    let env = Env::default();
    let contract_id = env.register(PongContract, ());
    let client = PongContractClient::new(&env, &contract_id);

    client.init_game();

    // Move player 2 paddle out of the way
    for _ in 0..20 {
        client.move_paddle(&2u32, &1i32);
    }

    // Play until someone wins
    for _ in 0..2000 {
        let state = client.update_tick();

        if state.player1_score >= WINNING_SCORE {
            assert!(!state.game_active);
            assert_eq!(state.player1_score, WINNING_SCORE);
            return;
        }
    }

    panic!("Game never ended");
}

#[test]
fn test_paddle_collision() {
    let env = Env::default();
    let contract_id = env.register(PongContract, ());
    let client = PongContractClient::new(&env, &contract_id);

    client.init_game();

    // Keep paddles centered
    let mut previous_vx = BALL_SPEED;

    for _ in 0..300 {
        let state = client.update_tick();

        // Check if ball velocity reversed (collision occurred)
        if state.ball_vx != previous_vx {
            // Velocity should have reversed
            assert_eq!(state.ball_vx, -previous_vx);
            return;
        }

        previous_vx = state.ball_vx;
    }

    panic!("Ball never collided with paddle");
}

#[test]
fn test_reset_game() {
    let env = Env::default();
    let contract_id = env.register(PongContract, ());
    let client = PongContractClient::new(&env, &contract_id);

    client.init_game();

    // Make some moves
    client.move_paddle(&1u32, &-1i32);
    client.move_paddle(&2u32, &1i32);
    client.update_tick();

    // Reset the game
    let reset_state = client.reset_game();

    // Should be back to initial state
    assert_eq!(reset_state.player1_paddle_y, FIELD_HEIGHT / 2);
    assert_eq!(reset_state.player2_paddle_y, FIELD_HEIGHT / 2);
    assert_eq!(reset_state.ball_x, FIELD_WIDTH / 2);
    assert_eq!(reset_state.ball_y, FIELD_HEIGHT / 2);
    assert_eq!(reset_state.player1_score, 0);
    assert_eq!(reset_state.player2_score, 0);
    assert!(reset_state.game_active);
}

#[test]
fn test_get_game_state() {
    let env = Env::default();
    let contract_id = env.register(PongContract, ());
    let client = PongContractClient::new(&env, &contract_id);

    let init_state = client.init_game();
    let retrieved_state = client.get_game_state();

    assert_eq!(init_state, retrieved_state);
}

#[test]
fn test_no_movement_when_game_inactive() {
    let env = Env::default();
    let contract_id = env.register(PongContract, ());
    let client = PongContractClient::new(&env, &contract_id);

    client.init_game();

    // Move player 2 paddle out of the way to let player 1 win
    for _ in 0..20 {
        client.move_paddle(&2u32, &1i32);
    }

    // Play until game ends
    for _ in 0..2000 {
        let state = client.update_tick();
        if !state.game_active {
            let final_state = state.clone();

            // Try to move paddle - should have no effect
            let after_move = client.move_paddle(&1u32, &-1i32);
            assert_eq!(after_move.player1_paddle_y, final_state.player1_paddle_y);

            // Try to update tick - should have no effect
            let after_tick = client.update_tick();
            assert_eq!(after_tick.ball_x, final_state.ball_x);
            assert_eq!(after_tick.ball_y, final_state.ball_y);

            return;
        }
    }

    panic!("Game never became inactive");
}
