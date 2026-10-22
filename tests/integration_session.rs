//! Integration tests for `cougr_core::session`.

#![cfg(feature = "testutils")]

use cougr_core::accounts::{
    AccountError, AuthMethod, GameAction, ReplayProtection, SessionStorage, SignedIntent,
};
use cougr_core::session::{SessionManager, RENEWAL_HINT_SECONDS};
use cougr_core::test::MockSession;
use soroban_sdk::{
    contract, contractimpl, symbol_short,
    testutils::{Address as _, Ledger as _},
    Address, Bytes, BytesN, Env, Symbol,
};
/// Thin contract used as the invocation frame for all session tests.
///
/// Each method that may touch `require_auth` (directly or via a fallback path)
/// must live here so Soroban has a proper contract frame in place.
#[contract]
#[derive(Clone)]
pub struct SessionArena;

#[contractimpl]
impl SessionArena {
    // ------------------------------------------------------------------
    // fallback_execute helpers
    //
    // `require_auth` panics outside a contract frame, so every test that
    // exercises the direct-auth fallback path calls one of these methods
    // through `SessionArenaClient` rather than raw `env.as_contract`.
    // ------------------------------------------------------------------

    /// Try session auth; fall back to direct owner auth on fallback errors.
    /// Returns the `AuthMethod` discriminant (0 = Direct, 1 = Session).
    pub fn run_fallback(
        env: Env,
        owner: Address,
        session_key_id: BytesN<32>,
        action_name: Symbol,
    ) -> u32 {
        let session =
            SessionStorage::load(&env, &owner, &session_key_id).expect("session must exist");
        let action = GameAction {
            system_name: action_name,
            data: Bytes::new(&env),
        };
        let session_intent = SignedIntent::session(
            &env,
            owner.clone(),
            &session_key_id,
            action.clone(),
            session.next_nonce,
            env.ledger().timestamp().saturating_add(1_000),
        );
        let direct_intent = SignedIntent::direct(
            &env,
            owner.clone(),
            action,
            ReplayProtection::next_account_nonce(&env, &owner),
            env.ledger().timestamp().saturating_add(1_000),
        );
        let result = SessionManager::fallback_execute(&env, &session_intent, &direct_intent)
            .expect("fallback_execute must not error");
        result.method as u32
    }

    /// Same as `run_fallback` but the session key is no longer in storage
    /// (it was revoked before the call), so we build the session intent from
    /// the supplied nonce rather than reading from storage.
    pub fn run_fallback_revoked(
        env: Env,
        owner: Address,
        session_key_id: BytesN<32>,
        action_name: Symbol,
        session_nonce: u64,
    ) -> u32 {
        let action = GameAction {
            system_name: action_name,
            data: Bytes::new(&env),
        };
        let session_intent = SignedIntent::session(
            &env,
            owner.clone(),
            &session_key_id,
            action.clone(),
            session_nonce,
            env.ledger().timestamp().saturating_add(1_000),
        );
        let direct_intent = SignedIntent::direct(
            &env,
            owner.clone(),
            action,
            ReplayProtection::next_account_nonce(&env, &owner),
            env.ledger().timestamp().saturating_add(1_000),
        );
        let result = SessionManager::fallback_execute(&env, &session_intent, &direct_intent)
            .expect("fallback_execute must not error");
        result.method as u32
    }

    /// Run fallback with a deliberately wrong session nonce (to trigger
    /// `NonceMismatch`) and the correct action, so the direct path can succeed.
    pub fn run_fallback_wrong_nonce(
        env: Env,
        owner: Address,
        session_key_id: BytesN<32>,
        action_name: Symbol,
        bad_nonce: u64,
    ) -> u32 {
        let action = GameAction {
            system_name: action_name,
            data: Bytes::new(&env),
        };
        let session_intent = SignedIntent::session(
            &env,
            owner.clone(),
            &session_key_id,
            action.clone(),
            bad_nonce,
            env.ledger().timestamp().saturating_add(1_000),
        );
        let direct_intent = SignedIntent::direct(
            &env,
            owner.clone(),
            action,
            ReplayProtection::next_account_nonce(&env, &owner),
            env.ledger().timestamp().saturating_add(1_000),
        );
        let result = SessionManager::fallback_execute(&env, &session_intent, &direct_intent)
            .expect("fallback_execute must not error");
        result.method as u32
    }

