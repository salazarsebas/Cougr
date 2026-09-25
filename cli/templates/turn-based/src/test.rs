//! Integration tests for {{crate_name}}.
//!
//! These run through `cougr_core::test::GameHarness`, the sandbox the canonical
//! examples use: it registers the contract in a fresh `Env`, mints player
//! addresses, and pairs with `Scenario` to drive alternating turns.

use crate::components::{BOARD_HEIGHT, BOARD_WIDTH, CELL_COUNT, DRAW, IN_PROGRESS, MARK_O, MARK_X, WIN_LENGTH, X_WINS};
use crate::systems::{detect_status, validate_move, MoveError};
use crate::{{ContractName}};
use crate::{{ContractName}}Client;
use cougr_core::test::{GameHarness, PlayerSlot, Scenario};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol_short, Address, Env};

/// A harness with two authorized players and an initialised match.
fn started_game() -> (GameHarness, Address, Address) {
    let env = Env::default();
    let mut harness = GameHarness::new(env, {{ContractName}});
    harness.mock_players(2);
    harness.mock_all_auths();

    let player_x = harness.player(PlayerSlot(0)).clone();
    let player_o = harness.player(PlayerSlot(1)).clone();

    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());
    client.init_game(&player_x, &player_o);

    (harness, player_x, player_o)
}

#[test]
fn init_game_starts_with_an_empty_board() {
    let (harness, player_x, _) = started_game();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let state = client.get_state();
    assert_eq!(state.cells.len(), CELL_COUNT);
    assert!(state.cells.iter().all(|cell| cell == 0));
    assert_eq!(state.player_x, player_x);
    assert!(state.is_x_turn);
    assert_eq!(state.status, IN_PROGRESS);
}

#[test]
fn players_alternate_marks() {
    let (harness, player_x, player_o) = started_game();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    assert!(client.make_move(&player_x, &0).success);
    assert!(client.make_move(&player_o, &4).success);

    let state = client.get_state();
    assert_eq!(state.cells.get(0).unwrap(), MARK_X);
    assert_eq!(state.cells.get(4).unwrap(), MARK_O);
    assert_eq!(state.move_count, 2);
    assert!(state.is_x_turn);
}

#[test]
fn playing_out_of_turn_is_rejected() {
    let (harness, _, player_o) = started_game();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let result = client.make_move(&player_o, &0);
    assert!(!result.success);
    assert_eq!(result.message, symbol_short!("notturn"));
    assert_eq!(result.game_state.move_count, 0);
}

#[test]
fn occupied_cells_and_out_of_range_positions_are_rejected() {
    let (harness, player_x, player_o) = started_game();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    client.make_move(&player_x, &0);

    assert_eq!(
        client.make_move(&player_o, &0).message,
        symbol_short!("occupied")
    );
    assert_eq!(
        client.make_move(&player_o, &99).message,
        symbol_short!("bounds")
    );
}

#[test]
fn a_third_address_cannot_play() {
    let (harness, _, _) = started_game();
    let stranger = Address::generate(harness.env());
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let result = client.make_move(&stranger, &0);
    assert!(!result.success);
    assert_eq!(result.message, symbol_short!("notplay"));
}

#[test]
fn scenario_plays_a_winning_column_for_x() {
    let (harness, _, _) = started_game();

    // X takes the left column while O answers in the middle one.
    Scenario::new("x wins the left column")
        .players(2)
        .turns(2 * WIN_LENGTH - 1)
        .run(&harness, |slot, turn, h| {
            let client = {{ContractName}}Client::new(h.env(), h.contract_id());
            let player = h.player(slot).clone();
            let row = turn.0 / 2;
            let col = turn.0 % 2;
            let result = client.make_move(&player, &(row * BOARD_WIDTH + col));
            assert!(result.success, "move {} was rejected", turn.0);
        });

    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());
    let winner = client.get_winner();

    assert_eq!(client.get_state().status, X_WINS);
    assert_eq!(winner, Some(harness.player(PlayerSlot(0)).clone()));
}

#[test]
fn moves_after_the_game_ends_are_rejected() {
    let (harness, player_x, player_o) = started_game();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    for turn in 0..(2 * WIN_LENGTH - 1) {
        let player = if turn % 2 == 0 { &player_x } else { &player_o };
        let position = (turn / 2) * BOARD_WIDTH + turn % 2;
        assert!(client.make_move(player, &position).success);
    }

    let next_cell = (WIN_LENGTH - 1) * BOARD_WIDTH + 1;

    assert_eq!(
        client.make_move(&player_o, &next_cell).message,
        symbol_short!("gameover")
    );
    assert!(!client.is_valid_move(&next_cell));
}

#[test]
fn reset_game_clears_the_board_and_keeps_the_players() {
    let (harness, player_x, player_o) = started_game();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    client.make_move(&player_x, &0);
    let state = client.reset_game();

    assert_eq!(state.move_count, 0);
    assert!(state.cells.iter().all(|cell| cell == 0));
    assert_eq!(state.player_x, player_x);
    assert_eq!(state.player_o, player_o);
}

#[test]
fn detect_status_reports_a_full_board_as_a_draw() {
    let env = Env::default();
    let cells = if BOARD_WIDTH == 3 && BOARD_HEIGHT == 3 && WIN_LENGTH == 3 {
        // X O X / X O O / O X X - full board with no line.
        soroban_sdk::vec![&env, 1, 2, 1, 1, 2, 2, 2, 1, 1]
    } else if BOARD_WIDTH == 5 && BOARD_HEIGHT == 4 && WIN_LENGTH == 4 {
        // X X O O X / O O X X O / X O X O X / X O O X O.
        soroban_sdk::vec![&env, 1, 1, 2, 2, 1, 2, 2, 1, 1, 2, 1, 2, 1, 2, 1, 1, 2, 2, 1, 2]
    } else {
        return;
    };

    assert_eq!(detect_status(&cells, CELL_COUNT), DRAW);
}

#[test]
fn configured_horizontal_line_wins() {
    let (harness, player_x, player_o) = started_game();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());
    for col in 0..WIN_LENGTH {
        assert!(client.make_move(&player_x, &col).success);
        if col + 1 < WIN_LENGTH {
            assert!(client.make_move(&player_o, &(BOARD_WIDTH + col)).success);
        }
    }
    assert_eq!(client.get_state().status, X_WINS);
}

#[test]
fn validate_move_rejects_a_non_player() {
    let env = Env::default();
    let board = crate::components::Board::new(&env);
    let turn = crate::components::TurnState::opening();

    assert_eq!(
        validate_move(&board, &turn, 0, false, false),
        Err(MoveError::NotAPlayer)
    );
}
