use super::*;
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal,
};

fn setup_game() -> (Env, TicTacToeContractClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TicTacToeContract, ());
    let client = TicTacToeContractClient::new(&env, &contract_id);

    let player_x = Address::generate(&env);
    let player_o = Address::generate(&env);

    client.init_game(&player_x, &player_o);

    (env, client, player_x, player_o)
}

#[test]
fn test_init_game() {
    let env = Env::default();
    let contract_id = env.register(TicTacToeContract, ());
    let client = TicTacToeContractClient::new(&env, &contract_id);

    let player_x = Address::generate(&env);
    let player_o = Address::generate(&env);

    let game_state = client.init_game(&player_x, &player_o);

    for i in 0..9 {
        assert_eq!(game_state.cells.get(i).unwrap(), 0);
    }
    assert_eq!(game_state.player_x, player_x);
    assert_eq!(game_state.player_o, player_o);
    assert!(game_state.is_x_turn);
    assert_eq!(game_state.move_count, 0);
    assert_eq!(game_state.status, 0);
}

#[test]
fn test_get_state() {
    let (_, client, player_x, player_o) = setup_game();

    let state = client.get_state();

    assert_eq!(state.player_x, player_x);
    assert_eq!(state.player_o, player_o);
    assert!(state.is_x_turn);
    assert_eq!(state.move_count, 0);
}

#[test]
fn test_valid_move_x() {
    let (_, client, player_x, _) = setup_game();

    let result = client.make_move(&player_x, &0);

    assert!(result.success);
    assert_eq!(result.game_state.cells.get(0).unwrap(), 1);
    assert!(!result.game_state.is_x_turn);
    assert_eq!(result.game_state.move_count, 1);
}

#[test]
fn test_valid_move_o() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &0);
    let result = client.make_move(&player_o, &4);

    assert!(result.success);
    assert_eq!(result.game_state.cells.get(4).unwrap(), 2);
    assert!(result.game_state.is_x_turn);
    assert_eq!(result.game_state.move_count, 2);
}

#[test]
fn test_all_positions_initially_valid() {
    let (_, client, _, _) = setup_game();

    // All positions should be valid at game start
    for i in 0..9u32 {
        assert!(client.is_valid_move(&i));
    }

    // Position 9+ should be invalid
    assert!(!client.is_valid_move(&9));
    assert!(!client.is_valid_move(&10));
}

#[test]
fn test_is_valid_move() {
    let (_, client, player_x, _) = setup_game();

    for i in 0..9u32 {
        assert!(client.is_valid_move(&i));
    }

    client.make_move(&player_x, &4);

    assert!(!client.is_valid_move(&4));
    assert!(client.is_valid_move(&0));
    assert!(client.is_valid_move(&8));
}

#[test]
fn test_invalid_position() {
    let (_, client, player_x, _) = setup_game();

    let result = client.make_move(&player_x, &9);

    assert!(!result.success);
    assert_eq!(result.message, symbol_short!("invalid"));
}

#[test]
fn test_occupied_cell() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &0);
    let result = client.make_move(&player_o, &0);

    assert!(!result.success);
    assert_eq!(result.message, symbol_short!("occupied"));
}

#[test]
fn test_wrong_turn_o_moves_first() {
    let (_, client, _, player_o) = setup_game();

    let result = client.make_move(&player_o, &0);

    assert!(!result.success);
    assert_eq!(result.message, symbol_short!("notturn"));
}

#[test]
fn test_wrong_turn_x_moves_twice() {
    let (_, client, player_x, _) = setup_game();

    client.make_move(&player_x, &0);
    let result = client.make_move(&player_x, &1);

    assert!(!result.success);
    assert_eq!(result.message, symbol_short!("notturn"));
}

#[test]
fn test_non_player_cannot_move() {
    let (env, client, _, _) = setup_game();

    let random_player = Address::generate(&env);
    let result = client.make_move(&random_player, &0);

    assert!(!result.success);
    assert_eq!(result.message, symbol_short!("notplay"));
}

