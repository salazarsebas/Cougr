//! Game rules for {{crate_name}}.
//!
//! All rule enforcement is expressed as pure functions over component values.
//! `lib.rs` owns storage; this module owns the rules. Keeping them separate
//! means a rule change never risks touching persistence and each function is
//! unit-testable without an `Env`.

use crate::components::{
    CheckpointState, MatchRecord, STATUS_RUNNING, DISPUTE_WINDOW_LEDGERS,
};

// ─── Error types ──────────────────────────────────────────────────────────────

/// Why a `commit_checkpoint` call was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitError {
    /// The match is not in the `STATUS_RUNNING` state.
    MatchNotRunning,
    /// The tick supplied is not strictly greater than the last committed tick.
    TickNotMonotone,
}

/// Why a `dispute_checkpoint` call was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisputeError {
    /// The match is not in the `STATUS_RUNNING` state.
    MatchNotRunning,
    /// The challenged tick does not match the last committed tick.
    TickMismatch,
    /// The dispute window for this checkpoint has already closed.
    WindowClosed,
    /// The challenger's hash matches the committed hash - nothing to dispute.
    HashMatches,
}

/// Why a `finalize_match` call was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinalizeError {
    /// The match is not in the `STATUS_RUNNING` state.
    MatchNotRunning,
    /// The dispute window for the latest checkpoint has not yet expired.
    WindowStillOpen,
}

// ─── Rule functions ───────────────────────────────────────────────────────────

/// Validate a `commit_checkpoint` request.
///
/// Returns `Ok(())` if the commit should proceed, or a `CommitError`
/// explaining why it should not.
pub fn validate_commit(
    record: &MatchRecord,
    cp: &CheckpointState,
    tick: u32,
) -> Result<(), CommitError> {
    if record.status != STATUS_RUNNING {
        return Err(CommitError::MatchNotRunning);
    }
    if tick <= cp.last_committed_tick {
        return Err(CommitError::TickNotMonotone);
    }
    Ok(())
}

/// Validate a `dispute_checkpoint` request.
///
/// - `challenged_tick` must equal the last committed tick.
/// - `current_ledger` must be within the dispute window.
/// - `claimed_hash` must differ from the committed hash.
pub fn validate_dispute(
    record: &MatchRecord,
    challenged_tick: u32,
    current_ledger: u32,
    claimed_hash: u64,
) -> Result<(), DisputeError> {
    if record.status != STATUS_RUNNING {
        return Err(DisputeError::MatchNotRunning);
    }
    if challenged_tick != record.last_tick {
        return Err(DisputeError::TickMismatch);
    }
    if !window_open(record.committed_at_ledger, current_ledger) {
        return Err(DisputeError::WindowClosed);
    }
    if claimed_hash == record.last_hash {
        return Err(DisputeError::HashMatches);
    }
    Ok(())
}

/// Validate a `finalize_match` request.
pub fn validate_finalize(
    record: &MatchRecord,
    current_ledger: u32,
) -> Result<(), FinalizeError> {
    if record.status != STATUS_RUNNING {
        return Err(FinalizeError::MatchNotRunning);
    }
    if window_open(record.committed_at_ledger, current_ledger) {
        return Err(FinalizeError::WindowStillOpen);
    }
    Ok(())
}

/// Whether the dispute window for a checkpoint committed at `committed_ledger`
/// is still open at `current_ledger`.
pub fn window_open(committed_ledger: u32, current_ledger: u32) -> bool {
    current_ledger.saturating_sub(committed_ledger) < DISPUTE_WINDOW_LEDGERS
}
