#![cfg(test)]

use crate::{TicTacToeContract, TicTacToeContractClient};
use cougr_core::test::{GameHarness, PlayerSlot, Scenario};
use soroban_sdk::Env;

#[test]
fn sandbox_three_move_game_scenario() {
    let env = Env::default();
    let mut harness = GameHarness::new(env, TicTacToeContract);
    harness.mock_players(2);
    harness.mock_all_auths();

    let client = TicTacToeContractClient::new(harness.env(), harness.contract_id());
    let p_x = harness.player(PlayerSlot(0)).clone();
    let p_o = harness.player(PlayerSlot(1)).clone();

    // Initialize the game
    client.init_game(&p_x, &p_o);

    Scenario::new("three moves")
        .players(2)
        .turns(3)
        .run(&harness, |player_slot, turn, h| {
            let c = TicTacToeContractClient::new(h.env(), h.contract_id());
            let p = h.player(player_slot).clone();
            match turn.0 {
                0 => {
                    let res = c.make_move(&p, &0u32); // player X moves at 0
                    assert!(res.success);
                    assert_eq!(res.game_state.cells.get(0).unwrap(), 1);
                }
                1 => {
                    let res = c.make_move(&p, &4u32); // player O moves at 4
                    assert!(res.success);
                    assert_eq!(res.game_state.cells.get(4).unwrap(), 2);
                }
                2 => {
                    let res = c.make_move(&p, &1u32); // player X moves at 1
                    assert!(res.success);
                    assert_eq!(res.game_state.cells.get(1).unwrap(), 1);
                }
                _ => {}
            }
        });
}
