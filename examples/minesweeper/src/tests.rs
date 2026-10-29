use super::*;
use soroban_sdk::Env;

// ==================== Initialization Tests ====================

#[test]
fn test_init_game() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    let state = client.init_game();

    assert_eq!(state.rows, ROWS);
    assert_eq!(state.cols, COLS);
    assert_eq!(state.total_mines, MINES);
    assert_eq!(state.status, STATUS_PLAYING);
    assert_eq!(state.revealed_count, 0);
    assert_eq!(state.safe_cells_remaining, (ROWS * COLS) - MINES);
}

#[test]
fn test_get_board_after_init() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    client.init_game();

    let board = client.get_board();
    assert_eq!(board.len() as usize, (ROWS * COLS) as usize);

    // All cells should be hidden initially
    for i in 0..board.len() {
        assert_eq!(board.get(i).unwrap_or(0), CELL_HIDDEN);
    }
}

// ==================== Safe Cell Reveal Tests ====================

#[test]
fn test_reveal_safe_cell() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    client.init_game();

    // Try to reveal a cell that we know is safe (not in mine positions)
    // Based on deterministic placement, (0, 0) should be safe
    let result = client.reveal_cell(&0, &0);

    assert!(result.success);
    assert!(!result.is_mine);
    assert_eq!(result.message, symbol_short!("ok"));

    // Should have some adjacent count (0-8)
    assert!(result.adjacent_mines <= 8);
}

#[test]
fn test_reveal_cell_returns_adjacent_count() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    client.init_game();

    // Reveal multiple cells and verify they return adjacent counts
    let result1 = client.reveal_cell(&0, &0);
    assert!(result1.success);
    assert!(!result1.is_mine);

    let result2 = client.reveal_cell(&0, &8);
    assert!(result2.success);
    assert!(!result2.is_mine);

    // Both should have valid adjacent counts
    assert!(result1.adjacent_mines <= 8);
    assert!(result2.adjacent_mines <= 8);
}

#[test]
fn test_reveal_increments_counter() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    client.init_game();

    let initial_state = client.get_state();
    assert_eq!(initial_state.revealed_count, 0);

    // Reveal a safe cell
    client.reveal_cell(&0, &0);

    let state = client.get_state();
    assert_eq!(state.revealed_count, 1);
    assert_eq!(state.safe_cells_remaining, ((ROWS * COLS) - MINES) - 1);
}

// ==================== Mine Reveal Tests ====================

#[test]
fn test_reveal_mine_triggers_loss() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    client.init_game();

    // Reveal a known mine position from deterministic layout
    // Mine at (1, 1)
    let result = client.reveal_cell(&1, &1);

    assert!(result.success);
    assert!(result.is_mine);
    assert_eq!(result.message, symbol_short!("boom"));

    // Game should be over
    assert!(client.is_finished());
    let state = client.get_state();
    assert_eq!(state.status, STATUS_LOST);
}

#[test]
fn test_multiple_mine_reveals() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    client.init_game();

    // Reveal one mine
    let result1 = client.reveal_cell(&1, &1);
    assert!(result1.is_mine);

    // Game should be lost
    assert!(client.is_finished());

    // Try to reveal another mine - should fail because game is over
    let result2 = client.reveal_cell(&3, &3);
    assert!(!result2.success);
    assert_eq!(result2.message, symbol_short!("over"));
}

// ==================== Repeated Reveal Tests ====================

#[test]
fn test_reveal_already_revealed_cell() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    client.init_game();

    // Reveal a cell
    let result1 = client.reveal_cell(&0, &0);
    assert!(result1.success);

    // Try to reveal the same cell again
    let result2 = client.reveal_cell(&0, &0);
    assert!(!result2.success);
    assert_eq!(result2.message, symbol_short!("revealed"));
}

#[test]
fn test_reveal_multiple_times_different_cells() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    client.init_game();

    // Reveal multiple different safe cells
    let result1 = client.reveal_cell(&0, &0);
    assert!(result1.success);

    let result2 = client.reveal_cell(&0, &1);
    assert!(result2.success);

    let result3 = client.reveal_cell(&0, &2);
    assert!(result3.success);

    // All should be safe
    assert!(!result1.is_mine);
    assert!(!result2.is_mine);
    assert!(!result3.is_mine);

    // Counter should increment
    let state = client.get_state();
    assert_eq!(state.revealed_count, 3);
}

// ==================== Out of Bounds Tests ====================

#[test]
fn test_reveal_out_of_bounds() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    client.init_game();

    // Test various out of bounds positions
    let result1 = client.reveal_cell(&10, &5); // Row out of bounds
    assert!(!result1.success);
    assert_eq!(result1.message, symbol_short!("invalid"));

    let result2 = client.reveal_cell(&5, &10); // Col out of bounds
    assert!(!result2.success);
    assert_eq!(result2.message, symbol_short!("invalid"));

    let result3 = client.reveal_cell(&100, &100); // Both out of bounds
    assert!(!result3.success);
    assert_eq!(result3.message, symbol_short!("invalid"));
}

