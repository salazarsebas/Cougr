use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env};

fn setup() -> (Env, ProofOfHuntContractClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register(ProofOfHuntContract, ());
    let client = ProofOfHuntContractClient::new(&env, &contract_id);
    let player = Address::generate(&env);
    (env, client, player)
}

fn append_u32_field(env: &Env, out: &mut Bytes, value: u32) {
    for _ in 0..28 {
        out.push_back(0);
    }
    let arr = value.to_be_bytes();
    out.append(&Bytes::from_array(env, &arr));
}

fn hash_pair(env: &Env, left: &BytesN<32>, right: &BytesN<32>) -> BytesN<32> {
    let mut bytes = Bytes::new(env);
    for i in 0..32 {
        bytes.push_back(left.get(i).unwrap());
    }
    for i in 0..32 {
        bytes.push_back(right.get(i).unwrap());
    }
    env.crypto().sha256(&bytes).into()
}

#[allow(clippy::too_many_arguments)]
fn proof_input(
    env: &Env,
    root: &BytesN<32>,
    x: u32,
    y: u32,
    is_treasure: bool,
    leaf_hash: &BytesN<32>,
    sibling_hash: &BytesN<32>,
    sibling_on_left: bool,
    nullifier_seed: u8,
) -> ProofInput {
    let mut public_inputs = Bytes::new(env);
    append_u32_field(env, &mut public_inputs, if is_treasure { 1 } else { 0 });
    append_u32_field(env, &mut public_inputs, x);
    append_u32_field(env, &mut public_inputs, y);
    for i in 0..32 {
        public_inputs.push_back(root.get(i).unwrap());
    }

    ProofInput {
        proof: BytesN::from_array(env, &[0u8; 256]),
        public_inputs,
        nullifier: BytesN::from_array(env, &[nullifier_seed; 32]),
        leaf_hash: leaf_hash.clone(),
        sibling_hash: sibling_hash.clone(),
        sibling_on_left,
    }
}

#[test]
fn test_initialization() {
    let (env, client, player) = setup();
    env.mock_all_auths();

    let leaf = BytesN::from_array(&env, &[7u8; 32]);
    let sibling = BytesN::from_array(&env, &[9u8; 32]);
    let root = hash_pair(&env, &leaf, &sibling);

    client.init_game(&player, &root, &4, &4);

    let state = client.get_state();
    assert_eq!(state.player, player);
    assert_eq!(state.map_commitment, root);
    assert_eq!(state.width, 4);
    assert_eq!(state.height, 4);
    assert_eq!(state.player_state.health, MAX_HEALTH);
    assert_eq!(state.status, GameStatus::Active);
}

#[test]
fn test_valid_exploration_proof_acceptance() {
    let (env, client, player) = setup();
    env.mock_all_auths();

    let leaf = BytesN::from_array(&env, &[5u8; 32]);
    let sibling = BytesN::from_array(&env, &[6u8; 32]);
    let root = hash_pair(&env, &leaf, &sibling);

    client.init_game(&player, &root, &4, &4);

    let proof = proof_input(&env, &root, 1, 2, true, &leaf, &sibling, false, 1);
    client.explore(&player, &1, &2, &proof);

    let state = client.get_state();
    assert_eq!(state.player_state.position_x, 1);
    assert_eq!(state.player_state.position_y, 2);
    assert_eq!(state.player_state.discoveries, 1);
    assert_eq!(state.discovered_cells, 1);
}

#[test]
#[should_panic]
fn test_invalid_proof_rejection() {
    let (env, client, player) = setup();
    env.mock_all_auths();

    let leaf = BytesN::from_array(&env, &[11u8; 32]);
    let sibling = BytesN::from_array(&env, &[22u8; 32]);
    let root = hash_pair(&env, &leaf, &sibling);

    client.init_game(&player, &root, &4, &4);

    let wrong_root = BytesN::from_array(&env, &[33u8; 32]);
    let bad_proof = proof_input(&env, &wrong_root, 0, 0, false, &leaf, &sibling, false, 2);
    client.explore(&player, &0, &0, &bad_proof);
}

#[test]
fn test_progression_updates_after_valid_exploration() {
    let (env, client, player) = setup();
    env.mock_all_auths();

    let leaf_a = BytesN::from_array(&env, &[1u8; 32]);
    let sibling_a = BytesN::from_array(&env, &[2u8; 32]);
    let root = hash_pair(&env, &leaf_a, &sibling_a);

    client.init_game(&player, &root, &4, &4);

    let p1 = proof_input(&env, &root, 0, 0, false, &leaf_a, &sibling_a, false, 3);
    client.explore(&player, &0, &0, &p1);

    let state_after = client.get_state();
    assert_eq!(state_after.player_state.health, MAX_HEALTH - 1);
    assert_eq!(state_after.player_state.score, 5);
    assert_eq!(state_after.status, GameStatus::Active);
}

#[test]
fn test_premium_hint_action_flow() {
    let (env, client, player) = setup();
    env.mock_all_auths();

    let leaf = BytesN::from_array(&env, &[44u8; 32]);
    let sibling = BytesN::from_array(&env, &[55u8; 32]);
    let root = hash_pair(&env, &leaf, &sibling);

    client.init_game(&player, &root, &4, &4);

    let receipt = BytesN::from_array(&env, &[90u8; 32]);
    client.credit_x402_payment(&player, &player, &3, &receipt);

    client.purchase_hint(&player, &0);
    client.purchase_hint(&player, &1);

    let state = client.get_state();
    assert_eq!(state.hint_usage.hints_used, 1);
    assert_eq!(state.hint_usage.scans_used, 1);
    assert_eq!(state.x402_credits, 0);
}

#[test]
fn test_game_completion_condition() {
    let (env, client, player) = setup();
    env.mock_all_auths();

    let leaf = BytesN::from_array(&env, &[101u8; 32]);
    let sibling = BytesN::from_array(&env, &[102u8; 32]);
    let root = hash_pair(&env, &leaf, &sibling);

    client.init_game(&player, &root, &1, &1);

    let p = proof_input(&env, &root, 0, 0, true, &leaf, &sibling, false, 10);
    client.explore(&player, &0, &0, &p);

    let state = client.get_state();
    assert_eq!(state.status, GameStatus::Won);
    assert!(client.is_finished());
}

#[test]
fn test_failure_condition_on_health_depletion() {
    let (env, client, player) = setup();
    env.mock_all_auths();

    let leaf = BytesN::from_array(&env, &[13u8; 32]);
    let sibling = BytesN::from_array(&env, &[14u8; 32]);
    let root = hash_pair(&env, &leaf, &sibling);

    client.init_game(&player, &root, &2, &2);

    let p1 = proof_input(&env, &root, 0, 0, false, &leaf, &sibling, false, 11);
    client.explore(&player, &0, &0, &p1);

    let p2 = proof_input(&env, &root, 1, 0, false, &leaf, &sibling, false, 12);
    client.explore(&player, &1, &0, &p2);

    let p3 = proof_input(&env, &root, 0, 1, false, &leaf, &sibling, false, 13);
    client.explore(&player, &0, &1, &p3);

    let state = client.get_state();
    assert_eq!(state.player_state.health, 0);
    assert_eq!(state.status, GameStatus::Lost);
    assert!(client.is_finished());
}