    /// Run fallback where the session intent has a corrupted hash, producing
    /// `InvalidIntent` — a non-fallback error that must propagate.
    /// Returns 99 on success (should never be reached), or panics to surface
    /// the error to the caller.
    pub fn run_fallback_bad_hash(
        env: Env,
        owner: Address,
        session_key_id: BytesN<32>,
        action_name: Symbol,
    ) -> u32 {
        let session =
            SessionStorage::load(&env, &owner, &session_key_id).expect("session must exist");
        let action = GameAction {
            system_name: action_name,
            data: Bytes::new(&env),
        };
        let mut session_intent = SignedIntent::session(
            &env,
            owner.clone(),
            &session_key_id,
            action.clone(),
            session.next_nonce,
            env.ledger().timestamp().saturating_add(1_000),
        );
        // Corrupt the hash — this produces `InvalidIntent`, not a fallback error.
        session_intent.action_hash = BytesN::from_array(&env, &[0x00u8; 32]);

        let direct_intent = SignedIntent::direct(
            &env,
            owner.clone(),
            action,
            ReplayProtection::next_account_nonce(&env, &owner),
            env.ledger().timestamp().saturating_add(1_000),
        );
        // This must return Err(InvalidIntent); we panic so the test catches the
        // contract error rather than a return value.
        SessionManager::fallback_execute(&env, &session_intent, &direct_intent)
            .expect("MUST_NOT_REACH");
        99
    }
}

fn make_action(env: &Env, name: &str) -> GameAction {
    GameAction {
        system_name: Symbol::new(env, name),
        data: Bytes::new(env),
    }
}

#[test]
fn session_manager_approve_and_status() {
    let env = Env::default();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let mock = MockSession::approve(&env, &owner, &["tap"], 5, 10_000);
        let status = SessionManager::status(&env, &owner, mock.key_id()).unwrap();
        assert!(status.active);
        assert_eq!(status.remaining_operations, 5);
        assert!(!status.needs_renewal);
    });
}

#[test]
fn session_manager_execute_action_consumes_budget() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let mock = MockSession::approve(&env, &owner, &["tap"], 2, 10_000);
        let action = make_action(&env, "tap");
        let result = SessionManager::execute_action(
            &env,
            &owner,
            &mock.key,
            action.clone(),
            env.ledger().timestamp() + 1000,
        )
        .unwrap();
        assert_eq!(result.method, AuthMethod::Session);
        assert_eq!(result.remaining_operations, 1);

        let status = SessionManager::status(&env, &owner, mock.key_id()).unwrap();
        assert_eq!(status.remaining_operations, 1);
    });
}

#[test]
fn session_manager_renew_extends_expiry() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let mock = MockSession::approve(&env, &owner, &["tap"], 5, 100);
        let renewed = SessionManager::renew(&env, &owner, mock.key_id(), 50_000).unwrap();
        assert_eq!(renewed.scope.expires_at, 50_000);
    });
}

#[test]
fn session_status_flags_renewal_when_near_expiry() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let mock = MockSession::approve(&env, &owner, &["tap"], 5, RENEWAL_HINT_SECONDS);
        let status = SessionManager::status(&env, &owner, mock.key_id()).unwrap();
        assert!(status.needs_renewal);
    });
}

#[test]
fn session_execute_fails_when_expired() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let mock = MockSession::approve(&env, &owner, &["tap"], 5, 50);
        env.ledger().with_mut(|li| {
            li.timestamp = 10_000;
        });

        let action = make_action(&env, "tap");
        let err =
            SessionManager::execute_action(&env, &owner, &mock.key, action, 20_000).unwrap_err();
        assert_eq!(err, AccountError::SessionExpired);
    });
}

#[test]
fn session_builder_expires_in_sets_relative_deadline() {
    let env = Env::default();
    env.ledger().with_mut(|li| {
        li.timestamp = 1_000;
    });
    let scope = cougr_core::accounts::SessionBuilder::new(&env)
        .allow_action(symbol_short!("move"))
        .max_operations(10)
        .expires_in(500)
        .build_scope();
    assert_eq!(scope.expires_at, 1_500);
}

