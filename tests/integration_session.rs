//! Integration tests for `cougr_core::session`.

#![cfg(feature = "testutils")]

use cougr_core::accounts::{AccountError, AuthMethod, GameAction};
use cougr_core::session::{SessionManager, RENEWAL_HINT_SECONDS};
use cougr_core::test::MockSession;
use soroban_sdk::{
    contract, contractimpl, symbol_short,
    testutils::{Address as _, Ledger as _},
    Address, Bytes, Env, Symbol,
};

#contract]
#[derive(Clone)]
pub struct SessionArena;

#contractimpl]
impl SessionArena {}

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
fn session_execute_fails_when_intent_expired() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SessionArena, ());
    let owner = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let mock = MockSession::approve(&env, &owner, &["tap"], 5, 50_000);
        env.ledger().with_mut(|li| {
            li.timestamp = 10_000;
        });

        let action = make_action(&env, "tap");
        // The session is still live, but the intent's own expiry has passed.
        let err = SessionManager::execute_action(&env, &owner, &mock.key, action, 5_000)
            .unwrap_error();
        assert_eq!(err, AccountError::IntentExpired);
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
        let err = SessionManager::renew(&env, &owner, &missing, 9_999).unwrap_error();
        assert_eq!(err, AccountError::SessionRevoked);
    });
}
