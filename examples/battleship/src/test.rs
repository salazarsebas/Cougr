use super::*;
use cougr_core::component::ComponentTrait;
use cougr_core::privacy::stable::{to_on_chain_proof, MerkleTree, OnChainMerkleProof};
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, Vec};

fn setup_game() -> (Env, BattleshipContractClient<'static>, Address, Address) {
    let env = Env::default();
    let contract_id = env.register(BattleshipContract, ());
    let client = BattleshipContractClient::new(&env, &contract_id);

    let player_a = Address::generate(&env);
    let player_b = Address::generate(&env);

    (env, client, player_a, player_b)
}

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

fn get_merkle_proof(env: &Env, tree: &MerkleTree, index: u32) -> OnChainMerkleProof {
    to_on_chain_proof(&tree.proof(index).unwrap(), env)
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
fn test_new_game() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    let state = client.get_state();
    assert_eq!(state.player_a, player_a);
    assert_eq!(state.player_b, player_b);
    assert_eq!(state.turn_state.phase, Phase::Setup);
}

#[test]
fn test_commit_board() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    let mut board_a = [0u32; 100];
    board_a[0] = 1; // Ship at (0,0)
    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let commitment_a = make_commitment(&env, &board_a, &salt_a);
    let (root_a, _) = build_merkle_tree(&env, &board_a);

    client.commit_board(&player_a, &commitment_a, &root_a);

    let state = client.get_state();
    assert_eq!(state.turn_state.phase, Phase::Setup);

    let mut board_b = [0u32; 100];
    board_b[1] = 1;
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);
    let commitment_b = make_commitment(&env, &board_b, &salt_b);
    let (root_b, _) = build_merkle_tree(&env, &board_b);

    client.commit_board(&player_b, &commitment_b, &root_b);

    let state = client.get_state();
    assert_eq!(state.turn_state.phase, Phase::Attack);
}

#[test]
fn test_attack_and_reveal_miss() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    // Setup boards
    let board_a = [0u32; 100];
    let mut board_b = [0u32; 100];
    board_b[10] = 1; // Ship at (0,1)

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let commitment_a = make_commitment(&env, &board_a, &salt_a);
    let commitment_b = make_commitment(&env, &board_b, &salt_b);

    let (root_a, _tree_a) = build_merkle_tree(&env, &board_a);
    let (root_b, tree_b) = build_merkle_tree(&env, &board_b);

    client.commit_board(&player_a, &commitment_a, &root_a);
    client.commit_board(&player_b, &commitment_b, &root_b);

    // Player A attacks (0,0) on B's board - miss
    client.attack(&player_a, &0, &0);

    let state = client.get_state();
    assert!(state.turn_state.has_pending);

    // Player B reveals
    let proof = get_merkle_proof(&env, &tree_b, 0);
    client.reveal_cell(&player_b, &0, &0, &0, &proof);

    let state = client.get_state();
    assert!(!state.turn_state.has_pending);
    assert_eq!(state.attack_grid_b.cells.get(0).unwrap(), CellResult::Miss);
}

#[test]
fn test_attack_and_reveal_hit() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    let board_a = [0u32; 100];
    let mut board_b = [0u32; 100];
    board_b[0] = 1; // Ship at (0,0)

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let commitment_a = make_commitment(&env, &board_a, &salt_a);
    let commitment_b = make_commitment(&env, &board_b, &salt_b);

    let (root_a, _) = build_merkle_tree(&env, &board_a);
    let (root_b, tree_b) = build_merkle_tree(&env, &board_b);

    client.commit_board(&player_a, &commitment_a, &root_a);
    client.commit_board(&player_b, &commitment_b, &root_b);

    // Player A attacks (0,0) - hit
    client.attack(&player_a, &0, &0);

    let proof = get_merkle_proof(&env, &tree_b, 0);
    client.reveal_cell(&player_b, &0, &0, &1, &proof);

    let state = client.get_state();
    assert_eq!(state.attack_grid_b.cells.get(0).unwrap(), CellResult::Hit);
    assert_eq!(state.ship_status.remaining_b, 16);
}