#[test]
fn test_x_wins_row_top() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &0);
    client.make_move(&player_o, &3);
    client.make_move(&player_x, &1);
    client.make_move(&player_o, &4);
    let result = client.make_move(&player_x, &2);

    assert!(result.success);
    assert_eq!(result.game_state.status, 1);
}

#[test]
fn test_x_wins_row_middle() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &3);
    client.make_move(&player_o, &0);
    client.make_move(&player_x, &4);
    client.make_move(&player_o, &1);
    let result = client.make_move(&player_x, &5);

    assert!(result.success);
    assert_eq!(result.game_state.status, 1);
}

#[test]
fn test_x_wins_row_bottom() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &6);
    client.make_move(&player_o, &0);
    client.make_move(&player_x, &7);
    client.make_move(&player_o, &1);
    let result = client.make_move(&player_x, &8);

    assert!(result.success);
    assert_eq!(result.game_state.status, 1);
}

#[test]
fn test_x_wins_column_left() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &0);
    client.make_move(&player_o, &1);
    client.make_move(&player_x, &3);
    client.make_move(&player_o, &2);
    let result = client.make_move(&player_x, &6);

    assert!(result.success);
    assert_eq!(result.game_state.status, 1);
}

#[test]
fn test_x_wins_column_middle() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &1);
    client.make_move(&player_o, &0);
    client.make_move(&player_x, &4);
    client.make_move(&player_o, &2);
    let result = client.make_move(&player_x, &7);

    assert!(result.success);
    assert_eq!(result.game_state.status, 1);
}

#[test]
fn test_x_wins_column_right() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &2);
    client.make_move(&player_o, &0);
    client.make_move(&player_x, &5);
    client.make_move(&player_o, &1);
    let result = client.make_move(&player_x, &8);

    assert!(result.success);
    assert_eq!(result.game_state.status, 1);
}

#[test]
fn test_x_wins_diagonal_main() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &0);
    client.make_move(&player_o, &1);
    client.make_move(&player_x, &4);
    client.make_move(&player_o, &2);
    let result = client.make_move(&player_x, &8);

    assert!(result.success);
    assert_eq!(result.game_state.status, 1);
}

#[test]
fn test_x_wins_diagonal_anti() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &2);
    client.make_move(&player_o, &0);
    client.make_move(&player_x, &4);
    client.make_move(&player_o, &1);
    let result = client.make_move(&player_x, &6);

    assert!(result.success);
    assert_eq!(result.game_state.status, 1);
}

#[test]
fn test_o_wins_row() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &0);
    client.make_move(&player_o, &3);
    client.make_move(&player_x, &1);
    client.make_move(&player_o, &4);
    client.make_move(&player_x, &8);
    let result = client.make_move(&player_o, &5);

    assert!(result.success);
    assert_eq!(result.game_state.status, 2);
}

#[test]
fn test_o_wins_column() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &0);
    client.make_move(&player_o, &1);
    client.make_move(&player_x, &2);
    client.make_move(&player_o, &4);
    client.make_move(&player_x, &3);
    let result = client.make_move(&player_o, &7);

    assert!(result.success);
    assert_eq!(result.game_state.status, 2);
}

#[test]
fn test_o_wins_diagonal() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &1);
    client.make_move(&player_o, &0);
    client.make_move(&player_x, &2);
    client.make_move(&player_o, &4);
    client.make_move(&player_x, &5);
    let result = client.make_move(&player_o, &8);

    assert!(result.success);
    assert_eq!(result.game_state.status, 2);
}

#[test]
fn test_draw() {
    let (_, client, player_x, player_o) = setup_game();

    // X | O | X
    // X | X | O
    // O | X | O
    client.make_move(&player_x, &0);
    client.make_move(&player_o, &1);
    client.make_move(&player_x, &2);
    client.make_move(&player_o, &5);
    client.make_move(&player_x, &3);
    client.make_move(&player_o, &6);
    client.make_move(&player_x, &4);
    client.make_move(&player_o, &8);
    let result = client.make_move(&player_x, &7);

    assert!(result.success);
    assert_eq!(result.game_state.status, 3);
    assert_eq!(result.game_state.move_count, 9);
}