#[test]
fn renew_unknown_session_returns_revoked_error() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);
    let missing = soroban_sdk::BytesN::from_array(&env, &[9u8; 32]);

    env.as_contract(&contract_id, || {
        let err = SessionManager::renew(&env, &owner, &missing, 9_999).unwrap_err();
        assert_eq!(err, AccountError::SessionRevoked);
    });
}

// ---------------------------------------------------------------------------
// Helpers (new tests)
// ---------------------------------------------------------------------------

// Discriminant values for AuthMethod (must match the #[repr(u32)] enum).
const AUTH_DIRECT: u32 = 0;
const AUTH_SESSION: u32 = 1;

// ---------------------------------------------------------------------------
// approve_with_builder
// ---------------------------------------------------------------------------

#[test]
fn approve_with_builder_creates_active_session() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);

    env.as_contract(&contract_id, || {
        // approve_with_builder is what MockSession::approve delegates to, so we
        // call it directly here to give it its own named test.
        let builder = cougr_core::accounts::SessionBuilder::new(&env)
            .allow_action(symbol_short!("tap"))
            .max_operations(3)
            .expires_in(5_000);

        let key = SessionManager::approve_with_builder(&env, &owner, builder).unwrap();

        // The returned key must be usable immediately.
        let status = SessionManager::status(&env, &owner, &key.key_id).unwrap();
        assert!(
            status.active,
            "session must be active after approve_with_builder"
        );
        assert_eq!(status.remaining_operations, 3);
        assert_eq!(key.operations_used, 0);
        assert_eq!(key.next_nonce, 0);
    });
}

// ---------------------------------------------------------------------------
// execute_intent
// ---------------------------------------------------------------------------

#[test]
fn execute_intent_routes_session_through_kernel() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let mock = MockSession::approve(&env, &owner, &["tap"], 5, 10_000);
        let action = make_action(&env, "tap");

        // Build the intent manually the same way execute_action does internally.
        let intent = SignedIntent::session(
            &env,
            owner.clone(),
            mock.key_id(),
            action,
            mock.key.next_nonce,
            env.ledger().timestamp() + 1_000,
        );

        let result = SessionManager::execute_intent(&env, &intent).unwrap();
        assert_eq!(result.method, AuthMethod::Session);
        assert_eq!(result.remaining_operations, 4);
    });
}

#[test]
fn execute_intent_with_tampered_hash_returns_invalid_intent() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let mock = MockSession::approve(&env, &owner, &["tap"], 5, 10_000);
        let action = make_action(&env, "tap");

        let mut intent = SignedIntent::session(
            &env,
            owner.clone(),
            mock.key_id(),
            action,
            mock.key.next_nonce,
            env.ledger().timestamp() + 1_000,
        );

        // Corrupt the hash so the kernel rejects it.
        intent.action_hash = soroban_sdk::BytesN::from_array(&env, &[0xffu8; 32]);

        let err = SessionManager::execute_intent(&env, &intent).unwrap_err();
        assert_eq!(err, AccountError::InvalidIntent);
    });
}

// ---------------------------------------------------------------------------
// revoke
// ---------------------------------------------------------------------------

#[test]
fn revoke_makes_session_unfindable() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let mock = MockSession::approve(&env, &owner, &["tap"], 5, 10_000);

        // Session is active before revoke.
        let pre = SessionManager::status(&env, &owner, mock.key_id()).unwrap();
        assert!(pre.active);

        // Revoke succeeds.
        SessionManager::revoke(&env, &owner, mock.key_id()).unwrap();

        // status now returns SessionRevoked (key absent from storage).
        let err = SessionManager::status(&env, &owner, mock.key_id()).unwrap_err();
        assert_eq!(err, AccountError::SessionRevoked);
    });
}

#[test]
fn revoke_then_execute_action_returns_session_revoked() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let mock = MockSession::approve(&env, &owner, &["tap"], 5, 10_000);

        // Revoke the key.
        SessionManager::revoke(&env, &owner, mock.key_id()).unwrap();

        // Attempting to execute an action with the revoked key must fail.
        let action = make_action(&env, "tap");
        let err = SessionManager::execute_action(
            &env,
            &owner,
            &mock.key,
            action,
            env.ledger().timestamp() + 1_000,
        )
        .unwrap_err();
        assert_eq!(err, AccountError::SessionRevoked);
    });
}

