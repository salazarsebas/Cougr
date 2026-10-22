use super::*;
use crate::components::{EMPTY, STATUS_PLAYING, STATUS_SOLVED};
use soroban_sdk::{Env, Vec};

fn setup(env: &Env) -> SudokuContractClient<'_> {
    let contract_id = env.register(SudokuContract, ());
    SudokuContractClient::new(env, &contract_id)
}

/// The standard puzzle fixture used across tests.
///
/// ```
///      0   1   2   3   4   5   6   7   8
///   0  5   3   .   .   7   .   .   .   .
///   1  6   .   .   1   9   5   .   .   .
///   2  .   9   8   .   .   .   .   6   .
///   3  8   .   .   .   6   .   .   .   3
///   4  4   .   .   8   .   3   .   .   1
///   5  7   .   .   .   2   .   .   .   6
///   6  .   6   .   .   .   .   2   8   .
///   7  .   .   .   4   1   9   .   .   5
///   8  .   .   .   .   8   .   .   7   9
/// ```
fn fixture(env: &Env) -> Vec<u32> {
    let vals: [u32; 81] = [
        5, 3, 0, 0, 7, 0, 0, 0, 0, 6, 0, 0, 1, 9, 5, 0, 0, 0, 0, 9, 8, 0, 0, 0, 0, 6, 0, 8, 0, 0,
        0, 6, 0, 0, 0, 3, 4, 0, 0, 8, 0, 3, 0, 0, 1, 7, 0, 0, 0, 2, 0, 0, 0, 6, 0, 6, 0, 0, 0, 0,
        2, 8, 0, 0, 0, 0, 4, 1, 9, 0, 0, 5, 0, 0, 0, 0, 8, 0, 0, 7, 9,
    ];
    let mut v = Vec::new(env);
    for val in vals.iter() {
        v.push_back(*val);
    }
    v
}

// ── Initialisation ────────────────────────────────────────────────────────────

#[test]
fn test_init_game_state() {
    let env = Env::default();
    let client = setup(&env);
    client.init_game(&fixture(&env));

    let state = client.get_state();
    assert_eq!(state.status, STATUS_PLAYING);
    assert_eq!(state.moves, 0);
}

#[test]
fn test_init_board_fixed_values() {
    let env = Env::default();
    let client = setup(&env);
    client.init_game(&fixture(&env));

    // Known fixed cells from the puzzle fixture
    let cell = client.get_cell(&0, &0);
    assert_eq!(cell.value, 5);
    assert!(cell.fixed);

    let cell = client.get_cell(&0, &1);
    assert_eq!(cell.value, 3);
    assert!(cell.fixed);

    let cell = client.get_cell(&4, &3);
    assert_eq!(cell.value, 8);
    assert!(cell.fixed);
}

#[test]
fn test_init_board_empty_cells() {
    let env = Env::default();
    let client = setup(&env);
    client.init_game(&fixture(&env));

    // Known empty cells from the puzzle fixture
    let cell = client.get_cell(&0, &2);
    assert_eq!(cell.value, EMPTY);
    assert!(!cell.fixed);

    let cell = client.get_cell(&1, &1);
    assert_eq!(cell.value, EMPTY);
    assert!(!cell.fixed);
}

#[test]
fn test_reinit_rejected() {
    let env = Env::default();
    let client = setup(&env);
    client.init_game(&fixture(&env));

    let result = client.try_init_game(&fixture(&env));
    assert!(result.is_err());
}

#[test]
fn test_is_solved_false_at_start() {
    let env = Env::default();
    let client = setup(&env);
    client.init_game(&fixture(&env));

    assert!(!client.is_solved());
}

// ── Move validation ───────────────────────────────────────────────────────────

#[test]
fn test_valid_move() {
    let env = Env::default();
    let client = setup(&env);
    client.init_game(&fixture(&env));

    // (0,2) is empty; correct value per solution is 4
    client.submit_value(&0, &2, &4);

    let cell = client.get_cell(&0, &2);
    assert_eq!(cell.value, 4);
    assert_eq!(client.get_state().moves, 1);
}

#[test]
fn test_reject_fixed_cell() {
    let env = Env::default();
    let client = setup(&env);
    client.init_game(&fixture(&env));

    // (0,0) is fixed (value=5)
    let result = client.try_submit_value(&0, &0, &1);
    assert!(result.is_err());
}

#[test]
fn test_reject_out_of_range_zero() {
    let env = Env::default();
    let client = setup(&env);
    client.init_game(&fixture(&env));

    let result = client.try_submit_value(&0, &2, &0);
    assert!(result.is_err());
}

#[test]
fn test_reject_out_of_range_ten() {
    let env = Env::default();
    let client = setup(&env);
    client.init_game(&fixture(&env));

    let result = client.try_submit_value(&0, &2, &10);
    assert!(result.is_err());
}

#[test]
fn test_reject_row_conflict() {
    let env = Env::default();
    let client = setup(&env);
    client.init_game(&fixture(&env));

    // Row 0 already has 5 at (0,0); placing 5 at empty (0,2) is a row conflict
    let result = client.try_submit_value(&0, &2, &5);
    assert!(result.is_err());
}

