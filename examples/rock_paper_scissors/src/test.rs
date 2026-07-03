use super::*;
use cougr_core::component::ComponentTrait;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

fn setup_game() -> (
    Env,
    RockPaperScissorsContractClient<'static>,
    Address,
    Address,
) {
    let env = Env::default();
    let contract_id = env.register(RockPaperScissorsContract, ());
    let client = RockPaperScissorsContractClient::new(&env, &contract_id);

    let player_a = Address::generate(&env);
    let player_b = Address::generate(&env);

    (env, client, player_a, player_b)
}

fn make_hash(env: &Env, choice: u32, salt: &BytesN<32>) -> BytesN<32> {
    let mut data = Bytes::new(env);
    data.append(&Bytes::from_array(env, &choice.to_be_bytes()));
    for i in 0..32 {
        data.push_back(salt.get(i).unwrap());
    }
    env.crypto().sha256(&data).into()
}

#[test]
fn test_new_match() {
    let (_env, client, player_a, player_b) = setup_game();

    client.new_match(&player_a, &player_b, &3);

    let state = client.get_state();
    assert_eq!(state.phase, Phase::Committing);
    assert_eq!(state.round, 1);

    let score = client.get_score();
    assert_eq!(score.wins_a, 0);
    assert_eq!(score.wins_b, 0);
    assert_eq!(score.draws, 0);
    assert_eq!(score.best_of, 3);
}

#[test]
fn test_commit_phase() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_match(&player_a, &player_b, &1);

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let hash_a = make_hash(&env, 0, &salt_a); // Rock
    let hash_b = make_hash(&env, 1, &salt_b); // Paper

    client.commit(&player_a, &hash_a);
    let state = client.get_state();
    assert_eq!(state.phase, Phase::Committing);

    client.commit(&player_b, &hash_b);
    let state = client.get_state();
    assert_eq!(state.phase, Phase::Revealing);
}

#[test]
fn test_reveal_and_resolve_rock_vs_scissors() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_match(&player_a, &player_b, &1);

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let hash_a = make_hash(&env, 0, &salt_a); // Rock
    let hash_b = make_hash(&env, 2, &salt_b); // Scissors

    client.commit(&player_a, &hash_a);
    client.commit(&player_b, &hash_b);

    client.reveal(&player_a, &0, &salt_a);
    client.reveal(&player_b, &2, &salt_b);

    let state = client.get_state();
    assert_eq!(state.phase, Phase::Resolved);
    assert_eq!(state.winner, Some(player_a.clone()));

    let score = client.get_score();
    assert_eq!(score.wins_a, 1);
    assert_eq!(score.wins_b, 0);
}

#[test]
fn test_paper_vs_rock() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_match(&player_a, &player_b, &1);

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let hash_a = make_hash(&env, 1, &salt_a); // Paper
    let hash_b = make_hash(&env, 0, &salt_b); // Rock

    client.commit(&player_a, &hash_a);
    client.commit(&player_b, &hash_b);

    client.reveal(&player_a, &1, &salt_a);
    client.reveal(&player_b, &0, &salt_b);

    let score = client.get_score();
    assert_eq!(score.wins_a, 1);
    assert_eq!(score.wins_b, 0);
}

#[test]
fn test_scissors_vs_paper() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_match(&player_a, &player_b, &1);

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let hash_a = make_hash(&env, 2, &salt_a); // Scissors
    let hash_b = make_hash(&env, 1, &salt_b); // Paper

    client.commit(&player_a, &hash_a);
    client.commit(&player_b, &hash_b);

    client.reveal(&player_a, &2, &salt_a);
    client.reveal(&player_b, &1, &salt_b);

    let score = client.get_score();
    assert_eq!(score.wins_a, 1);
    assert_eq!(score.wins_b, 0);
}

#[test]
fn test_draw_rock_vs_rock() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_match(&player_a, &player_b, &1);

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let hash_a = make_hash(&env, 0, &salt_a); // Rock
    let hash_b = make_hash(&env, 0, &salt_b); // Rock

    client.commit(&player_a, &hash_a);
    client.commit(&player_b, &hash_b);

    client.reveal(&player_a, &0, &salt_a);
    client.reveal(&player_b, &0, &salt_b);

    let score = client.get_score();
    assert_eq!(score.draws, 1);
}

#[test]
fn test_draw_paper_vs_paper() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_match(&player_a, &player_b, &1);

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let hash_a = make_hash(&env, 1, &salt_a);
    let hash_b = make_hash(&env, 1, &salt_b);

    client.commit(&player_a, &hash_a);
    client.commit(&player_b, &hash_b);

    client.reveal(&player_a, &1, &salt_a);
    client.reveal(&player_b, &1, &salt_b);

    let score = client.get_score();
    assert_eq!(score.draws, 1);
}

#[test]
fn test_draw_scissors_vs_scissors() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_match(&player_a, &player_b, &1);

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let hash_a = make_hash(&env, 2, &salt_a);
    let hash_b = make_hash(&env, 2, &salt_b);

    client.commit(&player_a, &hash_a);
    client.commit(&player_b, &hash_b);

    client.reveal(&player_a, &2, &salt_a);
    client.reveal(&player_b, &2, &salt_b);

    let score = client.get_score();
    assert_eq!(score.draws, 1);
}