#[test]
fn revoke_nonexistent_key_returns_invalid_scope() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);
    let missing_id = soroban_sdk::BytesN::from_array(&env, &[0xABu8; 32]);

    env.as_contract(&contract_id, || {
        let err = SessionManager::revoke(&env, &owner, &missing_id).unwrap_err();
        // ContractAccount::revoke_session returns InvalidScope when the key is absent.
        assert_eq!(err, AccountError::InvalidScope);
    });
}

// ---------------------------------------------------------------------------
// cleanup_expired
// ---------------------------------------------------------------------------

#[test]
fn cleanup_expired_removes_expired_keys_and_preserves_live_ones() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);

    env.as_contract(&contract_id, || {
        // Create two sessions: one expires at t=50, one at t=99_999.
        let _short = MockSession::approve(&env, &owner, &["tap"], 5, 50);
        let long = MockSession::approve(&env, &owner, &["tap"], 5, 99_999);

        // Advance time past the short session's expiry.
        env.ledger().with_mut(|l| l.timestamp = 1_000);

        let removed = SessionManager::cleanup_expired(&env, &owner);
        assert_eq!(removed, 1, "exactly one expired session should be removed");

        // The long-lived session must still be present.
        let status = SessionManager::status(&env, &owner, long.key_id()).unwrap();
        assert!(status.active, "live session must survive cleanup");
    });
}

#[test]
fn cleanup_expired_returns_zero_when_nothing_is_expired() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let _mock = MockSession::approve(&env, &owner, &["tap"], 5, 99_999);

        // No time advance — session is still live.
        let removed = SessionManager::cleanup_expired(&env, &owner);
        assert_eq!(removed, 0);
    });
}

#[test]
fn cleanup_expired_returns_zero_for_owner_with_no_sessions() {
    let env = Env::default();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let removed = SessionManager::cleanup_expired(&env, &owner);
        assert_eq!(removed, 0);
    });
}

// ---------------------------------------------------------------------------
// fallback_execute — success path
// ---------------------------------------------------------------------------

#[test]
fn fallback_execute_succeeds_via_session_when_session_is_valid() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);
    let client = SessionArenaClient::new(&env, &contract_id);

    // Approve inside the contract context.
    env.as_contract(&contract_id, || {
        MockSession::approve(&env, &owner, &["tap"], 5, 10_000);
    });
    // Read the key_id back so we can pass it to the contract method.
    let key_id = env.as_contract(&contract_id, || {
        cougr_core::accounts::SessionStorage::load_all(&env, &owner)
            .get(0)
            .unwrap()
            .key_id
    });

    let method = client.run_fallback(&owner, &key_id, &Symbol::new(&env, "tap"));
    assert_eq!(
        method, AUTH_SESSION,
        "valid session must not fall back to direct"
    );
}

// ---------------------------------------------------------------------------
// fallback_execute — SessionExpired triggers direct fallback
// ---------------------------------------------------------------------------

#[test]
fn fallback_execute_falls_back_on_session_expired() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);
    let client = SessionArenaClient::new(&env, &contract_id);

    let key_id = env.as_contract(&contract_id, || {
        // Session expires 50 seconds from now (ledger timestamp 0).
        MockSession::approve(&env, &owner, &["tap"], 5, 50);
        cougr_core::accounts::SessionStorage::load_all(&env, &owner)
            .get(0)
            .unwrap()
            .key_id
    });

    // Advance past expiry.
    env.ledger().with_mut(|l| l.timestamp = 10_000);

    let method = client.run_fallback(&owner, &key_id, &Symbol::new(&env, "tap"));
    assert_eq!(
        method, AUTH_DIRECT,
        "SessionExpired must trigger direct-auth fallback"
    );
}

// ---------------------------------------------------------------------------
// fallback_execute — SessionBudgetExceeded triggers direct fallback
// ---------------------------------------------------------------------------

