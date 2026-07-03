use super::*;
use cougr_core::test::{GameHarness, MockSession};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env,
};

#[test]
fn approve_and_tap_without_reauth() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, SessionArena);
    let owner = Address::generate(harness.env());
    let client = SessionArenaClient::new(harness.env(), harness.contract_id());

    let active = client.approve_session(&owner, &10, &10_000);
    client.tap(&owner, &active.key_id);
    client.tap(&owner, &active.key_id);
    assert_eq!(client.score(&owner), 2);
}

#[test]
fn renew_session_extends_play_window() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, SessionArena);
    let owner = Address::generate(harness.env());
    let client = SessionArenaClient::new(harness.env(), harness.contract_id());

    let active = client.approve_session(&owner, &5, &100);
    let renewed = client.renew_session(&owner, &active.key_id, &20_000);
    assert!(renewed.expires_at > active.expires_at);
}

#[test]
fn fallback_tap_uses_direct_auth_after_session_expires() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, SessionArena);
    let owner = Address::generate(harness.env());
    let client = SessionArenaClient::new(harness.env(), harness.contract_id());

    let active = client.approve_session(&owner, &5, &50);
    harness.env().ledger().with_mut(|li| {
        li.timestamp = 10_000;
    });
    let taps = client.fallback_tap(&owner, &active.key_id);
    assert_eq!(taps, 1);
}

#[test]
fn mock_session_helper_matches_manager_flow() {
    let env = Env::default();
    let harness = GameHarness::new(env, SessionArena);
    let owner = Address::generate(harness.env());

    harness.as_contract(|| {
        let mock = MockSession::approve(harness.env(), &owner, &["tap"], 3, 5_000);
        let status = mock.status(harness.env());
        assert!(status.active);
        assert_eq!(status.remaining_operations, 3);
    });
}
