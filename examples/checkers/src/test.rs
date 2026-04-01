use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{CheckersContract, CheckersContractClient, CheckersError, GameStatus};

// Helpers
fn setup() -> (Env, CheckersContractClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(CheckersContract, ());
    let client = CheckersContractClient::new(&env, &id);
    let p1 = Address::generate(&env);
    let p2 = Address::generate(&env);
    (env, client, p1, p2)
}

fn cell(client: &CheckersContractClient, row: u32, col: u32) -> i32 {
    client.get_board().cells.get(row * 8 + col).unwrap()
}

// 1. Initialisation
#[test]
fn test_init_sets_standard_start_position() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);

    assert_eq!(cell(&client, 0, 1), 1, "P1 piece at (0,1)");
    assert_eq!(cell(&client, 2, 7), 1, "P1 piece at (2,7)");
    assert_eq!(cell(&client, 5, 0), -1, "P2 piece at (5,0)");
    assert_eq!(cell(&client, 7, 6), -1, "P2 piece at (7,6)");
    assert_eq!(cell(&client, 0, 0), 0, "light square (0,0) empty");
    assert_eq!(cell(&client, 4, 4), 0, "neutral square (4,4) empty");

    let state = client.get_state();
    assert_eq!(state.turn.current_player, 1, "P1 moves first");
    assert_eq!(state.turn.move_number, 1);
    assert_eq!(state.status.status, GameStatus::Active);
    assert_eq!(state.status.winner, 0);
}

#[test]
fn test_double_init_fails() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);
    assert_eq!(
        client.try_init_game(&p1, &p2),
        Err(Ok(CheckersError::AlreadyInitialised))
    );
}

#[test]
fn test_get_state_before_init_returns_error() {
    let (_env, client, _p1, _p2) = setup();
    assert_eq!(
        client.try_get_state(),
        Err(Ok(CheckersError::NotInitialised))
    );
}

#[test]
fn test_get_board_before_init_returns_error() {
    let (_env, client, _p1, _p2) = setup();
    assert_eq!(
        client.try_get_board(),
        Err(Ok(CheckersError::NotInitialised))
    );
}

#[test]
fn test_get_current_player_is_p1_after_init() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);
    assert_eq!(client.get_current_player(), p1);
}

// 2. Legal diagonal movement
#[test]
fn test_legal_step_moves_piece() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);

    client.submit_move(&p1, &2, &1, &3, &2);

    assert_eq!(cell(&client, 2, 1), 0, "source cleared");
    assert_eq!(cell(&client, 3, 2), 1, "piece at destination");
}

#[test]
fn test_turn_advances_after_step() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);

    client.submit_move(&p1, &2, &1, &3, &2);

    let state = client.get_state();
    assert_eq!(state.turn.current_player, 2);
    assert_eq!(state.turn.move_number, 2);
    assert_eq!(client.get_current_player(), p2);
}

// 3. Invalid movement rejection
#[test]
fn test_light_square_destination_rejected() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);

    assert_eq!(
        client.try_submit_move(&p1, &2, &0, &3, &1),
        Err(Ok(CheckersError::NotDarkSquare))
    );
}

#[test]
fn test_moving_opponent_piece_rejected() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);

    assert_eq!(
        client.try_submit_move(&p1, &5, &0, &4, &1),
        Err(Ok(CheckersError::NotYourPiece))
    );
}

#[test]
fn test_moving_empty_square_rejected() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);

    assert_eq!(
        client.try_submit_move(&p1, &3, &0, &4, &1),
        Err(Ok(CheckersError::NotYourPiece))
    );
}

#[test]
fn test_non_diagonal_move_rejected() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);

    // (2,1)→(2,3): same row — geometry check fires before destination check.
    assert_eq!(
        client.try_submit_move(&p1, &2, &1, &2, &3),
        Err(Ok(CheckersError::IllegalMove))
    );
}

#[test]
fn test_backward_step_for_man_rejected() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);

    // Advance P1 to row 3.
    client.submit_move(&p1, &2, &1, &3, &2);
    // P2 filler on the far right — no capture created for P1 at (3,2).
    client.submit_move(&p2, &5, &6, &4, &7);
    // P1 tries to retreat: legal_steps returns nothing backward → IllegalMove.
    assert_eq!(
        client.try_submit_move(&p1, &3, &2, &2, &1),
        Err(Ok(CheckersError::IllegalMove))
    );
}

#[test]
fn test_out_of_bounds_rejected() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);

    assert_eq!(
        client.try_submit_move(&p1, &0, &1, &1, &8),
        Err(Ok(CheckersError::OutOfBounds))
    );
}

#[test]
fn test_destination_occupied_rejected() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);

    assert_eq!(
        client.try_submit_move(&p1, &0, &1, &1, &0),
        Err(Ok(CheckersError::DestinationOccupied))
    );
}

// 4. Wrong-turn rejection
#[test]
fn test_p2_cannot_move_on_p1_turn() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);

    assert_eq!(
        client.try_submit_move(&p2, &5, &0, &4, &1),
        Err(Ok(CheckersError::WrongTurn))
    );
}

