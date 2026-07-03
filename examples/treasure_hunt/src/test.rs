use super::*;
use cougr_core::privacy::stable::{to_on_chain_proof, MerkleTree, OnChainMerkleProof};
extern crate alloc;
use alloc::vec::Vec as StdVec;
use soroban_sdk::{testutils::Address as _, Address, Env};

const MAP_W: u32 = 3;
const MAP_H: u32 = 3;

struct OffchainMap {
    cell_values: StdVec<u8>,
    tree: MerkleTree,
}

fn setup() -> (Env, TreasureHuntContractClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register(TreasureHuntContract, ());
    let client = TreasureHuntContractClient::new(&env, &contract_id);
    let player = Address::generate(&env);
    (env, client, player)
}

fn encode_cell_leaf(x: u32, y: u32, cell_value: u8) -> [u8; 32] {
    let mut data = [0u8; 32];
    data[0..4].copy_from_slice(&x.to_be_bytes());
    data[4..8].copy_from_slice(&y.to_be_bytes());
    data[8] = cell_value;
    data
}

fn build_test_map(env: &Env) -> OffchainMap {
    let values: [u8; 9] = [
        0, 1, 2, //
        2, 0, 1, //
        0, 2, 0,
    ];
    let mut leaves: StdVec<[u8; 32]> = StdVec::new();
    let mut cell_values: StdVec<u8> = StdVec::new();

    for y in 0..MAP_H {
        for x in 0..MAP_W {
            let idx = (y * MAP_W + x) as usize;
            let value = values[idx];
            leaves.push(encode_cell_leaf(x, y, value));
            cell_values.push(value);
        }
    }

    let tree = MerkleTree::from_leaves(env, &leaves).unwrap();
    OffchainMap { cell_values, tree }
}

fn idx(x: u32, y: u32) -> u32 {
    y * MAP_W + x
}

fn proof_for(env: &Env, map: &OffchainMap, x: u32, y: u32) -> OnChainMerkleProof {
    let leaf_idx = idx(x, y);
    let proof = map.tree.proof(leaf_idx).unwrap();
    to_on_chain_proof(&proof, env)
}

fn value_at(map: &OffchainMap, x: u32, y: u32) -> u32 {
    map.cell_values[idx(x, y) as usize] as u32
}

#[test]
fn test_init_game_stores_roots_and_config() {
    let (env, client, player) = setup();
    env.mock_all_auths();
    let map = build_test_map(&env);
    let root = map.tree.root_bytes(&env);

    client.init_game(&player, &root, &MAP_W, &MAP_H, &2u32);

    let state = client.get_state();
    assert_eq!(state.player, player);
    assert_eq!(state.map_root.root, root);
    assert_eq!(state.map_root.width, MAP_W);
    assert_eq!(state.map_root.height, MAP_H);
    assert_eq!(state.map_root.total_treasures, 2);
    assert_eq!(state.player_state.health, DEFAULT_MAX_HEALTH);
    assert_eq!(state.game_config.status, GameStatus::Active);
    assert_eq!(state.explored_map.explored.len(), 0);
}

#[test]
fn test_valid_exploration_with_correct_proof() {
    let (env, client, player) = setup();
    env.mock_all_auths();
    let map = build_test_map(&env);
    let root = map.tree.root_bytes(&env);
    client.init_game(&player, &root, &MAP_W, &MAP_H, &99u32);

    let x = 1u32;
    let y = 0u32;
    let value = value_at(&map, x, y);
    let proof = proof_for(&env, &map, x, y);
    client.explore(&player, &x, &y, &value, &proof);

    let state = client.get_state();
    assert_eq!(state.player_state.x, x);
    assert_eq!(state.player_state.y, y);
    assert_eq!(state.player_state.score, DEFAULT_TREASURE_VALUE);
    assert_eq!(state.player_state.treasures_found, 1);
    assert!(client.is_explored(&x, &y));
}

#[test]
#[should_panic]
fn test_invalid_proof_rejected() {
    let (env, client, player) = setup();
    env.mock_all_auths();
    let map = build_test_map(&env);
    let root = map.tree.root_bytes(&env);
    client.init_game(&player, &root, &MAP_W, &MAP_H, &2u32);

    let x = 1u32;
    let y = 0u32;
    let good_value = value_at(&map, x, y);
    let bad_value = if good_value == CELL_TREASURE as u32 {
        CELL_TRAP as u32
    } else {
        CELL_TREASURE as u32
    };
    let proof = proof_for(&env, &map, x, y);
    client.explore(&player, &x, &y, &bad_value, &proof);
}

#[test]
fn test_trap_damage_and_loss_condition() {
    let (env, client, player) = setup();
    env.mock_all_auths();
    let map = build_test_map(&env);
    let root = map.tree.root_bytes(&env);
    client.init_game(&player, &root, &MAP_W, &MAP_H, &99u32);

    let moves = [
        (1u32, 0u32), // treasure
        (2u32, 0u32), // trap #1
        (2u32, 1u32), // treasure
        (1u32, 1u32), // empty
        (0u32, 1u32), // trap #2
        (0u32, 2u32), // empty
        (1u32, 2u32), // trap #3 => health reaches zero
    ];
    for (x, y) in moves {
        let state = client.get_state();
        if state.game_config.status != GameStatus::Active {
            break;
        }
        let value = value_at(&map, x, y);
        let proof = proof_for(&env, &map, x, y);
        client.explore(&player, &x, &y, &value, &proof);
    }

    let state = client.get_state();
    assert_eq!(state.player_state.health, 0);
    assert_eq!(state.game_config.status, GameStatus::Lost);
}