#[test]
fn test_no_moves_after_win() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &0);
    client.make_move(&player_o, &3);
    client.make_move(&player_x, &1);
    client.make_move(&player_o, &4);
    client.make_move(&player_x, &2);

    let result = client.make_move(&player_o, &5);

    assert!(!result.success);
    assert_eq!(result.message, symbol_short!("gameover"));
}

#[test]
fn test_no_moves_after_draw() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &0);
    client.make_move(&player_o, &1);
    client.make_move(&player_x, &2);
    client.make_move(&player_o, &5);
    client.make_move(&player_x, &3);
    client.make_move(&player_o, &6);
    client.make_move(&player_x, &4);
    client.make_move(&player_o, &8);
    client.make_move(&player_x, &7);

    let state = client.get_state();
    assert_eq!(state.status, 3);

    for i in 0..9u32 {
        assert!(!client.is_valid_move(&i));
    }
}

#[test]
fn test_get_winner_x() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &0);
    client.make_move(&player_o, &3);
    client.make_move(&player_x, &1);
    client.make_move(&player_o, &4);
    client.make_move(&player_x, &2);

    let winner = client.get_winner();
    assert_eq!(winner, Some(player_x));
}

#[test]
fn test_get_winner_o() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &0);
    client.make_move(&player_o, &3);
    client.make_move(&player_x, &1);
    client.make_move(&player_o, &4);
    client.make_move(&player_x, &8);
    client.make_move(&player_o, &5);

    let winner = client.get_winner();
    assert_eq!(winner, Some(player_o));
}

#[test]
fn test_get_winner_none_in_progress() {
    let (_, client, _, _) = setup_game();

    let winner = client.get_winner();
    assert_eq!(winner, None);
}

#[test]
fn test_get_winner_none_draw() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &0);
    client.make_move(&player_o, &1);
    client.make_move(&player_x, &2);
    client.make_move(&player_o, &5);
    client.make_move(&player_x, &3);
    client.make_move(&player_o, &6);
    client.make_move(&player_x, &4);
    client.make_move(&player_o, &8);
    client.make_move(&player_x, &7);

    let winner = client.get_winner();
    assert_eq!(winner, None);
}

#[test]
fn test_reset_game() {
    let (env, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &0);
    client.make_move(&player_o, &4);
    client.make_move(&player_x, &8);

    let reset_state = client.reset_game(&player_x);

    for i in 0..9 {
        assert_eq!(reset_state.cells.get(i).unwrap(), 0);
    }
    assert_eq!(reset_state.player_x, player_x);
    assert_eq!(reset_state.player_o, player_o);
    assert!(reset_state.is_x_turn);
    assert_eq!(reset_state.move_count, 0);
    assert_eq!(reset_state.status, 0);
    assert!(
        env.auths().iter().any(|(address, _)| address == &player_x),
        "Expected player_x in recorded authorizations"
    );
}

#[test]
fn test_reset_after_win() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &0);
    client.make_move(&player_o, &3);
    client.make_move(&player_x, &1);
    client.make_move(&player_o, &4);
    client.make_move(&player_x, &2);

    let state_before_reset = client.get_state();
    assert_eq!(state_before_reset.status, 1);

    let reset_state = client.reset_game(&player_x);
    assert_eq!(reset_state.status, 0);
    assert_eq!(reset_state.move_count, 0);

    let result = client.make_move(&player_x, &4);
    assert!(result.success);
}

#[test]
fn test_state_persistence() {
    let (_, client, player_x, player_o) = setup_game();

    client.make_move(&player_x, &0);
    client.make_move(&player_o, &4);
    client.make_move(&player_x, &8);

    let state = client.get_state();

    assert_eq!(state.cells.get(0).unwrap(), 1);
    assert_eq!(state.cells.get(4).unwrap(), 2);
    assert_eq!(state.cells.get(8).unwrap(), 1);
    assert_eq!(state.move_count, 3);
    assert!(!state.is_x_turn);
}

