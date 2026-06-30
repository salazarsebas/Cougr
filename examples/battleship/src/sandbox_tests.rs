#![cfg(test)]

use crate::{BattleshipContract, BattleshipContractClient, Phase};
use cougr_core::test::{GameHarness, Scenario, PlayerSlot};
use cougr_core::privacy::stable::{to_on_chain_proof, MerkleTree};
use soroban_sdk::{Bytes, BytesN, Env};

fn build_merkle_tree(env: &Env, board: &[u32; 100]) -> (BytesN<32>, MerkleTree) {
    let mut leaves = [[0u8; 32]; 100];
    for (idx, &value) in board.iter().enumerate() {
        let mut data = Bytes::new(env);
        data.append(&Bytes::from_array(env, &(idx as u32).to_be_bytes()));
        data.append(&Bytes::from_array(env, &value.to_be_bytes()));
        let leaf: BytesN<32> = env.crypto().sha256(&data).into();
        leaves[idx] = leaf.to_array();
    }
    let tree = MerkleTree::from_leaves(env, &leaves).unwrap();
    (tree.root_bytes(env), tree)
}

fn make_commitment(env: &Env, board: &[u32; 100], salt: &BytesN<32>) -> BytesN<32> {
    let mut data = Bytes::new(env);
    for &cell in board.iter() {
        data.append(&Bytes::from_array(env, &cell.to_be_bytes()));
    }
    for i in 0..32 {
        data.push_back(salt.get(i).unwrap());
    }
    env.crypto().sha256(&data).into()
}

#[test]
fn sandbox_battleship_gameplay_scenario() {
    let env = Env::default();
    let mut harness = GameHarness::new(env, BattleshipContract);
    harness.mock_players(2);
    harness.mock_all_auths();

    let client = BattleshipContractClient::new(harness.env(), harness.contract_id());
    let p_a = harness.player(PlayerSlot(0)).clone();
    let p_b = harness.player(PlayerSlot(1)).clone();

    client.new_game(&p_a, &p_b);

    // Setup boards and commitments
    let board_a = [0u32; 100];
    let mut board_b = [0u32; 100];
    board_b[10] = 1; // Ship at (1, 0) (index 10)

    let salt_a = BytesN::from_array(harness.env(), &[1u8; 32]);
    let salt_b = BytesN::from_array(harness.env(), &[2u8; 32]);

    let commitment_a = make_commitment(harness.env(), &board_a, &salt_a);
    let (root_a, _) = build_merkle_tree(harness.env(), &board_a);

    let commitment_b = make_commitment(harness.env(), &board_b, &salt_b);
    let (root_b, tree_b) = build_merkle_tree(harness.env(), &board_b);

    Scenario::new("battleship deployment and attack")
        .players(2)
        .turns(4)
        .run(&harness, |player_slot, turn, h| {
            let c = BattleshipContractClient::new(h.env(), h.contract_id());
            let p = h.player(player_slot).clone();
            match turn.0 {
                0 => {
                    // Player A commits
                    c.commit_board(&p, &commitment_a, &root_a);
                    assert_eq!(c.get_state().turn_state.phase, Phase::Setup);
                }
                1 => {
                    // Player B commits -> transitions to Attack phase
                    c.commit_board(&p, &commitment_b, &root_b);
                    assert_eq!(c.get_state().turn_state.phase, Phase::Attack);
                }
                2 => {
                    // Player A attacks cell (0, 1) (index 10)
                    c.attack(&p, &0, &1);
                    let state = c.get_state();
                    assert!(state.turn_state.has_pending);
                    assert_eq!(state.turn_state.pending_reveal_x, 0);
                    assert_eq!(state.turn_state.pending_reveal_y, 1);
                }
                3 => {
                    // Player B reveals cell (0, 1)
                    let proof = to_on_chain_proof(&tree_b.proof(10).unwrap(), h.env());
                    c.reveal_cell(&p, &0, &1, &1, &proof);
                    let state = c.get_state();
                    assert!(!state.turn_state.has_pending);
                    // Check hit recorded
                    let cell = state.attack_grid_b.cells.get(10).unwrap();
                    assert_eq!(cell, crate::CellResult::Hit);
                }
                _ => {}
            }
        });
}