#[test]
#[should_panic]
fn test_reexplore_rejected() {
    let (env, client, player) = setup();
    env.mock_all_auths();
    let map = build_test_map(&env);
    let root = map.tree.root_bytes(&env);
    client.init_game(&player, &root, &MAP_W, &MAP_H, &2u32);

    let x = 1u32;
    let y = 0u32;
    let value = value_at(&map, x, y);
    let proof = proof_for(&env, &map, x, y);

    client.explore(&player, &x, &y, &value, &proof);
    client.explore(&player, &x, &y, &value, &proof);
}

#[test]
fn test_win_condition_all_treasures_found() {
    let (env, client, player) = setup();
    env.mock_all_auths();
    let map = build_test_map(&env);
    let root = map.tree.root_bytes(&env);
    client.init_game(&player, &root, &MAP_W, &MAP_H, &2u32);

    let path = [(1u32, 0u32), (2u32, 0u32), (2u32, 1u32)];
    for (x, y) in path {
        let value = value_at(&map, x, y);
        let proof = proof_for(&env, &map, x, y);
        client.explore(&player, &x, &y, &value, &proof);
    }

    let state = client.get_state();
    assert_eq!(state.player_state.treasures_found, 2);
    assert_eq!(state.game_config.status, GameStatus::Won);
}

#[test]
fn test_fog_of_war_sparse_root_updates() {
    let (env, client, player) = setup();
    env.mock_all_auths();
    let map = build_test_map(&env);
    let root = map.tree.root_bytes(&env);
    client.init_game(&player, &root, &MAP_W, &MAP_H, &2u32);

    let initial = client.get_state().fog_root;

    let x = 1u32;
    let y = 0u32;
    let value = value_at(&map, x, y);
    let proof = proof_for(&env, &map, x, y);
    client.explore(&player, &x, &y, &value, &proof);

    let after = client.get_state().fog_root;
    assert_ne!(initial, after);
}

#[test]
#[should_panic]
fn test_non_adjacent_move_rejected() {
    let (env, client, player) = setup();
    env.mock_all_auths();
    let map = build_test_map(&env);
    let root = map.tree.root_bytes(&env);
    client.init_game(&player, &root, &MAP_W, &MAP_H, &2u32);

    let x = 2u32;
    let y = 2u32;
    let value = value_at(&map, x, y);
    let proof = proof_for(&env, &map, x, y);
    client.explore(&player, &x, &y, &value, &proof);
}

#[test]
fn test_full_playable_sequence_to_win() {
    let (env, client, player) = setup();
    env.mock_all_auths();
    let map = build_test_map(&env);
    let root = map.tree.root_bytes(&env);
    client.init_game(&player, &root, &MAP_W, &MAP_H, &2u32);

    let sequence = [(1u32, 0u32), (2u32, 0u32), (2u32, 1u32)];
    for (x, y) in sequence {
        let value = value_at(&map, x, y);
        let proof = proof_for(&env, &map, x, y);
        client.explore(&player, &x, &y, &value, &proof);
    }

    let state = client.get_state();
    assert_eq!(state.game_config.status, GameStatus::Won);
    assert_eq!(state.player_state.health, DEFAULT_MAX_HEALTH - 1);
    assert_eq!(state.player_state.score, DEFAULT_TREASURE_VALUE * 2);
}

#[test]
fn test_commit_phase() {
    // Tests that the game initializes in the committed state
    let (env, client, player) = setup();
    env.mock_all_auths();
    let map = build_test_map(&env);
    let root = map.tree.root_bytes(&env);

    client.init_game(&player, &root, &MAP_W, &MAP_H, &2u32);

    let state = client.get_state();
    assert_eq!(state.map_root.root, root);
    assert_eq!(state.game_config.status, GameStatus::Active);
}

#[test]
fn test_reveal_phase_with_valid_proof() {
    // Tests the reveal/exploration phase with correct Merkle proof
    let (env, client, player) = setup();
    env.mock_all_auths();
    let map = build_test_map(&env);
    let root = map.tree.root_bytes(&env);
    client.init_game(&player, &root, &MAP_W, &MAP_H, &99u32);

    let x = 1u32;
    let y = 0u32;
    let value = value_at(&map, x, y);
    let proof = proof_for(&env, &map, x, y);
    client.explore(&player, &x, &y, &value, &proof);

    let state = client.get_state();
    assert_eq!(state.player_state.x, x);
    assert_eq!(state.player_state.y, y);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_invalid_reveal_value_rejected() {
    // Tests that providing an incorrect cell value during reveal is rejected
    let (env, client, player) = setup();
    env.mock_all_auths();
    let map = build_test_map(&env);
    let root = map.tree.root_bytes(&env);
    client.init_game(&player, &root, &MAP_W, &MAP_H, &2u32);

    let x = 1u32;
    let y = 0u32;
    let proof = proof_for(&env, &map, x, y);
    // Claim wrong value
    client.explore(&player, &x, &y, &99, &proof);
}