#[test]
fn fallback_execute_falls_back_on_session_budget_exceeded() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);
    let client = SessionArenaClient::new(&env, &contract_id);

    let key_id = env.as_contract(&contract_id, || {
        // Budget of exactly 1.
        let mock = MockSession::approve(&env, &owner, &["tap"], 1, 10_000);
        // Exhaust the single operation.
        let action = make_action(&env, "tap");
        SessionManager::execute_action(
            &env,
            &owner,
            &mock.key,
            action,
            env.ledger().timestamp() + 1_000,
        )
        .unwrap();
        mock.key.key_id
    });

    // Budget is now 0; the next call via the contract must fall back to direct.
    let method = client.run_fallback(&owner, &key_id, &Symbol::new(&env, "tap"));
    assert_eq!(
        method, AUTH_DIRECT,
        "SessionBudgetExceeded must trigger direct-auth fallback"
    );
}

// ---------------------------------------------------------------------------
// fallback_execute — SessionRevoked triggers direct fallback
// ---------------------------------------------------------------------------

#[test]
fn fallback_execute_falls_back_on_session_revoked() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);
    let client = SessionArenaClient::new(&env, &contract_id);

    let (key_id, nonce) = env.as_contract(&contract_id, || {
        let mock = MockSession::approve(&env, &owner, &["tap"], 5, 10_000);
        let nonce = mock.key.next_nonce;
        // Revoke immediately.
        SessionManager::revoke(&env, &owner, &mock.key.key_id).unwrap();
        (mock.key.key_id, nonce)
    });

    // The key is gone; run_fallback_revoked builds the session intent from the
    // supplied nonce rather than reading it from storage.
    let method = client.run_fallback_revoked(&owner, &key_id, &Symbol::new(&env, "tap"), &nonce);
    assert_eq!(
        method, AUTH_DIRECT,
        "SessionRevoked must trigger direct-auth fallback"
    );
}

// ---------------------------------------------------------------------------
// fallback_execute — NonceMismatch triggers direct fallback
// ---------------------------------------------------------------------------

#[test]
fn fallback_execute_falls_back_on_nonce_mismatch() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);
    let client = SessionArenaClient::new(&env, &contract_id);

    let key_id = env.as_contract(&contract_id, || {
        let mock = MockSession::approve(&env, &owner, &["tap"], 5, 10_000);
        mock.key.key_id
    });

    // next_nonce is 0; supply 99 to trigger NonceMismatch.
    let bad_nonce: u64 = 99;
    let method =
        client.run_fallback_wrong_nonce(&owner, &key_id, &Symbol::new(&env, "tap"), &bad_nonce);
    assert_eq!(
        method, AUTH_DIRECT,
        "NonceMismatch must trigger direct-auth fallback"
    );
}

// ---------------------------------------------------------------------------
// fallback_execute — ActionNotAllowed triggers direct fallback
// ---------------------------------------------------------------------------

#[test]
fn fallback_execute_falls_back_on_action_not_allowed() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);
    let client = SessionArenaClient::new(&env, &contract_id);

    // Session only permits "tap"; we send "attack" via the session path.
    let key_id = env.as_contract(&contract_id, || {
        let mock = MockSession::approve(&env, &owner, &["tap"], 5, 10_000);
        mock.key.key_id
    });

    let method = client.run_fallback(&owner, &key_id, &Symbol::new(&env, "attack"));
    assert_eq!(
        method, AUTH_DIRECT,
        "ActionNotAllowed must trigger direct-auth fallback"
    );
}

// ---------------------------------------------------------------------------
// fallback_execute — non-fallback error is propagated (not swallowed)
// ---------------------------------------------------------------------------

#[test]
fn fallback_execute_propagates_non_fallback_error_without_trying_direct() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);
    let client = SessionArenaClient::new(&env, &contract_id);

    let key_id = env.as_contract(&contract_id, || {
        let mock = MockSession::approve(&env, &owner, &["tap"], 5, 10_000);
        mock.key.key_id
    });

    // run_fallback_bad_hash corrupts the session intent hash → InvalidIntent.
    // The contract method panics on the `expect("MUST_NOT_REACH")`, which the
    // Soroban test env surfaces as an Err from the client call.
    let result = client.try_run_fallback_bad_hash(&owner, &key_id, &Symbol::new(&env, "tap"));
    assert!(
        result.is_err(),
        "InvalidIntent must propagate as an error, not fall through to direct auth"
    );
}
