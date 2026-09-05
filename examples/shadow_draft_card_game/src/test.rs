use super::*;
use cougr_core::zk::{G1Point, G2Point, Groth16Proof, Scalar, VerificationKey};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Vec};

// ─── Test helpers ─────────────────────────────────────────────────────────────

fn mock_g1_point(env: &Env) -> G1Point {
    G1Point {
        bytes: BytesN::from_array(env, &[0u8; 64]),
    }
}

fn mock_g2_point(env: &Env) -> G2Point {
    G2Point {
        bytes: BytesN::from_array(env, &[0u8; 128]),
    }
}

fn mock_proof(env: &Env) -> Groth16Proof {
    Groth16Proof {
        a: mock_g1_point(env),
        b: mock_g2_point(env),
        c: mock_g1_point(env),
    }
}

fn mock_verification_key(env: &Env, num_public_inputs: u32) -> VerificationKey {
    let mut ic = Vec::new(env);
    for _ in 0..=num_public_inputs {
        ic.push_back(mock_g1_point(env));
    }

    VerificationKey {
        alpha: mock_g1_point(env),
        beta: mock_g2_point(env),
        gamma: mock_g2_point(env),
        delta: mock_g2_point(env),
        ic,
    }
}

fn empty_public_inputs(env: &Env) -> Vec<Scalar> {
    Vec::new(env)
}

fn setup() -> (Env, ShadowDraftCardGameClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ShadowDraftCardGame, ());
    let client = ShadowDraftCardGameClient::new(&env, &contract_id);
    let p1 = Address::generate(&env);
    let p2 = Address::generate(&env);
    client.init_match(&p1, &p2);
    (env, client, p1, p2)
}

fn make_nonce(env: &Env, seed: u8) -> BytesN<16> {
    BytesN::from_array(env, &[seed; 16])
}

fn make_choice(env: &Env, card_id: u32, nonce: &BytesN<16>) -> ChoiceInput {
    ChoiceInput {
        commitment: compute_commitment(env, card_id, nonce),
    }
}

fn make_play(env: &Env, card_id: u32, nonce: &BytesN<16>) -> PlayInput {
    PlayInput {
        card_id,
        nonce: nonce.clone(),
        proof: mock_proof(env),
        public_inputs: empty_public_inputs(env),
    }
}

/// Run a full round: both players commit then both reveal.
#[allow(clippy::too_many_arguments)]
fn play_round(
    env: &Env,
    client: &ShadowDraftCardGameClient,
    p1: &Address,
    p2: &Address,
    card_one: u32,
    card_two: u32,
    nonce_seed_one: u8,
    nonce_seed_two: u8,
) {
    let n1 = make_nonce(env, nonce_seed_one);
    let n2 = make_nonce(env, nonce_seed_two);
    client.submit_choice(p1, &make_choice(env, card_one, &n1));
    client.submit_choice(p2, &make_choice(env, card_two, &n2));
    client.play_card(p1, &make_play(env, card_one, &n1));
    client.play_card(p2, &make_play(env, card_two, &n2));
}

// ─── Match initialization ─────────────────────────────────────────────────────

#[test]
fn test_init_match_initial_state() {
    let (_, client, p1, p2) = setup();
    let state = client.get_state();

    assert_eq!(state.player_one, p1);
    assert_eq!(state.player_two, p2);
    assert_eq!(state.status.phase, PHASE_DRAFT);
    assert_eq!(state.status.status, STATUS_IN_PROGRESS);
    assert_eq!(state.board.round, 1);
    assert_eq!(state.board.score_one, 0);
    assert_eq!(state.board.score_two, 0);
    assert_eq!(state.active_format, 1);
    assert!(state.banned_cards.is_empty());
}

#[test]
fn test_cannot_init_match_twice() {
    let (_, client, p1, p2) = setup();
    let result = client.try_init_match(&p1, &p2);
    assert!(result.is_err());
}

// ─── Hidden-choice / draft phase ─────────────────────────────────────────────

#[test]
fn test_submit_choice_stays_in_draft_until_both_commit() {
    let (env, client, p1, p2) = setup();

    let n1 = make_nonce(&env, 1);
    client.submit_choice(&p1, &make_choice(&env, 5, &n1));
    // Only one player committed - still in draft.
    assert_eq!(client.get_state().status.phase, PHASE_DRAFT);

    let n2 = make_nonce(&env, 2);
    client.submit_choice(&p2, &make_choice(&env, 3, &n2));
    // Both committed - advances to play.
    assert_eq!(client.get_state().status.phase, PHASE_PLAY);
}