// ==================== Win Condition Tests ====================

#[test]
fn test_win_condition_tracking() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    client.init_game();

    let state = client.get_state();
    assert_eq!(state.status, STATUS_PLAYING);
    assert_eq!(state.safe_cells_remaining, (ROWS * COLS) - MINES);
}

#[test]
fn test_game_state_after_loss() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    client.init_game();

    // Hit a mine
    client.reveal_cell(&1, &1);

    let state = client.get_state();
    assert_eq!(state.status, STATUS_LOST);
    assert!(client.is_finished());
}

// ==================== Visible Cell State Tests ====================

#[test]
fn test_get_visible_cell_hidden() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    client.init_game();

    // Hidden cell should not be revealed
    let visible = client.get_visible_cell(&0, &0);
    assert!(!visible.is_revealed);
    assert!(!visible.is_mine);
    assert_eq!(visible.adjacent_mines, 0);
}

#[test]
fn test_get_visible_cell_revealed() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    client.init_game();

    // Reveal a cell
    client.reveal_cell(&0, &0);

    // Now it should be revealed with adjacent count
    let visible = client.get_visible_cell(&0, &0);
    assert!(visible.is_revealed);
    assert!(!visible.is_mine);
    assert!(visible.adjacent_mines <= 8);
}

#[test]
fn test_get_visible_cell_mine_after_reveal() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    client.init_game();

    // Before revealing mine
    let visible1 = client.get_visible_cell(&1, &1);
    assert!(!visible1.is_revealed);
    assert!(!visible1.is_mine);

    // Reveal the mine
    client.reveal_cell(&1, &1);

    // After revealing, should show as mine
    let visible2 = client.get_visible_cell(&1, &1);
    assert!(visible2.is_revealed);
    assert!(visible2.is_mine);
}

#[test]
fn test_get_visible_cell_out_of_bounds() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    client.init_game();

    let visible = client.get_visible_cell(&10, &10);
    assert!(!visible.is_revealed);
    assert!(!visible.is_mine);
    assert_eq!(visible.adjacent_mines, 0);
}

// ==================== Reset Game Tests ====================

#[test]
fn test_reset_game() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    client.init_game();

    // Make some moves
    client.reveal_cell(&0, &0);
    client.reveal_cell(&0, &1);

    let state1 = client.get_state();
    assert_eq!(state1.revealed_count, 2);

    // Reset
    let state2 = client.reset_game();
    assert_eq!(state2.revealed_count, 0);
    assert_eq!(state2.status, STATUS_PLAYING);
    assert_eq!(state2.safe_cells_remaining, (ROWS * COLS) - MINES);
}

// ==================== Adjacent Mine Count Verification ====================

#[test]
fn test_adjacent_mine_counts_across_board() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    client.init_game();

    // Reveal several cells and verify adjacent counts are reasonable
    let positions = [(0, 0), (0, 4), (4, 0), (4, 4), (8, 8)];

    for (row, col) in positions.iter() {
        let result = client.reveal_cell(row, col);
        if result.success && !result.is_mine {
            // Adjacent count should be between 0 and 8
            assert!(result.adjacent_mines <= 8);
        }
    }
}

// ==================== Edge Cases ====================

#[test]
fn test_corner_cells() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    client.init_game();

    // Test all four corners
    let corners = [(0, 0), (0, 8), (8, 0), (8, 8)];

    for (row, col) in corners.iter() {
        let result = client.reveal_cell(row, col);
        // Should succeed (unless it's a mine, which is fine)
        assert!(result.success);
    }
}

#[test]
fn test_cannot_play_after_game_over() {
    let env = Env::default();
    let contract_id = env.register(MinesweeperContract, ());
    let client = MinesweeperContractClient::new(&env, &contract_id);

    client.init_game();

    // Hit a mine to end game
    client.reveal_cell(&1, &1);
    assert!(client.is_finished());

    // Try to play after game over
    let result = client.reveal_cell(&5, &5);
    assert!(!result.success);
    assert_eq!(result.message, symbol_short!("over"));
}

#[test]
fn test_deterministic_mine_placement() {
    let env1 = Env::default();
    let contract_id1 = env1.register(MinesweeperContract, ());
    let client1 = MinesweeperContractClient::new(&env1, &contract_id1);

    let env2 = Env::default();
    let contract_id2 = env2.register(MinesweeperContract, ());
    let client2 = MinesweeperContractClient::new(&env2, &contract_id2);

    client1.init_game();
    client2.init_game();

    // Reveal same position in both games
    let result1 = client1.reveal_cell(&1, &1);
    let result2 = client2.reveal_cell(&1, &1);

    // Both should hit the mine (deterministic placement)
    assert_eq!(result1.is_mine, result2.is_mine);

    // If both are mines, good - proves determinism
    if result1.is_mine {
        assert!(result2.is_mine);
    }
}