#[test]
fn test_player_b_wins() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_match(&player_a, &player_b, &1);

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let hash_a = make_hash(&env, 0, &salt_a); // Rock
    let hash_b = make_hash(&env, 1, &salt_b); // Paper

    client.commit(&player_a, &hash_a);
    client.commit(&player_b, &hash_b);

    client.reveal(&player_a, &0, &salt_a);
    client.reveal(&player_b, &1, &salt_b);

    let state = client.get_state();
    assert_eq!(state.winner, Some(player_b.clone()));

    let score = client.get_score();
    assert_eq!(score.wins_b, 1);
}

#[test]
#[should_panic(expected = "Hash mismatch")]
fn test_hash_mismatch() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_match(&player_a, &player_b, &1);

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);
    let wrong_salt = BytesN::from_array(&env, &[99u8; 32]);

    let hash_a = make_hash(&env, 0, &salt_a);
    let hash_b = make_hash(&env, 1, &salt_b);

    client.commit(&player_a, &hash_a);
    client.commit(&player_b, &hash_b);

    // Try to reveal with wrong salt
    client.reveal(&player_a, &0, &wrong_salt);
}

#[test]
fn test_best_of_three() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_match(&player_a, &player_b, &3);

    // Round 1: A wins
    let salt_a1 = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b1 = BytesN::from_array(&env, &[2u8; 32]);
    client.commit(&player_a, &make_hash(&env, 0, &salt_a1));
    client.commit(&player_b, &make_hash(&env, 2, &salt_b1));
    client.reveal(&player_a, &0, &salt_a1);
    client.reveal(&player_b, &2, &salt_b1);

    let state = client.get_state();
    assert_eq!(state.phase, Phase::Committing);
    assert_eq!(state.round, 2);

    // Round 2: A wins again (match over)
    let salt_a2 = BytesN::from_array(&env, &[3u8; 32]);
    let salt_b2 = BytesN::from_array(&env, &[4u8; 32]);
    client.commit(&player_a, &make_hash(&env, 1, &salt_a2));
    client.commit(&player_b, &make_hash(&env, 0, &salt_b2));
    client.reveal(&player_a, &1, &salt_a2);
    client.reveal(&player_b, &0, &salt_b2);

    let state = client.get_state();
    assert_eq!(state.phase, Phase::Resolved);
    assert_eq!(state.winner, Some(player_a.clone()));

    let score = client.get_score();
    assert_eq!(score.wins_a, 2);
    assert_eq!(score.wins_b, 0);
}

#[test]
#[should_panic(expected = "Already committed")]
fn test_double_commit() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_match(&player_a, &player_b, &1);

    let salt = BytesN::from_array(&env, &[1u8; 32]);
    let hash = make_hash(&env, 0, &salt);

    client.commit(&player_a, &hash);
    client.commit(&player_a, &hash);
}

#[test]
#[should_panic(expected = "Not in reveal phase")]
fn test_reveal_before_both_commit() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_match(&player_a, &player_b, &1);

    let salt = BytesN::from_array(&env, &[1u8; 32]);
    let hash = make_hash(&env, 0, &salt);

    client.commit(&player_a, &hash);
    client.reveal(&player_a, &0, &salt);
}

#[test]
fn test_component_traits() {
    let env = Env::default();

    let commitment = PlayerCommitment {
        hash: BytesN::from_array(&env, &[0u8; 32]),
        revealed: false,
    };
    let serialized = commitment.serialize(&env);
    assert_eq!(serialized.len(), 33);
    assert_eq!(PlayerCommitment::component_type(), symbol_short!("commit"));

    let match_state = MatchState {
        phase: Phase::Committing,
        winner: None,
        round: 1,
    };
    let serialized = match_state.serialize(&env);
    assert_eq!(serialized.len(), 5);
    assert_eq!(MatchState::component_type(), symbol_short!("match"));
}

#[test]
fn test_all_nine_combinations() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    let combinations = [
        (0, 0, 0, 0, 1), // RR -> draw
        (0, 1, 0, 1, 0), // RP -> B wins
        (0, 2, 1, 0, 0), // RS -> A wins
        (1, 0, 1, 0, 0), // PR -> A wins
        (1, 1, 0, 0, 1), // PP -> draw
        (1, 2, 0, 1, 0), // PS -> B wins
        (2, 0, 0, 1, 0), // SR -> B wins
        (2, 1, 1, 0, 0), // SP -> A wins
        (2, 2, 0, 0, 1), // SS -> draw
    ];

    for (idx, (choice_a, choice_b, exp_a, exp_b, exp_draw)) in combinations.iter().enumerate() {
        client.new_match(&player_a, &player_b, &1);

        let salt_a = BytesN::from_array(&env, &[(idx * 2) as u8; 32]);
        let salt_b = BytesN::from_array(&env, &[(idx * 2 + 1) as u8; 32]);

        client.commit(&player_a, &make_hash(&env, *choice_a, &salt_a));
        client.commit(&player_b, &make_hash(&env, *choice_b, &salt_b));

        client.reveal(&player_a, choice_a, &salt_a);
        client.reveal(&player_b, choice_b, &salt_b);

        let score = client.get_score();
        assert_eq!(score.wins_a, *exp_a, "Test {} failed for wins_a", idx);
        assert_eq!(score.wins_b, *exp_b, "Test {} failed for wins_b", idx);
        assert_eq!(score.draws, *exp_draw, "Test {} failed for draws", idx);
    }
}
