//! Strategy commitment and resolution systems for Guild Treasury Wars.
//!
//! This module implements the **stellar-zk** integration - the core ZK
//! mechanic of the game. Players submit sealed war plans as SHA256
//! commitments (hiding their strategic intent), then reveal them later
//! for deterministic on-chain resolution.
//!
//! # stellar-zk Pattern
//!
//! This follows the on-chain verification model from
//! [stellar-zk](https://github.com/salazarsebas/stellar-zk):
//!
//! 1. **Commitment** (sealed intent): `SHA256(action_type || target_guild_id
//!    || resource_amount || salt)` is stored on-chain. No one can see the
//!    strategy until it is revealed.
//!
//! 2. **Reveal** (proof verification): The player provides the preimage
//!    (action_type, target, amount, salt). The contract recomputes the hash
//!    and verifies it matches the stored commitment.
//!
//! 3. **Nullifier** (anti-replay): Once revealed, the commitment hash is
//!    marked as used. This prevents double-execution - the same mechanism
//!    used in stellar-zk verifier contracts for anti-replay protection.
//!
//! 4. **Resolution**: After both guilds reveal, the game deterministically
//!    resolves the strategic outcome (attack vs defense, resource transfers).

use soroban_sdk::{Address, Bytes, Env};

use crate::types::*;

// ============================================================================
// StrategyProofSystem - Commit-reveal for sealed war plans
// ============================================================================

/// Submit a sealed strategy commitment (stellar-zk commit phase).
///
/// The commitment hash is `SHA256(action_type || target_guild_id ||
/// resource_amount || salt)`. This allows a guild member to commit to
/// a strategic action without revealing it to opponents.
///
/// # stellar-zk Integration
/// This mirrors the nullifier-based pattern from stellar-zk: the
/// commitment hash acts as a unique identifier (like a nullifier) that
/// can only be consumed once during the reveal phase.
///
/// # Panics
/// - Panics if the member is not part of the specified guild.
/// - Panics if the commitment hash has already been used (nullifier check).
pub fn submit_strategy_commitment(env: &Env, guild_member: &Address, proof_input: &ProofInput) {
    guild_member.require_auth();

    // Verify guild membership
    let is_member: bool = env
        .storage()
        .persistent()
        .get(&DataKey::GuildMember(
            proof_input.guild_id,
            guild_member.clone(),
        ))
        .unwrap_or(false);
    assert!(is_member, "not a guild member");

    // Nullifier check - ensure this commitment hash hasn't been used before
    let nullifier_used: bool = env
        .storage()
        .persistent()
        .get(&DataKey::Nullifier(proof_input.commitment_hash.clone()))
        .unwrap_or(false);
    assert!(!nullifier_used, "commitment hash already used");

    // Store the commitment
    let commitment = StrategyCommitment {
        commitment_hash: proof_input.commitment_hash.clone(),
        committer: guild_member.clone(),
        guild_id: proof_input.guild_id,
        committed_at: env.ledger().sequence() as u64,
        revealed: false,
    };

    env.storage().persistent().set(
        &DataKey::Commitment(proof_input.guild_id, guild_member.clone()),
        &commitment,
    );
}

/// Reveal a previously submitted strategy commitment (stellar-zk reveal phase).
///
/// The contract recomputes `SHA256(action_type || target_guild_id ||
/// resource_amount || salt)` from the provided preimage and verifies it
/// matches the stored commitment hash. On success, the nullifier is
/// consumed (marked as used) to prevent replay.
///
/// # stellar-zk Integration
/// This is the on-chain verification step: the proof (preimage) is
/// validated against the commitment. The nullifier tracking ensures
/// each commitment can only be revealed once, following stellar-zk's
/// anti-replay protection model.
///
/// # Panics
/// - Panics if no commitment exists for this member/guild.
/// - Panics if the commitment has already been revealed.
/// - Panics if the revealed data does not match the commitment hash.
pub fn reveal_strategy(env: &Env, guild_member: &Address, guild_id: u32, reveal: &StrategyReveal) {
    guild_member.require_auth();

    // Load the commitment
    let mut commitment: StrategyCommitment = env
        .storage()
        .persistent()
        .get(&DataKey::Commitment(guild_id, guild_member.clone()))
        .expect("no commitment found");

    // Check nullifier - commitment must not have been revealed
    assert!(!commitment.revealed, "commitment already revealed");

    // Recompute the commitment hash from revealed preimage
    // Hash = SHA256(action_type || target_guild_id || resource_amount || salt)
    let computed_hash = compute_commitment_hash(
        env,
        reveal.action_type,
        reveal.target_guild_id,
        reveal.resource_amount,
        &reveal.salt,
    );

    // Verify the hash matches - this is the proof verification step
    assert!(
        computed_hash == commitment.commitment_hash,
        "reveal does not match commitment"
    );

    // Mark commitment as revealed (consume nullifier)
    commitment.revealed = true;
    env.storage().persistent().set(
        &DataKey::Commitment(guild_id, guild_member.clone()),
        &commitment,
    );

    // Mark the nullifier as used globally (stellar-zk anti-replay)
    env.storage().persistent().set(
        &DataKey::Nullifier(commitment.commitment_hash.clone()),
        &true,
    );
}