#[test]
fn test_cannot_commit_twice_same_player() {
    let (env, client, p1, _) = setup();
    let n = make_nonce(&env, 1);
    let c = make_choice(&env, 5, &n);
    client.submit_choice(&p1, &c.clone());
    assert!(client.try_submit_choice(&p1, &c).is_err());
}

#[test]
fn test_non_player_cannot_commit() {
    let (env, client, _, _) = setup();
    let intruder = Address::generate(&env);
    let n = make_nonce(&env, 1);
    let result = client.try_submit_choice(&intruder, &make_choice(&env, 5, &n));
    assert!(result.is_err());
}

#[test]
fn test_cannot_commit_in_play_phase() {
    let (env, client, p1, p2) = setup();
    // Advance to play phase.
    let n1 = make_nonce(&env, 1);
    let n2 = make_nonce(&env, 2);
    client.submit_choice(&p1, &make_choice(&env, 5, &n1));
    client.submit_choice(&p2, &make_choice(&env, 3, &n2));

    // Attempt another commit while in PLAY - wrong phase.
    let n3 = make_nonce(&env, 3);
    assert!(client
        .try_submit_choice(&p1, &make_choice(&env, 1, &n3))
        .is_err());
}

// ─── Play card / proof validation ─────────────────────────────────────────────

#[test]
fn test_play_card_valid_commitment_resolves_round() {
    let (env, client, p1, p2) = setup();

    let n1 = make_nonce(&env, 10);
    let n2 = make_nonce(&env, 20);
    client.submit_choice(&p1, &make_choice(&env, 7, &n1));
    client.submit_choice(&p2, &make_choice(&env, 4, &n2));

    // VK not set - proof gate is skipped; commitment + format checks still run.
    client.play_card(&p1, &make_play(&env, 7, &n1));
    client.play_card(&p2, &make_play(&env, 4, &n2));

    // Card 7 (power 7) beats card 4 (power 4) → player one wins the round.
    let state = client.get_state();
    assert_eq!(state.board.score_one, 1);
    assert_eq!(state.board.score_two, 0);
    assert_eq!(state.board.last_played_one, 7);
    assert_eq!(state.board.last_played_two, 4);
    // Resolution resets to draft for the next round.
    assert_eq!(state.status.phase, PHASE_DRAFT);
    assert_eq!(state.board.round, 2);
}

#[test]
fn test_commitment_mismatch_is_rejected() {
    let (env, client, p1, p2) = setup();

    let n1 = make_nonce(&env, 10);
    let n2 = make_nonce(&env, 20);
    // Player one commits to card 7.
    client.submit_choice(&p1, &make_choice(&env, 7, &n1));
    client.submit_choice(&p2, &make_choice(&env, 4, &n2));

    // Player one tries to reveal card 5 with a different nonce - mismatch.
    let wrong_nonce = make_nonce(&env, 99);
    assert!(client
        .try_play_card(&p1, &make_play(&env, 5, &wrong_nonce))
        .is_err());
}

#[test]
fn test_cannot_play_before_both_commit() {
    let (env, client, p1, _) = setup();
    // Only player one commits - stays in DRAFT.
    let n1 = make_nonce(&env, 10);
    client.submit_choice(&p1, &make_choice(&env, 7, &n1));

    // Still in DRAFT phase - play_card should fail.
    assert!(client.try_play_card(&p1, &make_play(&env, 7, &n1)).is_err());
}

#[test]
fn test_cannot_reveal_twice() {
    let (env, client, p1, p2) = setup();
    let n1 = make_nonce(&env, 10);
    let n2 = make_nonce(&env, 20);
    client.submit_choice(&p1, &make_choice(&env, 7, &n1));
    client.submit_choice(&p2, &make_choice(&env, 4, &n2));

    client.play_card(&p1, &make_play(&env, 7, &n1));
    // Second reveal for same player in same round should fail.
    assert!(client.try_play_card(&p1, &make_play(&env, 7, &n1)).is_err());
}

#[test]
fn test_invalid_card_id_rejected() {
    let (env, client, p1, p2) = setup();
    let n1 = make_nonce(&env, 1);
    let n2 = make_nonce(&env, 2);
    // Commit to an out-of-range card ID.
    let bad_id = 99u32;
    client.submit_choice(
        &p1,
        &ChoiceInput {
            commitment: compute_commitment(&env, bad_id, &n1),
        },
    );
    client.submit_choice(&p2, &make_choice(&env, 3, &n2));

    // play_card must reject invalid card IDs before checking the commitment.
    assert!(client
        .try_play_card(
            &p1,
            &PlayInput {
                card_id: bad_id,
                nonce: n1,
                proof: mock_proof(&env),
                public_inputs: empty_public_inputs(&env),
            }
        )
        .is_err());
}

