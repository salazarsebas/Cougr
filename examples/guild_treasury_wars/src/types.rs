//! Data types for Guild Treasury Wars
//!
//! This module defines all ECS components, storage keys, and constants
//! for the guild governance game. Components follow the cougr-core ECS
//! pattern with Soroban-compatible contracttype annotations.
//!
//! **stellar-zk Integration**: `StrategyCommitment` stores SHA256-based
//! sealed war plans (commit-reveal). `StrategyReveal` holds the preimage
//! used for on-chain verification. This mirrors the nullifier-based
//! anti-replay pattern from stellar-zk's on-chain verifier contracts.

use soroban_sdk::{contracttype, Address, BytesN, String};

// ============================================================================
// ECS Components - Guild
// ============================================================================

/// Guild component - represents an on-chain faction with a shared treasury.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Guild {
    /// Guild administrator (creator)
    pub admin: Address,
    /// Human-readable guild name
    pub name: String,
    /// Current treasury balance (abstract resource units)
    pub treasury: u64,
    /// Number of registered members
    pub member_count: u32,
    /// Ledger sequence when the guild was created
    pub created_at: u64,
    /// Defense strength (accumulated through Defend proposals)
    pub defense_strength: u32,
    /// Attack strength (accumulated through Upgrade proposals)
    pub attack_strength: u32,
}

// ============================================================================
// ECS Components - Governance
// ============================================================================

/// Proposal actions that a guild can vote on.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ProposalAction {
    /// Strengthen guild defenses (costs DEFEND_COST from treasury)
    Defend = 0,
    /// Launch an attack campaign against another guild (costs ATTACK_COST)
    Attack = 1,
    /// Upgrade guild capabilities (costs UPGRADE_COST)
    Upgrade = 2,
    /// Allocate resources to a specific purpose (custom amount)
    Allocate = 3,
}

/// Status of a governance proposal.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ProposalStatus {
    /// Voting is in progress
    Active = 0,
    /// Proposal reached quorum and was approved
    Approved = 1,
    /// Proposal was rejected by majority
    Rejected = 2,
    /// Proposal was executed and treasury action applied
    Executed = 3,
}

/// Proposal input - data submitted when creating a new proposal.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProposalInput {
    /// The guild this proposal belongs to
    pub guild_id: u32,
    /// The proposed action
    pub action: ProposalAction,
    /// Resource amount for Allocate actions (ignored for others)
    pub resource_amount: u64,
    /// Target guild ID for Attack actions (ignored for others)
    pub target_guild_id: u32,
    /// Description of the proposal
    pub description: String,
}

/// Proposal component - a DAO governance proposal within a guild.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Proposal {
    /// Unique proposal identifier
    pub id: u32,
    /// Address of the proposer
    pub proposer: Address,
    /// Guild this proposal belongs to
    pub guild_id: u32,
    /// Proposed action
    pub action: ProposalAction,
    /// Resource amount (for Allocate)
    pub resource_amount: u64,
    /// Target guild (for Attack)
    pub target_guild_id: u32,
    /// Description
    pub description: String,
    /// Votes in favor
    pub votes_for: u32,
    /// Votes against
    pub votes_against: u32,
    /// Current status
    pub status: ProposalStatus,
    /// Ledger sequence when created
    pub created_at: u64,
    /// Ledger sequence when executed (0 if not executed)
    pub executed_at: u64,
}

// ============================================================================
// ECS Components - Strategy (stellar-zk)
// ============================================================================

/// Proof input - data used for submitting a strategic commitment.
///
/// The commitment is `SHA256(action_type || target_guild_id || resource_amount || salt)`.
/// This follows the stellar-zk commit-reveal pattern where the hash acts as
/// a sealed war plan that can only be verified when the preimage is revealed.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProofInput {
    /// SHA256 hash of the sealed strategy (32 bytes)
    pub commitment_hash: BytesN<32>,
    /// Guild ID this commitment belongs to
    pub guild_id: u32,
}