#[test]
fn test_unknown_address_rejected() {
    let (env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);

    let stranger = Address::generate(&env);
    assert_eq!(
        client.try_submit_move(&stranger, &2, &1, &3, &2),
        Err(Ok(CheckersError::NotAPlayer))
    );
}
// 5. Capture execution
#[test]
fn test_capture_removes_opponent_piece() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);

    client.submit_move(&p1, &2, &3, &3, &4);
    client.submit_move(&p2, &5, &6, &4, &5);
    client.submit_move(&p1, &3, &4, &5, &6);

    assert_eq!(cell(&client, 3, 4), 0, "source vacated");
    assert_eq!(cell(&client, 4, 5), 0, "captured piece removed");
    assert_eq!(cell(&client, 5, 6), 1, "P1 piece at landing square");
}

// 6. Forced-capture enforcement
#[test]
fn test_step_rejected_when_capture_available() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);

    client.submit_move(&p1, &2, &3, &3, &4);
    client.submit_move(&p2, &5, &6, &4, &5);

    assert_eq!(
        client.try_submit_move(&p1, &2, &1, &3, &0),
        Err(Ok(CheckersError::MustCapture))
    );
}
// 7. King promotion
#[test]
fn test_piece_promoted_to_king_at_back_rank() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);

    client.submit_move(&p1, &2, &5, &3, &4); // T1 P1: corridor advance
    client.submit_move(&p2, &5, &0, &4, &1); // T1 P2: unlock step 1 — frees (5,0)
    client.submit_move(&p1, &2, &7, &3, &6); // T2 P1: safe filler
    client.submit_move(&p2, &6, &1, &5, &0); // T2 P2: unlock step 2 — frees (6,1)
    client.submit_move(&p1, &2, &1, &3, &0); // T3 P1: safe filler (OOB blocks P2 cap)
    client.submit_move(&p2, &7, &0, &6, &1); // T3 P2: unlock step 3 — frees (7,0)
    client.submit_move(&p1, &1, &0, &2, &1); // T4 P1: filler ((2,1) vacated at T3)
    client.submit_move(&p2, &5, &2, &4, &3); // T4 P2: arm forced capture for P1@(3,4)
    client.submit_move(&p1, &3, &4, &5, &2); // T5 P1 hop 1: capture over (4,3)
    client.submit_move(&p1, &5, &2, &7, &0); // T5 P1 hop 2: chain capture over (6,1) → promotes!

    assert_eq!(
        cell(&client, 7, 0),
        2,
        "P1 piece promoted to king (value 2) at row 7, col 0"
    );
}

// 8. King movement — kings can move backward
#[test]
fn test_king_can_move_backward() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);

    // promotion sequence
    client.submit_move(&p1, &2, &5, &3, &4);
    client.submit_move(&p2, &5, &0, &4, &1);
    client.submit_move(&p1, &2, &7, &3, &6);
    client.submit_move(&p2, &6, &1, &5, &0);
    client.submit_move(&p1, &2, &1, &3, &0);
    client.submit_move(&p2, &7, &0, &6, &1);
    client.submit_move(&p1, &1, &0, &2, &1);
    client.submit_move(&p2, &5, &2, &4, &3);
    client.submit_move(&p1, &3, &4, &5, &2);
    client.submit_move(&p1, &5, &2, &7, &0);

    assert_eq!(cell(&client, 7, 0), 2, "king at (7,0)");

    // T6 P2: (6,3)→(5,2) — only safe step for P2 (verified exhaustively)
    client.submit_move(&p2, &6, &3, &5, &2);

    // T7 P1: safe filler — (0,1)→(1,0); no P2 forced cap after this
    client.submit_move(&p1, &0, &1, &1, &0);

    // T8 P2: (5,4)→(4,3) — safe filler; no P1 forced cap after this
    client.submit_move(&p2, &5, &4, &4, &3);

    // T9 P1: king retreats (7,0)→(6,1). (6,1) was vacated by the hop-2 capture.
    client.submit_move(&p1, &7, &0, &6, &1);

    assert_eq!(cell(&client, 6, 1), 2, "king retreated to (6,1)");
    assert_eq!(cell(&client, 7, 0), 0, "previous king square is empty");
}

// 9. Win detection
#[test]
fn test_game_is_active_at_start() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);

    let state = client.get_state();
    assert_eq!(state.status.status, GameStatus::Active);
    assert_eq!(state.status.winner, 0);
}

#[test]
fn test_game_over_error_after_game_finished() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);

    client.submit_move(&p1, &2, &3, &3, &4);
    client.submit_move(&p2, &5, &6, &4, &5);
    client.submit_move(&p1, &3, &4, &5, &6);

    let state = client.get_state();
    assert_eq!(state.status.status, GameStatus::Active);
}

// 10. get_board and get_current_player
#[test]
fn test_get_board_has_64_cells() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);
    assert_eq!(client.get_board().cells.len(), 64);
}

#[test]
fn test_current_player_alternates_each_turn() {
    let (_env, client, p1, p2) = setup();
    client.init_game(&p1, &p2);

    assert_eq!(client.get_current_player(), p1);

    client.submit_move(&p1, &2, &1, &3, &2);
    assert_eq!(client.get_current_player(), p2);

    client.submit_move(&p2, &5, &0, &4, &1);
    assert_eq!(client.get_current_player(), p1);
}