#[test]
#[should_panic(expected = "Invalid proof")]
fn test_invalid_proof_rejected() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    let board_a = [0u32; 100];
    let mut board_b = [0u32; 100];
    board_b[0] = 1;

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let commitment_a = make_commitment(&env, &board_a, &salt_a);
    let commitment_b = make_commitment(&env, &board_b, &salt_b);

    let (root_a, _) = build_merkle_tree(&env, &board_a);
    let (root_b, tree_b) = build_merkle_tree(&env, &board_b);

    client.commit_board(&player_a, &commitment_a, &root_a);
    client.commit_board(&player_b, &commitment_b, &root_b);

    client.attack(&player_a, &0, &0);

    // Try to reveal with wrong value (claim miss when it's hit)
    let proof = get_merkle_proof(&env, &tree_b, 0);
    client.reveal_cell(&player_b, &0, &0, &0, &proof); // Wrong value
}

#[test]
#[should_panic(expected = "Invalid proof")]
fn test_relabelled_proof_from_spare_slot_rejected() {
    // A defender commits a 128-leaf tree (100 real cells plus the 28 spare
    // padding slots every board's tree has). Spare slot 100 is set to the
    // leaf a genuine "coord 0, value 0" (miss) cell would produce, even
    // though coord 0 is a real ship. The defender then reveals coord 0 with
    // the real proof for slot 100, relabelled with `leaf_index = 0`. Before
    // the local depth/path_bits check, this verified: the hash chain
    // legitimately walks slot 100 up to the real root, and nothing checked
    // that `leaf_index` agreed with the position `path_bits` actually walks.
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    let board_a = [0u32; 100];
    let mut board_b = [0u32; 100];
    board_b[0] = 1; // real ship at coord 0

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);
    let commitment_a = make_commitment(&env, &board_a, &salt_a);
    let commitment_b = make_commitment(&env, &board_b, &salt_b);
    let (root_a, _) = build_merkle_tree(&env, &board_a);

    let mut cheat_leaves = [[0u8; 32]; 128];
    for idx in 0..100u32 {
        let value = board_b[idx as usize];
        let mut data = Bytes::new(&env);
        data.append(&Bytes::from_array(&env, &idx.to_be_bytes()));
        data.append(&Bytes::from_array(&env, &value.to_be_bytes()));
        let leaf: BytesN<32> = env.crypto().sha256(&data).into();
        cheat_leaves[idx as usize] = leaf.to_array();
    }
    // Spare slot 100: deliberately the leaf a genuine (coord 0, value 0)
    // cell would produce.
    let mut data = Bytes::new(&env);
    data.append(&Bytes::from_array(&env, &0u32.to_be_bytes()));
    data.append(&Bytes::from_array(&env, &0u32.to_be_bytes()));
    let spare_leaf: BytesN<32> = env.crypto().sha256(&data).into();
    cheat_leaves[100] = spare_leaf.to_array();
    for (idx, leaf_slot) in cheat_leaves.iter_mut().enumerate().skip(101) {
        let mut data = Bytes::new(&env);
        data.append(&Bytes::from_array(&env, &(idx as u32).to_be_bytes()));
        data.append(&Bytes::from_array(&env, &0u32.to_be_bytes()));
        let leaf: BytesN<32> = env.crypto().sha256(&data).into();
        *leaf_slot = leaf.to_array();
    }

    let cheat_tree = MerkleTree::from_leaves(&env, &cheat_leaves).unwrap();
    let cheat_root = cheat_tree.root_bytes(&env);

    client.commit_board(&player_a, &commitment_a, &root_a);
    client.commit_board(&player_b, &commitment_b, &cheat_root);

    client.attack(&player_a, &0, &0);

    // Real proof for the spare slot, relabelled to claim coord 0.
    let mut forged_proof = to_on_chain_proof(&cheat_tree.proof(100).unwrap(), &env);
    forged_proof.leaf_index = 0;

    client.reveal_cell(&player_b, &0, &0, &0, &forged_proof);
}

