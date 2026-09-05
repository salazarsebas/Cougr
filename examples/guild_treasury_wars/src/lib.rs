//! Guild Treasury Wars - DAO-Governed Factions with stellar-zk Commitments
//!
//! This smart contract implements a guild-based strategy game on the Stellar
//! blockchain using cougr-core's ECS framework. Guilds manage shared treasuries,
//! vote on strategic actions through DAO mechanics, and compete through
//! resource-driven campaigns with hidden strategic commitments.
//!
//! # stellar-zk Integration
//! Players submit **sealed war plans** as SHA256 commitments (commit-reveal).
//! This hides strategic intent until the reveal phase, at which point the
//! contract verifies the preimage and applies deterministic battle resolution.
//! The nullifier pattern from stellar-zk prevents double-execution of
//! revealed commitments.
//!
//! # Gameplay Flow
//! 1. **Guild Creation**: Admin creates a guild with initial treasury
//! 2. **Membership**: Players join guilds (up to MAX_GUILD_MEMBERS)
//! 3. **Proposals**: Members submit proposals (Defend/Attack/Upgrade/Allocate)
//! 4. **Voting**: Members vote; 51% threshold approves/rejects
//! 5. **Execution**: Approved proposals deduct treasury, apply effects
//! 6. **Strategy**: Members submit sealed commitments (stellar-zk)
//! 7. **Reveal**: Commitments are revealed and verified on-chain
//! 8. **Resolution**: Battle outcomes determined deterministically
//!
//! # Cougr-Core Integration
//! - ECS World: Entity management for game objects
//! - Components: Guild, Proposal, StrategyCommitment, GameState

#![no_std]

mod governance;
mod strategy;
mod types;

#[cfg(test)]
mod test;

use cougr_core::SimpleWorld;
use soroban_sdk::{contract, contractimpl, Address, Env};

pub use types::*;

#[contract]
pub struct GuildTreasuryWarsContract;

#[contractimpl]
impl GuildTreasuryWarsContract {
    /// Initialize a new guild with treasury resources.
    ///
    /// Creates the guild with the given admin as the first member and
    /// allocates `INITIAL_TREASURY` (1000) resources. Uses cougr-core's
    /// ECS World for entity tracking.
    ///
    /// # Arguments
    /// * `guild_admin` - The guild administrator's Stellar address
    ///
    /// # Returns
    /// The guild ID assigned to the new guild
    pub fn init_guild(env: Env, guild_admin: Address) -> u32 {
        // Create cougr-core ECS World for entity management
        let mut world = SimpleWorld::new(&env);
        let _guild_entity = world.spawn_entity();

        let guild_id = governance::init_guild(&env, &guild_admin);

        // Store ECS entity count
        env.storage()
            .instance()
            .set(&DataKey::EntityCount, &(world.next_entity_id() - 1));

        guild_id
    }

    /// Join an existing guild as a new member.
    ///
    /// # Arguments
    /// * `guild_id` - The guild to join
    /// * `member` - The joining member's address
    pub fn join_guild(env: Env, guild_id: u32, member: Address) {
        governance::join_guild(&env, guild_id, &member);
    }

    /// Submit a governance proposal for a guild.
    ///
    /// The proposer must be a member of the guild. Creates an Active proposal
    /// that other members can vote on. Proposals can be:
    /// - **Defend** (cost: 100): Strengthen guild defenses
    /// - **Attack** (cost: 200): Launch a campaign against another guild
    /// - **Upgrade** (cost: 150): Upgrade guild capabilities
    /// - **Allocate** (custom): Allocate resources for a purpose
    ///
    /// # Arguments
    /// * `proposer` - Address of the proposing member
    /// * `proposal` - The proposal input data
    ///
    /// # Returns
    /// The proposal ID assigned to the new proposal
    pub fn submit_proposal(env: Env, proposer: Address, proposal: ProposalInput) -> u32 {
        governance::submit_proposal(&env, &proposer, &proposal)
    }

    /// Cast a vote on an active proposal.
    ///
    /// The voter must be a member of the proposal's guild and must not have
    /// already voted. Votes are evaluated against a 51% threshold.
    ///
    /// # Arguments
    /// * `voter` - Address of the voting member
    /// * `proposal_id` - The proposal to vote on
    /// * `support` - `true` for, `false` against
    pub fn vote(env: Env, voter: Address, proposal_id: u32, support: bool) {
        governance::vote(&env, &voter, proposal_id, support);
    }

    /// Execute an approved proposal.
    ///
    /// Deducts the action cost from the guild treasury and applies the
    /// corresponding effect. The proposal must have been approved through
    /// the voting process.
    ///
    /// # Arguments
    /// * `proposal_id` - The approved proposal to execute
    pub fn execute_proposal(env: Env, proposal_id: u32) {
        governance::execute_proposal(&env, proposal_id);
    }

    /// Submit a sealed strategy commitment (stellar-zk commit phase).
    ///
    /// The commitment hash is `SHA256(action_type || target_guild_id ||
    /// resource_amount || salt)`. This allows a guild member to commit to
    /// a strategic action without revealing it to opponents.
    ///
    /// # Arguments
    /// * `guild_member` - Address of the committing member
    /// * `proof_input` - Contains the commitment hash and guild ID
    pub fn submit_strategy_commitment(env: Env, guild_member: Address, proof_input: ProofInput) {
        strategy::submit_strategy_commitment(&env, &guild_member, &proof_input);
    }

    /// Reveal a previously submitted strategy commitment.
    ///
    /// The contract recomputes the commitment hash from the revealed preimage
    /// and verifies it matches the stored commitment. Uses nullifier tracking
    /// to prevent double-reveal (stellar-zk anti-replay pattern).
    ///
    /// # Arguments
    /// * `guild_member` - Address of the member revealing their strategy
    /// * `guild_id` - The guild the commitment belongs to
    /// * `reveal` - The preimage data (action_type, target, amount, salt)
    pub fn reveal_strategy(env: Env, guild_member: Address, guild_id: u32, reveal: StrategyReveal) {
        strategy::reveal_strategy(&env, &guild_member, guild_id, &reveal);
    }

    /// Resolve a battle between two guilds after strategies are revealed.
    ///
    /// Compares attack and defense strengths to determine the outcome.
    /// Successful attacks transfer resources from the defender's treasury.
    ///
    /// # Arguments
    /// * `attacker_member` - The attacking guild member
    /// * `defender_member` - The defending guild member
    /// * `attacker_guild_id` - The attacking guild's ID
    /// * `defender_guild_id` - The defending guild's ID
    pub fn resolve_battle(
        env: Env,
        attacker_member: Address,
        defender_member: Address,
        attacker_guild_id: u32,
        defender_guild_id: u32,
    ) {
        strategy::resolve_battle(
            &env,
            &attacker_member,
            &defender_member,
            attacker_guild_id,
            defender_guild_id,
        );
    }

    /// Get the current global game state.
    ///
    /// Returns total guilds, proposals, active campaigns, and current round.
    pub fn get_state(env: Env) -> GameState {
        governance::get_state(&env)
    }

    /// Get a guild by ID.
    pub fn get_guild(env: Env, guild_id: u32) -> Guild {
        governance::get_guild(&env, guild_id)
    }

    /// Get a proposal by ID.
    pub fn get_proposal(env: Env, proposal_id: u32) -> Proposal {
        governance::get_proposal(&env, proposal_id)
    }

    /// Get the cougr-core entity count (demonstrates ECS integration).
    pub fn get_entity_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::EntityCount)
            .unwrap_or(0)
    }
}
