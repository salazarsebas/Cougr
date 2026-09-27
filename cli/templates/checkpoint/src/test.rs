//! Integration tests for {{crate_name}}.
//!
//! These run through `cougr_core::test::GameHarness`, the sandbox the canonical
//! examples use: it registers the contract in a fresh `Env` and hands back the
//! contract ID for the generated client.
//!
//! Tests cover:
//!   - a valid checkpoint commit (happy path)
//!   - a dispute that wins (challenger hash differs, within window)
//!   - a late dispute that is rejected (window expired)
//!   - state consistency across the full match lifecycle

use crate::components::{DISPUTE_WINDOW_LEDGERS, STATUS_DISPUTED, STATUS_FINALISED, STATUS_RUNNING};
use crate::systems::{
    validate_commit, validate_dispute, validate_finalize, CommitError, DisputeError, FinalizeError,
};
use crate::{{{ContractName}}, {{ContractName}}Client, MatchRecord};
use crate::components::{CheckpointState, MatchConfig, STATUS_RUNNING as SR};
use cougr_core::test::GameHarness;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol_short, Address, Env};

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn started_match() -> (GameHarness, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, {{ContractName}});
    let player = Address::generate(harness.env());
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());
    client.start_match(&player);
    (harness, player)
}

// ─── start_match ─────────────────────────────────────────────────────────────

#[test]
fn start_match_initialises_running_state() {
    let (harness, player) = started_match();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let state = client.get_state();
    assert_eq!(state.player, player);
    assert_eq!(state.last_tick, 0);
    assert_eq!(state.last_hash, 0);
    assert_eq!(state.last_score, 0);
    assert_eq!(state.status, STATUS_RUNNING);
}

// ─── commit_checkpoint ───────────────────────────────────────────────────────

#[test]
fn valid_checkpoint_is_accepted() {
    let (harness, player) = started_match();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let result = client.commit_checkpoint(&player, &100, &0xdeadbeef_u64, &42);

    assert!(result.success, "commit was rejected: {:?}", result.message);
    assert_eq!(result.message, symbol_short!("ok"));
    let state = result.match_state;
    assert_eq!(state.last_tick, 100);
    assert_eq!(state.last_hash, 0xdeadbeef_u64);
    assert_eq!(state.last_score, 42);
    assert_eq!(state.status, STATUS_RUNNING);
}

#[test]
fn multiple_checkpoints_advance_the_state() {
    let (harness, player) = started_match();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    client.commit_checkpoint(&player, &100, &0x1111_u64, &10);
    let result = client.commit_checkpoint(&player, &200, &0x2222_u64, &20);

    assert!(result.success);
    assert_eq!(result.match_state.last_tick, 200);
    assert_eq!(result.match_state.last_score, 20);
}

#[test]
fn non_player_cannot_commit() {
    let (harness, _player) = started_match();
    let stranger = Address::generate(harness.env());
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    let result = client.commit_checkpoint(&stranger, &100, &0xabcd_u64, &5);
    assert!(!result.success);
    assert_eq!(result.message, symbol_short!("notplay"));
}

#[test]
fn tick_must_be_monotone() {
    let (harness, player) = started_match();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    client.commit_checkpoint(&player, &100, &0xaaaa_u64, &5);

    // Same tick is not strictly greater.
    let same = client.commit_checkpoint(&player, &100, &0xbbbb_u64, &6);
    assert!(!same.success);
    assert_eq!(same.message, symbol_short!("oldtick"));

    // Earlier tick is rejected too.
    let earlier = client.commit_checkpoint(&player, &50, &0xcccc_u64, &7);
    assert!(!earlier.success);
    assert_eq!(earlier.message, symbol_short!("oldtick"));
}

// ─── dispute_checkpoint ──────────────────────────────────────────────────────

#[test]
fn dispute_within_window_with_different_hash_wins() {
    let (harness, player) = started_match();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    client.commit_checkpoint(&player, &100, &0xaaaa_u64, &5);

    // Challenger provides a different hash for the same tick within the window.
    let challenger = Address::generate(harness.env());
    let result = client.dispute_checkpoint(&challenger, &100, &0xbbbb_u64);

    assert!(result.success, "dispute was rejected: {:?}", result.message);
    assert_eq!(result.message, symbol_short!("disputed"));
    assert_eq!(result.match_state.status, STATUS_DISPUTED);
}

#[test]
fn dispute_after_window_is_rejected() {
    let (harness, player) = started_match();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    client.commit_checkpoint(&player, &100, &0xaaaa_u64, &5);

    // Jump past the dispute window.
    harness.env().ledger().with_mut(|l| {
        l.sequence_number += DISPUTE_WINDOW_LEDGERS + 1;
    });

    let challenger = Address::generate(harness.env());
    let result = client.dispute_checkpoint(&challenger, &100, &0xbbbb_u64);

    assert!(!result.success);
    assert_eq!(result.message, symbol_short!("toolate"));
    // Match is still running — the late dispute did not change state.
    assert_eq!(result.match_state.status, STATUS_RUNNING);
}

