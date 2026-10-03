//! Integration tests for the account_kernel_counter game.
//!
//! These tests exercise the three paths named in issue #375 **through the
//! game's own contract functions**, not through the piece's unit tests:
//!
//! 1. `in_policy_action_increments_counter`  – allow path
//! 2. `expired_session_blocks_increment`     – expiry path
//! 3. `replayed_nonce_blocks_increment`      – replay path
//!
//! A fourth test (`multiple_increments_drain_budget`) shows the op-budget
//! running to zero, which is the complementary exhaustion case to replay.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, BytesN, Env,
};

// ── helpers ──────────────────────────────────────────────────────────────────

/// Register `env`, create an owner, register a session, and return the
/// (owner, key_id, client) triple ready for game calls.
fn setup(
    env: &Env,
    max_ops: u32,
    session_lifetime: u64,
) -> (Address, BytesN<32>, CounterGameClient<'_>) {
    env.mock_all_auths();
    let contract_id = env.register(CounterGame, ());
    let client = CounterGameClient::new(env, &contract_id);

    let owner = Address::generate(env);
    let key_id = BytesN::from_array(env, &[0xABu8; 32]);
    let expires_at = env.ledger().timestamp().saturating_add(session_lifetime);

    client.register_session(&owner, &key_id, &max_ops, &expires_at);
    (owner, key_id, client)
}

// ── test 1: allow ─────────────────────────────────────────────────────────

/// A valid in-policy action through the game contract succeeds and bumps the
/// counter.  The authorization runs through `account_kernel::authorize_action`
/// (the unmodified piece) → `AccountKernel::authorize_session` → `SessionPolicy`.
#[test]
fn in_policy_action_increments_counter() {
    let env = Env::default();
    let (owner, key_id, client) = setup(&env, 5, 10_000);

    let intent_expires = env.ledger().timestamp().saturating_add(60);

    // First increment with nonce 0.
    let count = client.increment(&owner, &key_id, &0u64, &intent_expires);
    assert_eq!(count, 1);

    // Second increment with nonce 1 (sequential nonces are required by the kernel).
    let count = client.increment(&owner, &key_id, &1u64, &intent_expires);
    assert_eq!(count, 2);

    assert_eq!(client.count(&owner), 2);
}

// ── test 2: expiry ────────────────────────────────────────────────────────

/// When the ledger timestamp advances past `expires_at` the game contract
/// must reject the increment.  The check runs through the kernel's
/// `SessionPolicy` which calls `check_session_expired` internally.
#[test]
#[should_panic]
fn expired_session_blocks_increment() {
    let env = Env::default();
    // Session expires in 50 ledger-seconds.
    let (owner, key_id, client) = setup(&env, 10, 50);

    // Advance the ledger past the session expiry.
    env.ledger().with_mut(|li| {
        li.timestamp = 10_000; // well past the 50-second window
    });

    // This must panic because the session is expired.
    client.increment(&owner, &key_id, &0u64, &(env.ledger().timestamp() + 60));
}

// ── test 3: replay ────────────────────────────────────────────────────────

/// Submitting the same nonce twice must be rejected after the first use.
/// The replay protection is enforced by `SessionStorage::consume_authorized_session`
/// inside the kernel, surfaced through `authorize_action` in the piece.
#[test]
#[should_panic]
fn replayed_nonce_blocks_increment() {
    let env = Env::default();
    let (owner, key_id, client) = setup(&env, 10, 10_000);

    let intent_expires = env.ledger().timestamp().saturating_add(60);

    // First use of nonce 0 succeeds.
    client.increment(&owner, &key_id, &0u64, &intent_expires);

    // Second use of nonce 0 must panic (NonceMismatch from the kernel).
    client.increment(&owner, &key_id, &0u64, &intent_expires);
}

// ── bonus: op-budget exhaustion ───────────────────────────────────────────

/// When max_operations is 3, the fourth increment must be rejected regardless
/// of the nonce being correct.  This exercises the `SessionBudgetExceeded`
/// path in the kernel through the game contract.
#[test]
#[should_panic]
fn budget_exhaustion_blocks_further_increments() {
    let env = Env::default();
    let (owner, key_id, client) = setup(&env, 3, 10_000);

    let intent_expires = env.ledger().timestamp().saturating_add(60);

    client.increment(&owner, &key_id, &0u64, &intent_expires);
    client.increment(&owner, &key_id, &1u64, &intent_expires);
    client.increment(&owner, &key_id, &2u64, &intent_expires);

    // Budget of 3 exhausted – fourth call must panic.
    client.increment(&owner, &key_id, &3u64, &intent_expires);
}

// ── baseline: count without any increments ────────────────────────────────

#[test]
fn count_returns_zero_before_any_increment() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CounterGame, ());
    let client = CounterGameClient::new(&env, &contract_id);
    let owner = Address::generate(&env);

    assert_eq!(client.count(&owner), 0);
}
