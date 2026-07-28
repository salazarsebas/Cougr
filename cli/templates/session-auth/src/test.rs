//! Integration tests for {{crate_name}}.
//!
//! These run through `cougr_core::test::GameHarness`, the sandbox the canonical
//! examples use. `env.mock_all_auths()` stands in for the one wallet prompt a
//! real owner would answer at `approve_session`; every `tap` after that is
//! authorized by the session key, not the wallet.

use crate::components::Score;
use crate::systems::{validate_request, SessionPolicyError, MAX_SESSION_LIFETIME};
use crate::{{ContractName}};
use crate::{{ContractName}}Client;
use cougr_core::test::{GameHarness, MockSession};
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{Address, Env};

const TAP_BUDGET: u32 = 10;
const SESSION_LIFETIME: u64 = 10_000;

/// A harness with mocked auth and one owner address.
fn arena() -> (GameHarness, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, {{ContractName}});
    let owner = Address::generate(harness.env());

    (harness, owner)
}

#[test]
fn approving_a_session_reports_its_budget() {
    let (harness, owner) = arena();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let session = client.approve_session(&owner, &TAP_BUDGET, &SESSION_LIFETIME);

    assert_eq!(session.operations_remaining, TAP_BUDGET);
    assert!(!session.needs_renewal);
    assert!(session.expires_at >= SESSION_LIFETIME);
}

#[test]
fn taps_need_no_further_owner_approval() {
    let (harness, owner) = arena();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let session = client.approve_session(&owner, &TAP_BUDGET, &SESSION_LIFETIME);
    client.tap(&owner, &session.key_id);
    client.tap(&owner, &session.key_id);
    client.tap(&owner, &session.key_id);

    assert_eq!(client.score(&owner), 3);
}

#[test]
fn scores_are_tracked_per_player() {
    let (harness, owner) = arena();
    let other = Address::generate(harness.env());
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let owner_session = client.approve_session(&owner, &TAP_BUDGET, &SESSION_LIFETIME);
    let other_session = client.approve_session(&other, &TAP_BUDGET, &SESSION_LIFETIME);

    client.tap(&owner, &owner_session.key_id);
    client.tap(&other, &other_session.key_id);
    client.tap(&other, &other_session.key_id);

    assert_eq!(client.score(&owner), 1);
    assert_eq!(client.score(&other), 2);
}

#[test]
fn an_unknown_player_has_no_score() {
    let (harness, _) = arena();
    let stranger = Address::generate(harness.env());
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    assert_eq!(client.score(&stranger), 0);
}

#[test]
fn session_state_tracks_the_remaining_budget() {
    let (harness, owner) = arena();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let session = client.approve_session(&owner, &TAP_BUDGET, &SESSION_LIFETIME);
    client.tap(&owner, &session.key_id);

    let state = client.session_state(&owner, &session.key_id).unwrap();
    assert!(state.operations_remaining < TAP_BUDGET);
}

#[test]
fn renewing_extends_the_play_window() {
    let (harness, owner) = arena();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let session = client.approve_session(&owner, &TAP_BUDGET, &100);
    let renewed = client.renew_session(&owner, &session.key_id, &20_000);

    assert!(renewed.expires_at > session.expires_at);
    assert_eq!(renewed.key_id, session.key_id);
}

#[test]
fn fallback_tap_keeps_playing_after_the_session_expires() {
    let (harness, owner) = arena();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let session = client.approve_session(&owner, &TAP_BUDGET, &50);
    harness.env().ledger().with_mut(|ledger| {
        ledger.timestamp = 10_000;
    });

    assert_eq!(client.fallback_tap(&owner, &session.key_id), 1);
}

#[test]
#[should_panic(expected = "tap budget out of range")]
fn a_zero_tap_budget_is_rejected() {
    let (harness, owner) = arena();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    client.approve_session(&owner, &0, &SESSION_LIFETIME);
}

#[test]
#[should_panic(expected = "session lifetime out of range")]
fn an_overlong_session_is_rejected() {
    let (harness, owner) = arena();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    client.approve_session(&owner, &TAP_BUDGET, &(MAX_SESSION_LIFETIME + 1));
}

#[test]
fn mock_session_matches_the_manager_flow() {
    let (harness, owner) = arena();

    harness.as_contract(|| {
        let mock = MockSession::approve(harness.env(), &owner, &["tap"], 3, 5_000);
        let status = mock.status(harness.env());

        assert!(status.active);
        assert_eq!(status.remaining_operations, 3);
    });
}

#[test]
fn validate_request_rejects_an_empty_budget() {
    assert_eq!(
        validate_request(0, SESSION_LIFETIME),
        Err(SessionPolicyError::BudgetOutOfRange)
    );
}

#[test]
fn score_component_defaults_to_zero_taps() {
    assert_eq!(Score { taps: 0 }.taps, 0);
}