// ============================================================================
// ResolutionSystem - Deterministic battle outcome resolution
// ============================================================================

/// Resolve a battle between two guilds after strategies are revealed.
///
/// Compares the attacking guild's attack strength + committed resources
/// against the defending guild's defense strength. The outcome
/// deterministically transfers resources: successful attacks drain the
/// defender's treasury; failed attacks waste the attacker's resources.
///
/// # Panics
/// - Panics if either guild does not exist.
/// - Panics if the attacker's commitment has not been revealed.
pub fn resolve_battle(
    env: &Env,
    attacker_member: &Address,
    defender_member: &Address,
    attacker_guild_id: u32,
    defender_guild_id: u32,
) {
    // Load the attacker's revealed commitment
    let attacker_commitment: StrategyCommitment = env
        .storage()
        .persistent()
        .get(&DataKey::Commitment(
            attacker_guild_id,
            attacker_member.clone(),
        ))
        .expect("attacker has no commitment");
    assert!(
        attacker_commitment.revealed,
        "attacker strategy not yet revealed"
    );

    // Load guilds
    let mut attacker_guild: Guild = env
        .storage()
        .persistent()
        .get(&DataKey::Guild(attacker_guild_id))
        .expect("attacker guild does not exist");

    let mut defender_guild: Guild = env
        .storage()
        .persistent()
        .get(&DataKey::Guild(defender_guild_id))
        .expect("defender guild does not exist");

    // Determine battle outcome
    // Attack power = guild attack_strength (from Upgrades)
    // Defense power = guild defense_strength (from Defend proposals)
    let attack_power = attacker_guild.attack_strength;
    let defense_power = defender_guild.defense_strength;

    // Check defender's commitment if it exists
    let defender_has_defense = if let Some(defender_commitment) = env
        .storage()
        .persistent()
        .get::<DataKey, StrategyCommitment>(
        &DataKey::Commitment(defender_guild_id, defender_member.clone()),
    ) {
        defender_commitment.revealed
    } else {
        false
    };

    // Combined defense includes guild strength + active defense commitment
    let total_defense = if defender_has_defense {
        defense_power + DEFEND_BONUS
    } else {
        defense_power
    };

    // Resolution: attacker wins if attack_power > total_defense
    let plunder_amount: u64 = 50; // Fixed plunder on successful attack
    if attack_power > total_defense {
        // Attacker wins - transfer resources
        let transfer = if defender_guild.treasury >= plunder_amount {
            plunder_amount
        } else {
            defender_guild.treasury
        };
        defender_guild.treasury -= transfer;
        attacker_guild.treasury += transfer;
    }
    // If defense wins, no resource transfer (attacker already paid ATTACK_COST)

    // Update game state
    let mut state: GameState = env
        .storage()
        .instance()
        .get(&DataKey::State)
        .unwrap_or_default();
    state.round += 1;
    if state.active_campaigns > 0 {
        state.active_campaigns -= 1;
    }
    env.storage().instance().set(&DataKey::State, &state);

    // Persist guild changes
    env.storage()
        .persistent()
        .set(&DataKey::Guild(attacker_guild_id), &attacker_guild);
    env.storage()
        .persistent()
        .set(&DataKey::Guild(defender_guild_id), &defender_guild);
}

// ============================================================================
// Commitment hash computation (stellar-zk primitive)
// ============================================================================

/// Compute the SHA256 commitment hash from strategy preimage fields.
///
/// `hash = SHA256(action_type || target_guild_id || resource_amount || salt)`
///
/// This is the core cryptographic primitive that enables sealed war plans.
/// The hash hides the strategy while the preimage serves as the proof.
pub fn compute_commitment_hash(
    env: &Env,
    action_type: u32,
    target_guild_id: u32,
    resource_amount: u64,
    salt: &soroban_sdk::BytesN<32>,
) -> soroban_sdk::BytesN<32> {
    let mut data = Bytes::new(env);

    // Serialize fields into the hash preimage
    data.append(&Bytes::from_array(env, &action_type.to_be_bytes()));
    data.append(&Bytes::from_array(env, &target_guild_id.to_be_bytes()));
    data.append(&Bytes::from_array(env, &resource_amount.to_be_bytes()));
    data.append(&Bytes::from_slice(env, &salt.to_array()));

    // Compute SHA256 hash
    env.crypto().sha256(&data).into()
}
