//! ECS components for {{crate_name}}.
//!
//! `MatchConfig` and `MatchRecord` hold `Address` fields and are declared with
//! `impl_rich_component!`, which gives them an XDR codec that survives round
//! trips through Soroban instance storage. `CheckpointState` is all fixed-size
//! scalars, so the cheaper `impl_component!` is sufficient.

use cougr_core::{impl_component, impl_rich_component};
use soroban_sdk::{contracttype, Address};

/// The single entity all match state hangs off.
///
/// One match per contract instance; a fixed entity ID is simpler than a dynamic
/// population.
pub const MATCH_ENTITY: u32 = 1;

/// How many ledgers a checkpoint stays open for dispute after it is committed.
///
/// ~1 000 ledgers ≈ 83 minutes at the Stellar average of ~5 s/ledger.
/// Adjust for your game's pace before deploying.
pub const DISPUTE_WINDOW_LEDGERS: u32 = 1_000;

// ─── Match status ─────────────────────────────────────────────────────────────

pub const STATUS_RUNNING: u32 = 0;
pub const STATUS_FINALISED: u32 = 1;
pub const STATUS_DISPUTED: u32 = 2;

// ─── Components ───────────────────────────────────────────────────────────────

/// Immutable match parameters set at `start_match`.
#[contracttype]
#[derive(Clone, Debug)]
pub struct MatchConfig {
    /// The address that is authorised to commit checkpoints.
    pub player: Address,
    /// Ledger sequence when the match opened.
    pub started_at_ledger: u32,
}

impl_rich_component!(MatchConfig, "mcfg");

/// The latest committed checkpoint and match outcome.
#[contracttype]
#[derive(Clone, Debug)]
pub struct MatchRecord {
    /// State hash committed at the most recent checkpoint, or 0 before the
    /// first commit.
    pub last_hash: u64,
    /// Score committed at the most recent checkpoint.
    pub last_score: u32,
    /// Tick index of the most recent checkpoint.
    pub last_tick: u32,
    /// Ledger sequence at which the most recent checkpoint was committed.
    pub committed_at_ledger: u32,
    /// One of `STATUS_RUNNING`, `STATUS_FINALISED`, `STATUS_DISPUTED`.
    pub status: u32,
    /// The challenger's address if the match is disputed, otherwise the player.
    pub authority: Address,
}

impl_rich_component!(MatchRecord, "mrec");

/// The monotone tick counter used for ordering validation.
///
/// Stored separately because it is fixed-size and can use the cheaper codec.
#[contracttype]
#[derive(Clone, Debug)]
pub struct CheckpointState {
    /// Tick index of the last committed checkpoint.
    pub last_committed_tick: u32,
}

impl_component!(CheckpointState, "cpst", Table, {
    last_committed_tick: u32
});