#[test]
fn dispute_with_matching_hash_is_rejected() {
    let (harness, player) = started_match();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    client.commit_checkpoint(&player, &100, &0xaaaa_u64, &5);

    let challenger = Address::generate(harness.env());
    // Same hash as committed — nothing to dispute.
    let result = client.dispute_checkpoint(&challenger, &100, &0xaaaa_u64);

    assert!(!result.success);
    assert_eq!(result.message, symbol_short!("nohash"));
}

#[test]
fn dispute_wrong_tick_is_rejected() {
    let (harness, player) = started_match();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    client.commit_checkpoint(&player, &100, &0xaaaa_u64, &5);

    let challenger = Address::generate(harness.env());
    let result = client.dispute_checkpoint(&challenger, &200, &0xbbbb_u64);

    assert!(!result.success);
    assert_eq!(result.message, symbol_short!("badtick"));
}

// ─── finalize_match ───────────────────────────────────────────────────────────

#[test]
fn match_finalises_after_window_expires() {
    let (harness, player) = started_match();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    client.commit_checkpoint(&player, &100, &0xaaaa_u64, &99);

    // Jump past the dispute window.
    harness.env().ledger().with_mut(|l| {
        l.sequence_number += DISPUTE_WINDOW_LEDGERS + 1;
    });

    let result = client.finalize_match(&player);
    assert!(result.success, "finalise was rejected: {:?}", result.message);
    assert_eq!(result.match_state.status, STATUS_FINALISED);
    assert_eq!(result.match_state.last_score, 99);
}

#[test]
fn finalize_inside_window_is_rejected() {
    let (harness, player) = started_match();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    client.commit_checkpoint(&player, &100, &0xaaaa_u64, &99);

    // Still within the window.
    let result = client.finalize_match(&player);
    assert!(!result.success);
    assert_eq!(result.message, symbol_short!("window"));
    assert_eq!(result.match_state.status, STATUS_RUNNING);
}

#[test]
fn second_checkpoint_after_finalize_is_rejected() {
    let (harness, player) = started_match();
    let client = {{ContractName}}Client::new(harness.env(), harness.contract_id());

    client.commit_checkpoint(&player, &100, &0xaaaa_u64, &5);
    harness.env().ledger().with_mut(|l| {
        l.sequence_number += DISPUTE_WINDOW_LEDGERS + 1;
    });
    client.finalize_match(&player);

    let result = client.commit_checkpoint(&player, &200, &0xbbbb_u64, &10);
    assert!(!result.success);
    assert_eq!(result.message, symbol_short!("notrun"));
}

// ─── Pure systems unit tests ─────────────────────────────────────────────────

fn running_record(env: &Env, player: &Address) -> MatchRecord {
    MatchRecord {
        last_hash: 0xaaaa,
        last_score: 5,
        last_tick: 100,
        committed_at_ledger: 50,
        status: SR,
        authority: player.clone(),
    }
}

#[test]
fn systems_validate_commit_rejects_old_tick() {
    let env = Env::default();
    let player = Address::generate(&env);
    let record = running_record(&env, &player);
    let cp = CheckpointState {
        last_committed_tick: 100,
    };
    assert_eq!(
        validate_commit(&record, &cp, 100),
        Err(CommitError::TickNotMonotone)
    );
    assert_eq!(
        validate_commit(&record, &cp, 50),
        Err(CommitError::TickNotMonotone)
    );
    assert!(validate_commit(&record, &cp, 101).is_ok());
}

#[test]
fn systems_validate_dispute_detects_closed_window() {
    let env = Env::default();
    let player = Address::generate(&env);
    let record = running_record(&env, &player);
    // Window closed: current_ledger >= committed_at + DISPUTE_WINDOW_LEDGERS.
    let closed_ledger = 50 + DISPUTE_WINDOW_LEDGERS;
    assert_eq!(
        validate_dispute(&record, 100, closed_ledger, 0xbbbb),
        Err(DisputeError::WindowClosed)
    );
    // Window still open.
    assert!(validate_dispute(&record, 100, 51, 0xbbbb).is_ok());
}

#[test]
fn systems_validate_finalize_requires_closed_window() {
    let env = Env::default();
    let player = Address::generate(&env);
    let record = running_record(&env, &player);
    // Window open.
    assert_eq!(
        validate_finalize(&record, 51),
        Err(FinalizeError::WindowStillOpen)
    );
    // Window closed.
    assert!(validate_finalize(&record, 50 + DISPUTE_WINDOW_LEDGERS).is_ok());
}