// ─── Valid proof accepted (VK not set) ───────────────────────────────────────

#[test]
fn test_proof_gate_skipped_without_vk() {
    // When no VK is registered the Groth16 gate is bypassed and any
    // commitment-valid play succeeds regardless of the proof bytes.
    let (env, client, p1, p2) = setup();
    let n1 = make_nonce(&env, 1);
    let n2 = make_nonce(&env, 2);
    client.submit_choice(&p1, &make_choice(&env, 5, &n1));
    client.submit_choice(&p2, &make_choice(&env, 3, &n2));

    // mock_proof contains zeroed-out curve points that would fail real
    // verification - they must be accepted here because vk_set == false.
    client.play_card(&p1, &make_play(&env, 5, &n1));
    client.play_card(&p2, &make_play(&env, 3, &n2));

    let state = client.get_state();
    assert_eq!(state.board.last_played_one, 5);
    assert_eq!(state.board.last_played_two, 3);
}

// ─── Invalid proof rejected (VK set) ─────────────────────────────────────────

#[test]
fn test_invalid_proof_rejected_when_vk_set() {
    // When a VK is registered the Groth16 gate activates.
    // A mock proof with zeroed curve points will fail pairing verification,
    // causing play_card to panic with InvalidProof.
    let (env, client, p1, p2) = setup();

    let vk = mock_verification_key(&env, 1);
    client.set_vk(&vk);

    let n1 = make_nonce(&env, 1);
    let n2 = make_nonce(&env, 2);
    client.submit_choice(&p1, &make_choice(&env, 5, &n1));
    client.submit_choice(&p2, &make_choice(&env, 3, &n2));

    // The mock proof is cryptographically invalid - the call must fail.
    let result = client.try_play_card(&p1, &make_play(&env, 5, &n1));
    assert!(result.is_err());
}

// ─── Round resolution ─────────────────────────────────────────────────────────

#[test]
fn test_higher_power_card_wins_round() {
    let (env, client, p1, p2) = setup();
    // Card 8 (power 8) vs card 3 (power 3) → player one wins.
    play_round(&env, &client, &p1, &p2, 8, 3, 1, 2);

    let state = client.get_state();
    assert_eq!(state.board.score_one, 1);
    assert_eq!(state.board.score_two, 0);
}

#[test]
fn test_tie_awards_no_point() {
    let (env, client, p1, p2) = setup();
    // Both play card 5 (power 5) - tie, no points.
    play_round(&env, &client, &p1, &p2, 5, 5, 1, 2);

    let state = client.get_state();
    assert_eq!(state.board.score_one, 0);
    assert_eq!(state.board.score_two, 0);
    assert_eq!(state.board.round, 2);
}

#[test]
fn test_lower_power_player_loses_round() {
    let (env, client, p1, p2) = setup();
    // Card 2 (power 2) vs card 9 (power 3) → player two wins.
    play_round(&env, &client, &p1, &p2, 2, 9, 1, 2);

    let state = client.get_state();
    assert_eq!(state.board.score_one, 0);
    assert_eq!(state.board.score_two, 1);
}

#[test]
fn test_win_after_three_rounds() {
    let (env, client, p1, p2) = setup();

    // Player one wins three rounds with card 10 (power 5) vs card 1 (power 1).
    for i in 0..3u32 {
        play_round(
            &env,
            &client,
            &p1,
            &p2,
            10,
            1,
            (i * 10 + 1) as u8,
            (i * 10 + 2) as u8,
        );
    }

    let state = client.get_state();
    assert_eq!(state.status.status, STATUS_PLAYER_ONE_WINS);
    assert_eq!(state.status.phase, PHASE_FINISHED);
    assert_eq!(state.board.score_one, 3);
}

#[test]
fn test_cannot_play_after_match_ends() {
    let (env, client, p1, p2) = setup();

    for i in 0..3u32 {
        play_round(
            &env,
            &client,
            &p1,
            &p2,
            10,
            1,
            (i * 10 + 1) as u8,
            (i * 10 + 2) as u8,
        );
    }

    // Match is finished - commit attempt must fail.
    let n = make_nonce(&env, 99);
    assert!(client
        .try_submit_choice(&p1, &make_choice(&env, 5, &n))
        .is_err());
}

