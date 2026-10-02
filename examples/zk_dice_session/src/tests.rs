use super::*;
use cougr_core::circuits::{fair_dice, test_fixtures, CircuitId};
use cougr_core::test::GameHarness;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env,
};

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Initialise the contract and return a client plus the seed/proof fixtures.
fn setup(
    harness: &GameHarness,
) -> (
    ZkDiceSessionClient<'_>,
    soroban_sdk::BytesN<32>,      // seed_commitment
    cougr_core::zk::Groth16Proof, // pipeline proof
) {
    let client = ZkDiceSessionClient::new(harness.env(), harness.contract_id());
    let seed = test_fixtures::pipeline_public_bytes32(harness.env(), CircuitId::FairDice);
    let proof = test_fixtures::pipeline_proof(harness.env(), CircuitId::FairDice);
    client.init_game(&6, &seed);
    (client, seed, proof)
}

// ─── Definition-of-done test 1 ───────────────────────────────────────────────
// "Rolls succeed inside an active session."

#[test]
fn roll_succeeds_inside_active_session() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, ZkDiceSession);
    let (client, _, proof) = setup(&harness);

    let owner = Address::generate(harness.env());

    // open_session – the single wallet prompt per session
    let session = client.open_session(&owner, &5, &10_000);

    // roll – no wallet prompt, session key authorizes
    let ok = client.roll(&owner, &session.key_id, &6, &5, &proof);
    assert!(ok, "roll inside active session must succeed");

    let record = client.last_roll(&owner);
    assert_eq!(record.roll, 6);
    assert_eq!(record.nonce, 5);
    assert_eq!(record.player, owner);
}

// ─── Definition-of-done test 2 ───────────────────────────────────────────────
// "Rolls fail with no session."

#[test]
#[should_panic(expected = "session missing")]
fn roll_fails_with_no_session() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, ZkDiceSession);
    let (client, _, proof) = setup(&harness);

    let owner = Address::generate(harness.env());
    // no open_session call – the key_id is random/unknown
    let fake_key_id = soroban_sdk::BytesN::from_array(harness.env(), &[0xAB; 32]);

    // must panic: "session missing"
    client.roll(&owner, &fake_key_id, &6, &5, &proof);
}

// ─── Definition-of-done test 3 ───────────────────────────────────────────────
// "An expired session cannot authorize a roll."

#[test]
#[should_panic(expected = "session rejected")]
fn roll_fails_after_session_expiry() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, ZkDiceSession);
    let (client, _, proof) = setup(&harness);

    let owner = Address::generate(harness.env());

    // open_session with a short window (50 seconds)
    let session = client.open_session(&owner, &5, &50);

    // advance the ledger past the expiry
    harness.env().ledger().with_mut(|li| {
        li.timestamp = 10_000; // well past the 50-second window
    });

    // must panic: "session rejected" (SessionExpired propagated from SessionManager)
    client.roll(&owner, &session.key_id, &6, &5, &proof);
}

// ─── Additional coverage ─────────────────────────────────────────────────────

/// An invalid proof (out-of-range roll result) returns false, not a panic.
/// Verifies that the ZK gate is still intact and independent of the session
/// gate – the proof-verification path is unchanged from dice_duel.
#[test]
fn roll_returns_false_for_invalid_proof() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, ZkDiceSession);
    let (client, _, proof) = setup(&harness);

    let owner = Address::generate(harness.env());
    let session = client.open_session(&owner, &5, &10_000);

    // roll_result = 7 is out of range for a 6-sided die → verify_dice_roll returns false
    let ok = client.roll(&owner, &session.key_id, &7, &5, &proof);
    assert!(!ok, "out-of-range roll must return false");
}

/// Budget exhaustion: once all session operations are consumed, the next roll
/// is rejected even though the session has not expired.
#[test]
#[should_panic(expected = "session rejected")]
fn roll_fails_when_session_budget_exhausted() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, ZkDiceSession);
    let (client, _, proof) = setup(&harness);

    let owner = Address::generate(harness.env());

    // open_session with budget = 1
    let session = client.open_session(&owner, &1, &10_000);

    // consume the single allowed operation (may return true or false – doesn't matter)
    let _ = client.roll(&owner, &session.key_id, &6, &5, &proof);

    // second roll must be rejected by session budget, not proof verification
    client.roll(&owner, &session.key_id, &6, &6, &proof);
}

/// Multiple players each hold independent sessions; rolls are isolated.
///
/// Both rolls use the same pipeline fixture (roll=6, nonce=5) because the
/// proof is bound to those public inputs.  The important thing is that each
/// player uses their own session key and their `last_roll` records are
/// written independently.
#[test]
fn independent_sessions_for_multiple_players() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, ZkDiceSession);
    let (client, _, proof) = setup(&harness);

    let p1 = Address::generate(harness.env());
    let p2 = Address::generate(harness.env());

    let s1 = client.open_session(&p1, &3, &10_000);
    let s2 = client.open_session(&p2, &3, &10_000);

    // pipeline fixture: roll=6, nonce=5
    assert!(client.roll(&p1, &s1.key_id, &6, &5, &proof));
    assert!(client.roll(&p2, &s2.key_id, &6, &5, &proof));

    // rolls recorded independently per player
    assert_eq!(client.last_roll(&p1).player, p1);
    assert_eq!(client.last_roll(&p2).player, p2);
    assert_eq!(client.last_roll(&p1).roll, 6);
    assert_eq!(client.last_roll(&p2).roll, 6);
}

/// Verify that init_game rejects sides < 2, matching the fair_dice contract.
#[test]
#[should_panic(expected = "invalid game params")]
fn init_game_rejects_single_sided_die() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, ZkDiceSession);
    let client = ZkDiceSessionClient::new(harness.env(), harness.contract_id());
    let seed = test_fixtures::pipeline_public_bytes32(harness.env(), CircuitId::FairDice);

    // sides = 1 must be rejected
    client.init_game(&1, &seed);
}

/// Demonstrate that the circuit spec can be independently validated (the same
/// call path as dice_duel's init_duel_binds_seed_commitment test).
#[test]
fn circuit_spec_matches_fair_dice() {
    let env = Env::default();
    let harness = GameHarness::new(env, ZkDiceSession);
    let seed = test_fixtures::pipeline_public_bytes32(harness.env(), CircuitId::FairDice);

    let spec = fair_dice(harness.env(), 6, &seed).expect("spec must build");
    assert_eq!(spec.circuit_id, CircuitId::FairDice);
}