#[test]
fn test_move_count_increments() {
    let (_, client, player_x, player_o) = setup_game();

    let initial_state = client.get_state();
    assert_eq!(initial_state.move_count, 0);

    client.make_move(&player_x, &0);
    let state1 = client.get_state();
    assert_eq!(state1.move_count, 1);

    client.make_move(&player_o, &1);
    let state2 = client.get_state();
    assert_eq!(state2.move_count, 2);

    client.make_move(&player_x, &2);
    let state3 = client.get_state();
    assert_eq!(state3.move_count, 3);
}

#[test]
fn test_make_move_auth_names_player_x() {
    let env = Env::default();
    let contract_id = env.register(TicTacToeContract, ());
    let client = TicTacToeContractClient::new(&env, &contract_id);
    let player_x = Address::generate(&env);
    let player_o = Address::generate(&env);

    client.init_game(&player_x, &player_o);

    env.mock_auths(&[MockAuth {
        address: &player_x,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "make_move",
            args: (&player_x, 0u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let result = client.make_move(&player_x, &0);
    assert!(result.success);
    assert!(
        env.auths().iter().any(|(address, _)| address == &player_x),
        "Expected player_x in recorded authorizations"
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_player_o_cannot_authorize_player_x_move() {
    let env = Env::default();
    let contract_id = env.register(TicTacToeContract, ());
    let client = TicTacToeContractClient::new(&env, &contract_id);
    let player_x = Address::generate(&env);
    let player_o = Address::generate(&env);

    client.init_game(&player_x, &player_o);

    env.mock_auths(&[MockAuth {
        address: &player_o,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "make_move",
            args: (&player_x, 0u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.make_move(&player_x, &0);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_stranger_cannot_authorize_player_x_move() {
    let env = Env::default();
    let contract_id = env.register(TicTacToeContract, ());
    let client = TicTacToeContractClient::new(&env, &contract_id);
    let player_x = Address::generate(&env);
    let player_o = Address::generate(&env);
    let stranger = Address::generate(&env);

    client.init_game(&player_x, &player_o);

    env.mock_auths(&[MockAuth {
        address: &stranger,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "make_move",
            args: (&player_x, 0u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.make_move(&player_x, &0);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_stranger_cannot_authorize_reset_game() {
    let env = Env::default();
    let contract_id = env.register(TicTacToeContract, ());
    let client = TicTacToeContractClient::new(&env, &contract_id);
    let player_x = Address::generate(&env);
    let player_o = Address::generate(&env);
    let stranger = Address::generate(&env);

    client.init_game(&player_x, &player_o);

    env.mock_auths(&[MockAuth {
        address: &stranger,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "reset_game",
            args: (&player_x,).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.reset_game(&player_x);
}

#[test]
#[should_panic]
fn test_make_move_requires_auth() {
    let env = Env::default();
    let contract_id = env.register(TicTacToeContract, ());
    let client = TicTacToeContractClient::new(&env, &contract_id);
    let player_x = Address::generate(&env);
    let player_o = Address::generate(&env);

    client.init_game(&player_x, &player_o);
    client.make_move(&player_x, &0);
}

#[test]
fn test_wrong_player_cannot_move() {
    let (_, client, _, player_o) = setup_game();

    let result = client.make_move(&player_o, &0);

    assert!(!result.success);
    assert_eq!(result.message, symbol_short!("notturn"));
}

#[test]
#[should_panic(expected = "gameinprogress")]
fn test_init_game_cannot_overwrite_live_match() {
    let (env, client, player_x, _) = setup_game();

    client.make_move(&player_x, &0);

    let new_x = Address::generate(&env);
    let new_o = Address::generate(&env);
    client.init_game(&new_x, &new_o);
}

#[test]
#[should_panic(expected = "notplay")]
fn test_non_player_cannot_reset_game() {
    let (env, client, _, _) = setup_game();
    let stranger = Address::generate(&env);

    client.reset_game(&stranger);
}
