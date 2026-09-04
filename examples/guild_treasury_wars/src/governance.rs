//! Governance systems for Guild Treasury Wars.
//!
//! Implements the DAO-style governance mechanics following cougr-core's ECS
//! pattern, inspired by Stellar governance patterns (governance.script3.io):
//! - **ProposalSystem**: Guild members submit proposals for strategic actions
//! - **VotingSystem**: Members cast votes, threshold determines approval
//! - **TreasuryExecutionSystem**: Approved proposals deduct treasury and
//!   apply effects (defense boosts, attack campaigns, upgrades)

use soroban_sdk::{Address, Env};

use crate::types::*;

// ============================================================================
// ProposalSystem - Guild creation and proposal management
// ============================================================================

/// Initialize a new guild with a shared treasury.
///
/// Creates the guild with the given admin as the first member and
/// allocates `INITIAL_TREASURY` resources. Uses cougr-core's ECS World
/// for entity tracking.
///
/// # Panics
/// Panics if the admin does not authorize the operation.
pub fn init_guild(env: &Env, guild_admin: &Address) -> u32 {
    guild_admin.require_auth();

    // Load or create game state
    let mut state: GameState = env
        .storage()
        .instance()
        .get(&DataKey::State)
        .unwrap_or_default();

    let guild_id = state.total_guilds;
    state.total_guilds += 1;

    // Create guild component
    let guild = Guild {
        admin: guild_admin.clone(),
        name: soroban_sdk::String::from_str(env, "Guild"),
        treasury: INITIAL_TREASURY,
        member_count: 1,
        created_at: env.ledger().sequence() as u64,
        defense_strength: 0,
        attack_strength: 0,
    };

    // Store guild on-chain
    env.storage()
        .persistent()
        .set(&DataKey::Guild(guild_id), &guild);

    // Register admin as first member
    env.storage()
        .persistent()
        .set(&DataKey::GuildMember(guild_id, guild_admin.clone()), &true);

    // Update global state
    env.storage().instance().set(&DataKey::State, &state);

    guild_id
}

/// Add a member to an existing guild.
///
/// # Panics
/// Panics if the guild does not exist, the member limit is reached,
/// or the member is already registered.
pub fn join_guild(env: &Env, guild_id: u32, member: &Address) {
    member.require_auth();

    let mut guild: Guild = env
        .storage()
        .persistent()
        .get(&DataKey::Guild(guild_id))
        .expect("guild does not exist");

    // Check member limit
    assert!(
        guild.member_count < MAX_GUILD_MEMBERS,
        "guild is at maximum capacity"
    );

    // Check not already a member
    let already_member: bool = env
        .storage()
        .persistent()
        .get(&DataKey::GuildMember(guild_id, member.clone()))
        .unwrap_or(false);
    assert!(!already_member, "already a guild member");

    // Register member
    env.storage()
        .persistent()
        .set(&DataKey::GuildMember(guild_id, member.clone()), &true);

    guild.member_count += 1;
    env.storage()
        .persistent()
        .set(&DataKey::Guild(guild_id), &guild);
}

/// Submit a governance proposal for a guild.
///
/// The proposer must be a member of the guild. Creates an Active proposal
/// that other members can vote on.
///
/// # Panics
/// Panics if the proposer is not a guild member.
pub fn submit_proposal(env: &Env, proposer: &Address, input: &ProposalInput) -> u32 {
    proposer.require_auth();

    // Verify proposer is a guild member
    let is_member: bool = env
        .storage()
        .persistent()
        .get(&DataKey::GuildMember(input.guild_id, proposer.clone()))
        .unwrap_or(false);
    assert!(is_member, "proposer is not a guild member");

    // Load game state for proposal ID
    let mut state: GameState = env
        .storage()
        .instance()
        .get(&DataKey::State)
        .unwrap_or_default();

    let proposal_id = state.total_proposals;
    state.total_proposals += 1;

    // Create proposal
    let proposal = Proposal {
        id: proposal_id,
        proposer: proposer.clone(),
        guild_id: input.guild_id,
        action: input.action.clone(),
        resource_amount: input.resource_amount,
        target_guild_id: input.target_guild_id,
        description: input.description.clone(),
        votes_for: 0,
        votes_against: 0,
        status: ProposalStatus::Active,
        created_at: env.ledger().sequence() as u64,
        executed_at: 0,
    };

    env.storage()
        .persistent()
        .set(&DataKey::Proposal(proposal_id), &proposal);

    env.storage().instance().set(&DataKey::State, &state);

    proposal_id
}

// ============================================================================
// VotingSystem - Vote casting and threshold evaluation
// ============================================================================