/// Strategy commitment component - an unrevealed sealed war plan.
///
/// Uses the stellar-zk nullifier pattern: once revealed, the commitment
/// is marked and cannot be replayed. This prevents double-execution of
/// hidden strategy actions.
#[contracttype]
#[derive(Clone, Debug)]
pub struct StrategyCommitment {
    /// SHA256 commitment hash (the "nullifier" in stellar-zk terms)
    pub commitment_hash: BytesN<32>,
    /// Who submitted the commitment
    pub committer: Address,
    /// Guild this commitment belongs to
    pub guild_id: u32,
    /// Ledger sequence when committed
    pub committed_at: u64,
    /// Whether this commitment has been revealed (nullifier used)
    pub revealed: bool,
}

/// Strategy reveal - the preimage data that proves a commitment.
///
/// When revealed, the contract recomputes
/// `SHA256(action_type || target_guild_id || resource_amount || salt)`
/// and verifies it matches the stored commitment hash.
#[contracttype]
#[derive(Clone, Debug)]
pub struct StrategyReveal {
    /// Type of strategic action (0=Defend, 1=Attack, 2=Upgrade, 3=Allocate)
    pub action_type: u32,
    /// Target guild ID
    pub target_guild_id: u32,
    /// Resource amount committed
    pub resource_amount: u64,
    /// Random salt used in the commitment (32 bytes)
    pub salt: BytesN<32>,
}

// ============================================================================
// ECS Components - Game State
// ============================================================================

/// Global game state - summary of the entire contract state.
#[contracttype]
#[derive(Clone, Debug)]
pub struct GameState {
    /// Total guilds created
    pub total_guilds: u32,
    /// Total proposals submitted across all guilds
    pub total_proposals: u32,
    /// Number of active (unresolved) campaigns
    pub active_campaigns: u32,
    /// Current game round (incremented on battle resolution)
    pub round: u32,
}

impl GameState {
    /// Create a new zeroed game state.
    pub fn new() -> Self {
        Self {
            total_guilds: 0,
            total_proposals: 0,
            active_campaigns: 0,
            round: 0,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Storage Keys
// ============================================================================

/// Storage keys for Soroban persistent storage.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Guild data by guild ID
    Guild(u32),
    /// Guild membership: (guild_id, member_address) -> bool
    GuildMember(u32, Address),
    /// Proposal by proposal ID
    Proposal(u32),
    /// Vote record: (proposal_id, voter_address) -> bool
    VoteRecord(u32, Address),
    /// Strategy commitment by (guild_id, member_address)
    Commitment(u32, Address),
    /// Nullifier tracking: commitment_hash -> bool (stellar-zk pattern)
    Nullifier(BytesN<32>),
    /// Global game state
    State,
    /// Count of cougr-core entities (ECS integration)
    EntityCount,
}

// ============================================================================
// Game Constants
// ============================================================================

/// Vote approval threshold percentage (51% majority).
pub const VOTE_THRESHOLD_PCT: u32 = 51;

/// Maximum members per guild.
pub const MAX_GUILD_MEMBERS: u32 = 10;

/// Proposal voting duration in ledger sequences.
pub const PROPOSAL_DURATION: u64 = 100;

/// Initial treasury balance for new guilds.
pub const INITIAL_TREASURY: u64 = 1000;

/// Treasury cost for defense actions.
pub const DEFEND_COST: u64 = 100;

/// Treasury cost for attack campaigns.
pub const ATTACK_COST: u64 = 200;

/// Treasury cost for guild upgrades.
pub const UPGRADE_COST: u64 = 150;

/// Defense bonus granted by a Defend proposal.
pub const DEFEND_BONUS: u32 = 10;

/// Attack bonus granted by an Upgrade proposal.
pub const UPGRADE_BONUS: u32 = 10;