#[test]
#[should_panic(expected = "Already attacked")]
fn test_cannot_attack_same_cell_twice() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    let board_a = [0u32; 100];
    let board_b = [0u32; 100];

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let commitment_a = make_commitment(&env, &board_a, &salt_a);
    let commitment_b = make_commitment(&env, &board_b, &salt_b);

    let (root_a, tree_a) = build_merkle_tree(&env, &board_a);
    let (root_b, tree_b) = build_merkle_tree(&env, &board_b);

    client.commit_board(&player_a, &commitment_a, &root_a);
    client.commit_board(&player_b, &commitment_b, &root_b);

    // A attacks (0,0) on B's board
    client.attack(&player_a, &0, &0);
    let proof = get_merkle_proof(&env, &tree_b, 0);
    client.reveal_cell(&player_b, &0, &0, &0, &proof);

    // B attacks (1,1) on A's board
    client.attack(&player_b, &1, &1);
    let proof = get_merkle_proof(&env, &tree_a, 11);
    client.reveal_cell(&player_a, &1, &1, &0, &proof);

    // A tries to attack (0,0) on B's board again - should panic
    client.attack(&player_a, &0, &0);
}

#[test]
#[should_panic(expected = "Not your turn")]
fn test_turn_enforcement() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    let board_a = [0u32; 100];
    let board_b = [0u32; 100];

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let commitment_a = make_commitment(&env, &board_a, &salt_a);
    let commitment_b = make_commitment(&env, &board_b, &salt_b);

    let (root_a, _) = build_merkle_tree(&env, &board_a);
    let (root_b, _) = build_merkle_tree(&env, &board_b);

    client.commit_board(&player_a, &commitment_a, &root_a);
    client.commit_board(&player_b, &commitment_b, &root_b);

    // Player B tries to attack when it's A's turn
    client.attack(&player_b, &0, &0);
}

#[test]
fn test_win_condition() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    let board_a = [0u32; 100];
    let mut board_b = [0u32; 100];
    board_b[0] = 1; // Only one ship cell

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let commitment_a = make_commitment(&env, &board_a, &salt_a);
    let commitment_b = make_commitment(&env, &board_b, &salt_b);

    let (root_a, _) = build_merkle_tree(&env, &board_a);
    let (root_b, tree_b) = build_merkle_tree(&env, &board_b);

    client.commit_board(&player_a, &commitment_a, &root_a);
    client.commit_board(&player_b, &commitment_b, &root_b);

    // Manually set remaining to 1 for quick test
    let mut state = client.get_state();
    state.ship_status.remaining_b = 1;
    env.as_contract(&client.address, || {
        env.storage().instance().set(&GAME_KEY, &state);
    });

    // Attack and sink last ship
    client.attack(&player_a, &0, &0);
    let proof = get_merkle_proof(&env, &tree_b, 0);
    client.reveal_cell(&player_b, &0, &0, &1, &proof);

    let state = client.get_state();
    assert_eq!(state.turn_state.phase, Phase::Finished);
    assert_eq!(state.winner, Some(player_a));
}

#[test]
fn test_component_trait_via_macro() {
    let env = Env::default();

    let commitment = BoardCommitment {
        commitment: BytesN::from_array(&env, &[0u8; 32]),
        merkle_root: BytesN::from_array(&env, &[1u8; 32]),
    };

    let serialized = commitment.serialize(&env);
    assert_eq!(serialized.len(), 64);
    assert_eq!(BoardCommitment::component_type(), symbol_short!("board"));

    // Test round-trip deserialize
    let deserialized = BoardCommitment::deserialize(&env, &serialized).unwrap();
    assert_eq!(deserialized.commitment, commitment.commitment);
    assert_eq!(deserialized.merkle_root, commitment.merkle_root);
}

#[test]
fn test_turn_switching() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    let board_a = [0u32; 100];
    let board_b = [0u32; 100];

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let commitment_a = make_commitment(&env, &board_a, &salt_a);
    let commitment_b = make_commitment(&env, &board_b, &salt_b);

    let (root_a, tree_a) = build_merkle_tree(&env, &board_a);
    let (root_b, tree_b) = build_merkle_tree(&env, &board_b);

    client.commit_board(&player_a, &commitment_a, &root_a);
    client.commit_board(&player_b, &commitment_b, &root_b);

    let state = client.get_state();
    assert_eq!(state.turn_state.current_player, player_a);

    // A attacks
    client.attack(&player_a, &0, &0);
    let proof = get_merkle_proof(&env, &tree_b, 0);
    client.reveal_cell(&player_b, &0, &0, &0, &proof);

    let state = client.get_state();
    assert_eq!(state.turn_state.current_player, player_b);

    // B attacks
    client.attack(&player_b, &1, &1);
    let proof = get_merkle_proof(&env, &tree_a, 11);
    client.reveal_cell(&player_a, &1, &1, &0, &proof);

    let state = client.get_state();
    assert_eq!(state.turn_state.current_player, player_a);
}