#[test]
fn test_round_resets_between_rounds() {
    let (env, client, p1, p2) = setup();
    play_round(&env, &client, &p1, &p2, 7, 3, 1, 2);

    // After resolution the game returns to DRAFT and round counter increments.
    let state = client.get_state();
    assert_eq!(state.status.phase, PHASE_DRAFT);
    assert_eq!(state.board.round, 2);
    // Players can now commit again for the new round.
    let n1 = make_nonce(&env, 11);
    let n2 = make_nonce(&env, 12);
    client.submit_choice(&p1, &make_choice(&env, 6, &n1));
    client.submit_choice(&p2, &make_choice(&env, 2, &n2));
    assert_eq!(client.get_state().status.phase, PHASE_PLAY);
}

// ─── Format governance ────────────────────────────────────────────────────────

#[test]
fn test_ban_card_via_governance() {
    let (_, client, p1, _) = setup();

    client.submit_format_proposal(
        &p1,
        &ProposalInput {
            ban_card: 8,
            unban_card: 0,
        },
    );

    let state = client.get_state();
    assert_eq!(state.banned_cards.len(), 1);
    assert_eq!(state.banned_cards.get(0).unwrap(), 8);
    // Proposal was accepted - no pending proposals.
    assert_eq!(state.pending_proposals, 0);
}

#[test]
fn test_unban_card_via_governance() {
    let (_, client, p1, _) = setup();

    // Ban card 8 first.
    client.submit_format_proposal(
        &p1,
        &ProposalInput {
            ban_card: 8,
            unban_card: 0,
        },
    );
    assert_eq!(client.get_state().banned_cards.len(), 1);

    // Unban card 8.
    client.submit_format_proposal(
        &p1,
        &ProposalInput {
            ban_card: 0,
            unban_card: 8,
        },
    );
    assert!(client.get_state().banned_cards.is_empty());
}

#[test]
fn test_banned_card_cannot_be_played() {
    let (env, client, p1, p2) = setup();

    // Ban card 7.
    client.submit_format_proposal(
        &p1,
        &ProposalInput {
            ban_card: 7,
            unban_card: 0,
        },
    );

    let n1 = make_nonce(&env, 1);
    let n2 = make_nonce(&env, 2);
    // Player one commits to the now-banned card 7.
    client.submit_choice(&p1, &make_choice(&env, 7, &n1));
    client.submit_choice(&p2, &make_choice(&env, 3, &n2));

    // Playing the banned card must be rejected.
    assert!(client.try_play_card(&p1, &make_play(&env, 7, &n1)).is_err());
}

#[test]
fn test_multiple_bans_accumulate() {
    let (_, client, p1, _) = setup();

    client.submit_format_proposal(
        &p1,
        &ProposalInput {
            ban_card: 5,
            unban_card: 0,
        },
    );
    client.submit_format_proposal(
        &p1,
        &ProposalInput {
            ban_card: 8,
            unban_card: 0,
        },
    );

    let state = client.get_state();
    assert_eq!(state.banned_cards.len(), 2);
}

#[test]
fn test_governance_proposal_accepted_status() {
    let (_, client, p1, _) = setup();

    // After proposal is applied there are no pending proposals.
    client.submit_format_proposal(
        &p1,
        &ProposalInput {
            ban_card: 5,
            unban_card: 0,
        },
    );
    assert_eq!(client.get_state().pending_proposals, 0);

    // Card 5 is banned.
    let state = client.get_state();
    let card5_banned =
        (0..state.banned_cards.len()).any(|i| state.banned_cards.get(i).unwrap() == 5);
    assert!(card5_banned);
}

// ─── Commitment utility ───────────────────────────────────────────────────────

#[test]
fn test_compute_commitment_deterministic() {
    let env = Env::default();
    let nonce = make_nonce(&env, 42);
    let c1 = compute_commitment(&env, 7, &nonce);
    let c2 = compute_commitment(&env, 7, &nonce);
    assert_eq!(c1, c2);
}

#[test]
fn test_different_cards_produce_different_commitments() {
    let env = Env::default();
    let nonce = make_nonce(&env, 42);
    let c1 = compute_commitment(&env, 7, &nonce);
    let c2 = compute_commitment(&env, 8, &nonce);
    assert_ne!(c1, c2);
}

#[test]
fn test_different_nonces_produce_different_commitments() {
    let env = Env::default();
    let n1 = make_nonce(&env, 1);
    let n2 = make_nonce(&env, 2);
    let c1 = compute_commitment(&env, 7, &n1);
    let c2 = compute_commitment(&env, 7, &n2);
    assert_ne!(c1, c2);
}
