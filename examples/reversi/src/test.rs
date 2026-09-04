use super::*;
use crate::components::{BLACK, EMPTY, STATUS_ACTIVE, WHITE, WINNER_NONE};
use soroban_sdk::{testutils::Address as _, Env};

fn setup(env: &Env) -> (Address, Address, ReversiContractClient<'_>) {
    let contract_id = env.register(ReversiContract, ());
    let client = ReversiContractClient::new(env, &contract_id);
    let p1 = Address::generate(env);
    let p2 = Address::generate(env);
    (p1, p2, client)
}

#[test]
fn test_init_board_layout() {
    let env = Env::default();
    let (p1, p2, client) = setup(&env);
    client.init_game(&p1, &p2);

    let board = client.get_board();
    assert_eq!(board.width, 8);
    assert_eq!(board.height, 8);
    // Standard opening: (3,3)=W  (3,4)=B  (4,3)=B  (4,4)=W
    assert_eq!(board.cells.get(3 * 8 + 3), Some(WHITE));
    assert_eq!(board.cells.get(3 * 8 + 4), Some(BLACK));
    assert_eq!(board.cells.get(4 * 8 + 3), Some(BLACK));
    assert_eq!(board.cells.get(4 * 8 + 4), Some(WHITE));
    assert_eq!(board.cells.get(0), Some(EMPTY));
}

#[test]
fn test_init_score() {
    let env = Env::default();
    let (p1, p2, client) = setup(&env);
    client.init_game(&p1, &p2);

    let score = client.get_score();
    assert_eq!(score.black_count, 2);
    assert_eq!(score.white_count, 2);
    assert_eq!(score.winner, WINNER_NONE);
}

#[test]
fn test_init_game_state() {
    let env = Env::default();
    let (p1, p2, client) = setup(&env);
    client.init_game(&p1, &p2);

    let state = client.get_state();
    assert_eq!(state.current_player, BLACK);
    assert_eq!(state.pass_count, 0);
    assert_eq!(state.status, STATUS_ACTIVE);
}

#[test]
fn test_illegal_move_occupied_cell() {
    let env = Env::default();
    env.mock_all_auths();
    let (p1, p2, client) = setup(&env);
    client.init_game(&p1, &p2);
    // (3,3) is occupied by WHITE at start
    let result = client.try_submit_move(&p1, &3u32, &3u32);
    assert!(result.is_err());
}

#[test]
fn test_illegal_move_no_flip() {
    let env = Env::default();
    env.mock_all_auths();
    let (p1, p2, client) = setup(&env);
    client.init_game(&p1, &p2);
    // (0,0) is empty but flips nothing for Black
    let result = client.try_submit_move(&p1, &0u32, &0u32);
    assert!(result.is_err());
}

#[test]
fn test_legal_move_horizontal_flip() {
    // Black at (3,2): right scan hits (3,3)=W then (3,4)=B → flips (3,3)
    let env = Env::default();
    env.mock_all_auths();
    let (p1, p2, client) = setup(&env);
    client.init_game(&p1, &p2);
    client.submit_move(&p1, &3u32, &2u32);
    let board = client.get_board();
    assert_eq!(board.cells.get(3 * 8 + 2), Some(BLACK)); // placed
    assert_eq!(board.cells.get(3 * 8 + 3), Some(BLACK)); // flipped
    assert_eq!(board.cells.get(3 * 8 + 4), Some(BLACK)); // unchanged
}

#[test]
fn test_legal_move_vertical_flip() {
    // Black at (2,3): down scan hits (3,3)=W then (4,3)=B → flips (3,3)
    let env = Env::default();
    env.mock_all_auths();
    let (p1, p2, client) = setup(&env);
    client.init_game(&p1, &p2);
    client.submit_move(&p1, &2u32, &3u32);
    let board = client.get_board();
    assert_eq!(board.cells.get(2 * 8 + 3), Some(BLACK)); // placed
    assert_eq!(board.cells.get(3 * 8 + 3), Some(BLACK)); // flipped
    assert_eq!(board.cells.get(4 * 8 + 3), Some(BLACK)); // unchanged
}

#[test]
fn test_legal_move_diagonal_flip() {
    // Black at (3,2) first, then White at (2,2):
    // White at (2,2): down-right dr=1,dc=1: (3,3)=B(opp to White) → (4,4)=W(mine) → 1 flip ✓
    let env = Env::default();
    env.mock_all_auths();
    let (p1, p2, client) = setup(&env);
    client.init_game(&p1, &p2);
    client.submit_move(&p1, &3u32, &2u32); // Black plays (3,2)
    client.submit_move(&p2, &2u32, &2u32); // White plays (2,2) - diagonal flip
    let board = client.get_board();
    assert_eq!(board.cells.get(2 * 8 + 2), Some(WHITE)); // placed
    assert_eq!(board.cells.get(3 * 8 + 3), Some(WHITE)); // flipped diagonally
}

#[test]
fn test_score_updates_after_move() {
    // Black at (3,2): places 1, flips 1 → Black=4, White=1
    let env = Env::default();
    env.mock_all_auths();
    let (p1, p2, client) = setup(&env);
    client.init_game(&p1, &p2);
    client.submit_move(&p1, &3u32, &2u32);
    let score = client.get_score();
    assert_eq!(score.black_count, 4);
    assert_eq!(score.white_count, 1);
}

#[test]
fn test_turn_alternates() {
    let env = Env::default();
    env.mock_all_auths();
    let (p1, p2, client) = setup(&env);
    client.init_game(&p1, &p2);

    // Black moves → White's turn
    client.submit_move(&p1, &3u32, &2u32);
    assert_eq!(client.get_state().current_player, WHITE);

    // White moves → Black's turn
    client.submit_move(&p2, &2u32, &2u32);
    assert_eq!(client.get_state().current_player, BLACK);
}

#[test]
fn test_wrong_player_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (p1, p2, client) = setup(&env);
    client.init_game(&p1, &p2);

    // White tries to move on Black's turn
    let result = client.try_submit_move(&p2, &3u32, &2u32);
    assert!(result.is_err());
}

#[test]
fn test_pass_count_normal_after_move() {
    // After a normal alternating move, pass_count should be 0
    let env = Env::default();
    env.mock_all_auths();
    let (p1, p2, client) = setup(&env);
    client.init_game(&p1, &p2);

    client.submit_move(&p1, &3u32, &2u32);
    assert_eq!(client.get_state().pass_count, 0);
}

#[test]
fn test_multi_move_sequence_stays_active() {
    // Play 3 valid moves and verify game is still active
    let env = Env::default();
    env.mock_all_auths();
    let (p1, p2, client) = setup(&env);
    client.init_game(&p1, &p2);

    // Black at (3,2) - horizontal flip
    client.submit_move(&p1, &3u32, &2u32);
    // White at (2,2) - diagonal flip
    client.submit_move(&p2, &2u32, &2u32);
    // Black at (2,3) - vertical flip
    client.submit_move(&p1, &2u32, &3u32);

    let state = client.get_state();
    assert_eq!(state.status, STATUS_ACTIVE);

    let score = client.get_score();
    assert_eq!(score.winner, WINNER_NONE);
    // Total pieces must be more than the starting 4
    assert!(score.black_count + score.white_count > 4);
}

#[test]
fn test_reinit_rejected() {
    // init_game a second time should panic
    let env = Env::default();
    env.mock_all_auths();
    let (p1, p2, client) = setup(&env);
    client.init_game(&p1, &p2);

    let result = client.try_init_game(&p1, &p2);
    assert!(result.is_err());
}