#[test]
#[should_panic(expected = "No pending attack")]
fn test_reveal_without_pending_attack() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    let board_a = [0u32; 100];
    let board_b = [0u32; 100];

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let commitment_a = make_commitment(&env, &board_a, &salt_a);
    let commitment_b = make_commitment(&env, &board_b, &salt_b);

    let (root_a, _) = build_merkle_tree(&env, &board_a);
    let (root_b, _) = build_merkle_tree(&env, &board_b);

    client.commit_board(&player_a, &commitment_a, &root_a);
    client.commit_board(&player_b, &commitment_b, &root_b);

    // Try to reveal without attacking first
    let fake_proof = OnChainMerkleProof {
        siblings: Vec::new(&env),
        path_bits: 0,
        leaf: BytesN::from_array(&env, &[0u8; 32]),
        leaf_index: 0,
        depth: 0,
    };
    client.reveal_cell(&player_b, &0, &0, &0, &fake_proof);
}

#[test]
#[should_panic(expected = "Not in attack phase")]
fn test_attack_before_commit() {
    let (env, client, player_a, _player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &Address::generate(&env));

    // Try to attack before boards are committed
    client.attack(&player_a, &0, &0);
}

#[test]
fn test_new_game_requires_both_participants_auth() {
    use soroban_sdk::{
        testutils::{MockAuth, MockAuthInvoke},
        IntoVal,
    };

    let (env, client, player_a, player_b) = setup_game();
    let outsider = Address::generate(&env);

    // Unauthenticated call must fail
    assert!(client.try_new_game(&player_a, &player_b).is_err());

    // Non-participant auth alone must fail
    env.mock_auths(&[MockAuth {
        address: &outsider,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "new_game",
            args: (&player_a, &player_b).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(client.try_new_game(&player_a, &player_b).is_err());

    // Only player_a auth without player_b must fail
    env.mock_auths(&[MockAuth {
        address: &player_a,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "new_game",
            args: (&player_a, &player_b).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(client.try_new_game(&player_a, &player_b).is_err());
}

#[test]
fn test_new_game_refuses_active_match_until_finished() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    // Cannot overwrite while in Setup phase
    assert_eq!(
        client.try_new_game(&player_a, &player_b),
        Err(Ok(BattleshipError::MatchAlreadyActive))
    );

    let board_a = [0u32; 100];
    let mut board_b = [0u32; 100];
    board_b[0] = 1;
    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);
    let commitment_a = make_commitment(&env, &board_a, &salt_a);
    let commitment_b = make_commitment(&env, &board_b, &salt_b);
    let (root_a, _) = build_merkle_tree(&env, &board_a);
    let (root_b, tree_b) = build_merkle_tree(&env, &board_b);

    client.commit_board(&player_a, &commitment_a, &root_a);
    client.commit_board(&player_b, &commitment_b, &root_b);
    assert_eq!(client.get_state().turn_state.phase, Phase::Attack);

    // Cannot overwrite while in Attack phase
    assert_eq!(
        client.try_new_game(&player_a, &player_b),
        Err(Ok(BattleshipError::MatchAlreadyActive))
    );

    // Finish the game and verify starting a new match succeeds
    let mut state = client.get_state();
    state.ship_status.remaining_b = 1;
    env.as_contract(&client.address, || {
        env.storage().instance().set(&GAME_KEY, &state);
    });
    client.attack(&player_a, &0, &0);
    let proof = get_merkle_proof(&env, &tree_b, 0);
    client.reveal_cell(&player_b, &0, &0, &1, &proof);
    assert_eq!(client.get_state().turn_state.phase, Phase::Finished);

    assert_eq!(client.try_new_game(&player_a, &player_b), Ok(Ok(())));
    assert_eq!(client.get_state().turn_state.phase, Phase::Setup);
}