/// Cast a vote on a proposal.
///
/// The voter must be a member of the proposal's guild and must not have
/// voted already. After each vote, the threshold is evaluated: if votes_for
/// reaches `VOTE_THRESHOLD_PCT` of the guild's member count, the proposal
/// is approved; if votes_against reaches the complement, it is rejected.
///
/// # Panics
/// Panics if the voter is not a guild member, has already voted, or the
/// proposal is not active.
pub fn vote(env: &Env, voter: &Address, proposal_id: u32, support: bool) {
    voter.require_auth();

    let mut proposal: Proposal = env
        .storage()
        .persistent()
        .get(&DataKey::Proposal(proposal_id))
        .expect("proposal does not exist");

    // Must be active
    assert!(
        proposal.status == ProposalStatus::Active,
        "proposal is not active"
    );

    // Verify voter is a guild member
    let is_member: bool = env
        .storage()
        .persistent()
        .get(&DataKey::GuildMember(proposal.guild_id, voter.clone()))
        .unwrap_or(false);
    assert!(is_member, "voter is not a guild member");

    // Check hasn't voted already
    let has_voted: bool = env
        .storage()
        .persistent()
        .get(&DataKey::VoteRecord(proposal_id, voter.clone()))
        .unwrap_or(false);
    assert!(!has_voted, "member has already voted");

    // Record vote
    env.storage()
        .persistent()
        .set(&DataKey::VoteRecord(proposal_id, voter.clone()), &true);

    if support {
        proposal.votes_for += 1;
    } else {
        proposal.votes_against += 1;
    }

    // Evaluate threshold
    let guild: Guild = env
        .storage()
        .persistent()
        .get(&DataKey::Guild(proposal.guild_id))
        .expect("guild does not exist");

    let threshold = (guild.member_count * VOTE_THRESHOLD_PCT).div_ceil(100);

    if proposal.votes_for >= threshold {
        proposal.status = ProposalStatus::Approved;
    } else if proposal.votes_against >= threshold {
        proposal.status = ProposalStatus::Rejected;
    }

    env.storage()
        .persistent()
        .set(&DataKey::Proposal(proposal_id), &proposal);
}

// ============================================================================
// TreasuryExecutionSystem - Execute approved proposals
// ============================================================================

/// Execute an approved proposal, applying treasury changes.
///
/// Deducts the action cost from the guild treasury and applies the
/// corresponding effect (defense boost, attack campaign, upgrade, or
/// custom allocation).
///
/// # Panics
/// Panics if the proposal is not approved, the guild has insufficient
/// treasury, or the proposal has already been executed.
pub fn execute_proposal(env: &Env, proposal_id: u32) {
    let mut proposal: Proposal = env
        .storage()
        .persistent()
        .get(&DataKey::Proposal(proposal_id))
        .expect("proposal does not exist");

    assert!(
        proposal.status == ProposalStatus::Approved,
        "proposal is not approved"
    );

    let mut guild: Guild = env
        .storage()
        .persistent()
        .get(&DataKey::Guild(proposal.guild_id))
        .expect("guild does not exist");

    // Determine cost based on action
    let cost = match proposal.action {
        ProposalAction::Defend => DEFEND_COST,
        ProposalAction::Attack => ATTACK_COST,
        ProposalAction::Upgrade => UPGRADE_COST,
        ProposalAction::Allocate => proposal.resource_amount,
    };

    assert!(
        guild.treasury >= cost,
        "insufficient treasury for this action"
    );

    // Deduct treasury
    guild.treasury -= cost;

    // Apply action effects
    match proposal.action {
        ProposalAction::Defend => {
            guild.defense_strength += DEFEND_BONUS;
        }
        ProposalAction::Attack => {
            // Attack campaign is tracked via active_campaigns
            let mut state: GameState = env
                .storage()
                .instance()
                .get(&DataKey::State)
                .unwrap_or_default();
            state.active_campaigns += 1;
            env.storage().instance().set(&DataKey::State, &state);
        }
        ProposalAction::Upgrade => {
            guild.attack_strength += UPGRADE_BONUS;
        }
        ProposalAction::Allocate => {
            // Generic allocation - treasury already deducted
        }
    }

    // Mark proposal as executed
    proposal.status = ProposalStatus::Executed;
    proposal.executed_at = env.ledger().sequence() as u64;

    env.storage()
        .persistent()
        .set(&DataKey::Proposal(proposal_id), &proposal);
    env.storage()
        .persistent()
        .set(&DataKey::Guild(proposal.guild_id), &guild);
}

// ============================================================================
// Query functions
// ============================================================================

/// Get the global game state.
pub fn get_state(env: &Env) -> GameState {
    env.storage()
        .instance()
        .get(&DataKey::State)
        .unwrap_or_default()
}

/// Get a guild by ID.
///
/// # Panics
/// Panics if the guild does not exist.
pub fn get_guild(env: &Env, guild_id: u32) -> Guild {
    env.storage()
        .persistent()
        .get(&DataKey::Guild(guild_id))
        .expect("guild does not exist")
}

/// Get a proposal by ID.
///
/// # Panics
/// Panics if the proposal does not exist.
pub fn get_proposal(env: &Env, proposal_id: u32) -> Proposal {
    env.storage()
        .persistent()
        .get(&DataKey::Proposal(proposal_id))
        .expect("proposal does not exist")
}
