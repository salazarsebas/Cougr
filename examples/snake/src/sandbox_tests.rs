#![cfg(test)]

use crate::{SnakeContract, SnakeContractClient};
use cougr_core::test::{GameHarness, Scenario};
use soroban_sdk::Env;

#[test]
fn sandbox_snake_movement_scenario() {
    let env = Env::default();
    let harness = GameHarness::new(env, SnakeContract);
    let client = SnakeContractClient::new(harness.env(), harness.contract_id());

    // Initialize game
    client.init_game();

    let (x1, y1) = client.get_head_pos();

    Scenario::new("three ticks")
        .turns(3)
        .run(&harness, |_player, turn, h| {
            let c = SnakeContractClient::new(h.env(), h.contract_id());
            match turn.0 {
                0 => {
                    c.update_tick();
                    let (x, y) = c.get_head_pos();
                    // Moves right by default
                    assert_eq!((x, y), (x1 + 1, y1));
                }
                1 => {
                    // Change direction to Up (0)
                    assert!(c.change_direction(&0));
                    c.update_tick();
                    let (x, y) = c.get_head_pos();
                    assert_eq!((x, y), (x1 + 1, y1 - 1));
                }
                2 => {
                    // Change direction to Left (2)
                    assert!(c.change_direction(&2));
                    c.update_tick();
                    let (x, y) = c.get_head_pos();
                    assert_eq!((x, y), (x1, y1 - 1));
                }
                _ => {}
            }
        });
}