#[test]
fn test_reject_col_conflict() {
    let env = Env::default();
    let client = setup(&env);
    client.init_game(&fixture(&env));

    // Col 0 has 5 at (0,0); placing 5 at empty (2,0) is a col conflict
    let result = client.try_submit_value(&2, &0, &5);
    assert!(result.is_err());
}

#[test]
fn test_reject_block_conflict() {
    let env = Env::default();
    let client = setup(&env);
    client.init_game(&fixture(&env));

    // Block (0,0): contains 5,3,6,9,8. Placing 9 at (0,2) is a block conflict.
    let result = client.try_submit_value(&0, &2, &9);
    assert!(result.is_err());
}

#[test]
fn test_moves_increment() {
    let env = Env::default();
    let client = setup(&env);
    client.init_game(&fixture(&env));

    client.submit_value(&0, &2, &4); // correct solution value
    client.submit_value(&0, &3, &6); // correct solution value
    assert_eq!(client.get_state().moves, 2);
}

// ── Completion ────────────────────────────────────────────────────────────────

const SOLUTION: &[(u32, u32, u32)] = &[
    (0, 2, 4),
    (0, 3, 6),
    (0, 5, 8),
    (0, 6, 9),
    (0, 7, 1),
    (0, 8, 2),
    (1, 1, 7),
    (1, 2, 2),
    (1, 6, 3),
    (1, 7, 4),
    (1, 8, 8),
    (2, 0, 1),
    (2, 3, 3),
    (2, 4, 4),
    (2, 5, 2),
    (2, 6, 5),
    (2, 8, 7),
    (3, 1, 5),
    (3, 2, 9),
    (3, 3, 7),
    (3, 5, 1),
    (3, 6, 4),
    (3, 7, 2),
    (4, 1, 2),
    (4, 2, 6),
    (4, 4, 5),
    (4, 6, 7),
    (4, 7, 9),
    (5, 1, 1),
    (5, 2, 3),
    (5, 3, 9),
    (5, 5, 4),
    (5, 6, 8),
    (5, 7, 5),
    (6, 0, 9),
    (6, 2, 1),
    (6, 3, 5),
    (6, 4, 3),
    (6, 5, 7),
    (6, 8, 4),
    (7, 0, 2),
    (7, 1, 8),
    (7, 2, 7),
    (7, 6, 6),
    (7, 7, 3),
    (8, 0, 3),
    (8, 1, 4),
    (8, 2, 5),
    (8, 3, 2),
    (8, 5, 6),
    (8, 6, 1),
];

#[test]
fn test_puzzle_solved_after_full_solution() {
    let env = Env::default();
    let client = setup(&env);
    client.init_game(&fixture(&env));

    for &(r, c, v) in SOLUTION {
        client.submit_value(&r, &c, &v);
    }

    assert!(client.is_solved());
    assert_eq!(client.get_state().status, STATUS_SOLVED);
    assert_eq!(client.get_state().moves, 51);
}

#[test]
fn test_reject_move_after_solved() {
    let env = Env::default();
    let client = setup(&env);
    client.init_game(&fixture(&env));

    for &(r, c, v) in SOLUTION {
        client.submit_value(&r, &c, &v);
    }
    assert!(client.is_solved());

    // The "Puzzle already solved" guard in submit_value fires before any cell
    // validation, so any call is rejected regardless of cell or value chosen.
    let result = client.try_submit_value(&0, &2, &4);
    assert!(result.is_err());
}

// ── Contradictory givens rejection tests (#466) ───────────────────────────────

#[test]
#[should_panic(expected = "Puzzle givens violate Sudoku constraints")]
fn test_reject_duplicate_givens_in_row() {
    let env = Env::default();
    let client = setup(&env);
    // Duplicate 5 at (0, 0) and (0, 2) in the first row
    let mut puzzle = fixture(&env);
    puzzle.set(2, 5);
    client.init_game(&puzzle);
}

#[test]
#[should_panic(expected = "Puzzle givens violate Sudoku constraints")]
fn test_reject_duplicate_givens_in_column() {
    let env = Env::default();
    let client = setup(&env);
    // Cell (0, 0) is 5. Place another 5 at (2, 0) in the first column
    let mut puzzle = fixture(&env);
    puzzle.set(18, 5);
    client.init_game(&puzzle);
}

#[test]
#[should_panic(expected = "Puzzle givens violate Sudoku constraints")]
fn test_reject_duplicate_givens_in_block() {
    let env = Env::default();
    let client = setup(&env);
    // Top-left 3x3 block has 5 at (0, 0) and 3 at (0, 1). Place duplicate 5 at (1, 1)
    let mut puzzle = fixture(&env);
    puzzle.set(10, 5);
    client.init_game(&puzzle);
}

// ── get_cell bounds checking tests (#466) ────────────────────────────────────

#[test]
#[should_panic(expected = "Cell out of bounds")]
fn test_get_cell_out_of_bounds_col() {
    let env = Env::default();
    let client = setup(&env);
    client.init_game(&fixture(&env));
    client.get_cell(&0, &9);
}

#[test]
#[should_panic(expected = "Cell out of bounds")]
fn test_get_cell_out_of_bounds_row() {
    let env = Env::default();
    let client = setup(&env);
    client.init_game(&fixture(&env));
    client.get_cell(&9, &0);
}
