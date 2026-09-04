//! Unit tests for Guild Treasury Wars contract
//!
//! These tests validate the complete governance + strategy flow:
//! - Guild initialization and membership
//! - Proposal creation and validation
//! - Vote counting and threshold behavior
//! - Treasury execution for approved proposals
//! - Invalid and unauthorized actions
//! - stellar-zk commitment, reveal, and verification
//! - Battle resolution with deterministic outcomes
//! - Nullifier anti-replay protection

use super::*;
use soroban_sdk::{testutils::Address as _, Env};

// ============================================================================
// Helpers
// ============================================================================

/// Helper: Set up a guild with an admin and return (contract_id, client, admin, guild_id).
fn setup_guild<'a>(env: &'a Env) -> (Address, GuildTreasuryWarsContractClient<'a>, Address, u32) {
    env.mock_all_auths();
    let contract_id = env.register(GuildTreasuryWarsContract, ());
    let client = GuildTreasuryWarsContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let guild_id = client.init_guild(&admin);

    (contract_id, client, admin, guild_id)
}

/// Helper: Compute the commitment hash for a given strategy.
fn compute_test_hash(
    env: &Env,
    action_type: u32,
    target_guild_id: u32,
    resource_amount: u64,
    salt: &soroban_sdk::BytesN<32>,
) -> soroban_sdk::BytesN<32> {
    strategy::compute_commitment_hash(env, action_type, target_guild_id, resource_amount, salt)
}

/// Helper: Create a test salt (32 bytes).
fn make_test_salt(env: &Env, seed: u8) -> soroban_sdk::BytesN<32> {
    let mut bytes = [0u8; 32];
    bytes[0] = seed;
    bytes[1] = 0xAB;
    soroban_sdk::BytesN::from_array(env, &bytes)
}

// ============================================================================
// Guild Initialization Tests
// ============================================================================

/// Test that guild initialization creates a guild with correct state.
#[test]
fn test_init_guild() {
    let env = Env::default();
    let (_, client, admin, guild_id) = setup_guild(&env);

    assert_eq!(guild_id, 0);

    let guild = client.get_guild(&guild_id);
    assert_eq!(guild.admin, admin);
    assert_eq!(guild.treasury, INITIAL_TREASURY);
    assert_eq!(guild.member_count, 1);
    assert_eq!(guild.defense_strength, 0);
    assert_eq!(guild.attack_strength, 0);

    // Verify game state
    let state = client.get_state();
    assert_eq!(state.total_guilds, 1);
    assert_eq!(state.total_proposals, 0);

    // Verify ECS entity count
    assert!(client.get_entity_count() > 0);
}

/// Test joining a guild as a new member.
#[test]
fn test_join_guild() {
    let env = Env::default();
    let (_, client, _, guild_id) = setup_guild(&env);

    let member = Address::generate(&env);
    client.join_guild(&guild_id, &member);

    let guild = client.get_guild(&guild_id);
    assert_eq!(guild.member_count, 2);
}

// ============================================================================
// Proposal Tests
// ============================================================================

/// Test submitting a governance proposal.
#[test]
fn test_submit_proposal() {
    let env = Env::default();
    let (_, client, admin, guild_id) = setup_guild(&env);

    let input = ProposalInput {
        guild_id,
        action: ProposalAction::Defend,
        resource_amount: 0,
        target_guild_id: 0,
        description: soroban_sdk::String::from_str(&env, "Strengthen walls"),
    };

    let proposal_id = client.submit_proposal(&admin, &input);
    assert_eq!(proposal_id, 0);

    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.proposer, admin);
    assert_eq!(proposal.guild_id, guild_id);
    assert_eq!(proposal.action, ProposalAction::Defend);
    assert_eq!(proposal.votes_for, 0);
    assert_eq!(proposal.votes_against, 0);
    assert_eq!(proposal.status, ProposalStatus::Active);
}

/// Test that a non-member cannot submit a proposal.
#[test]
#[should_panic(expected = "proposer is not a guild member")]
fn test_unauthorized_proposal() {
    let env = Env::default();
    let (_, client, _, guild_id) = setup_guild(&env);

    let outsider = Address::generate(&env);
    let input = ProposalInput {
        guild_id,
        action: ProposalAction::Defend,
        resource_amount: 0,
        target_guild_id: 0,
        description: soroban_sdk::String::from_str(&env, "Infiltrate"),
    };

    client.submit_proposal(&outsider, &input);
}

// ============================================================================
// Voting Tests
// ============================================================================

/// Test vote counting and approval threshold.
#[test]
fn test_vote_counting_and_threshold() {
    let env = Env::default();
    let (_, client, admin, guild_id) = setup_guild(&env);

    // Add a second member
    let member = Address::generate(&env);
    client.join_guild(&guild_id, &member);

    // Submit a proposal
    let input = ProposalInput {
        guild_id,
        action: ProposalAction::Defend,
        resource_amount: 0,
        target_guild_id: 0,
        description: soroban_sdk::String::from_str(&env, "Fortify"),
    };
    let proposal_id = client.submit_proposal(&admin, &input);

    // Both members vote yes - 2/2 = 100% > 51%
    client.vote(&admin, &proposal_id, &true);
    client.vote(&member, &proposal_id, &true);

    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.votes_for, 2);
    assert_eq!(proposal.status, ProposalStatus::Approved);
}

/// Test that a majority-against vote rejects the proposal.
#[test]
fn test_vote_rejection() {
    let env = Env::default();
    let (_, client, admin, guild_id) = setup_guild(&env);

    let member = Address::generate(&env);
    client.join_guild(&guild_id, &member);

    let input = ProposalInput {
        guild_id,
        action: ProposalAction::Attack,
        resource_amount: 0,
        target_guild_id: 1,
        description: soroban_sdk::String::from_str(&env, "Bad idea"),
    };
    let proposal_id = client.submit_proposal(&admin, &input);

    // Both members vote no
    client.vote(&admin, &proposal_id, &false);
    client.vote(&member, &proposal_id, &false);

    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.votes_against, 2);
    assert_eq!(proposal.status, ProposalStatus::Rejected);
}

/// Test that a non-member cannot vote.
#[test]
#[should_panic(expected = "voter is not a guild member")]
fn test_unauthorized_vote() {
    let env = Env::default();
    let (_, client, admin, guild_id) = setup_guild(&env);

    let input = ProposalInput {
        guild_id,
        action: ProposalAction::Defend,
        resource_amount: 0,
        target_guild_id: 0,
        description: soroban_sdk::String::from_str(&env, "Test"),
    };
    let proposal_id = client.submit_proposal(&admin, &input);

    let outsider = Address::generate(&env);
    client.vote(&outsider, &proposal_id, &true);
}

/// Test that a member cannot vote twice on the same proposal.
#[test]
#[should_panic(expected = "member has already voted")]
fn test_double_vote() {
    let env = Env::default();
    let (_, client, admin, guild_id) = setup_guild(&env);

    // Add two more members so the first vote doesn't auto-approve (1/3 < 51%)
    let member2 = Address::generate(&env);
    let member3 = Address::generate(&env);
    client.join_guild(&guild_id, &member2);
    client.join_guild(&guild_id, &member3);

    let input = ProposalInput {
        guild_id,
        action: ProposalAction::Defend,
        resource_amount: 0,
        target_guild_id: 0,
        description: soroban_sdk::String::from_str(&env, "Test"),
    };
    let proposal_id = client.submit_proposal(&admin, &input);

    client.vote(&admin, &proposal_id, &true);
    client.vote(&admin, &proposal_id, &true);
}

// ============================================================================
// Treasury Execution Tests
// ============================================================================

/// Test that executing an approved Defend proposal deducts treasury and boosts defense.
#[test]
fn test_treasury_execution_approved() {
    let env = Env::default();
    let (_, client, admin, guild_id) = setup_guild(&env);

    let input = ProposalInput {
        guild_id,
        action: ProposalAction::Defend,
        resource_amount: 0,
        target_guild_id: 0,
        description: soroban_sdk::String::from_str(&env, "Build walls"),
    };
    let proposal_id = client.submit_proposal(&admin, &input);

    // Single-member guild: 1 vote = 100% > 51% → Approved
    client.vote(&admin, &proposal_id, &true);

    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.status, ProposalStatus::Approved);

    // Execute
    client.execute_proposal(&proposal_id);

    // Verify treasury deducted
    let guild = client.get_guild(&guild_id);
    assert_eq!(guild.treasury, INITIAL_TREASURY - DEFEND_COST);
    assert_eq!(guild.defense_strength, DEFEND_BONUS);

    // Verify proposal marked as executed
    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.status, ProposalStatus::Executed);
}

/// Test that executing a non-approved proposal panics.
#[test]
#[should_panic(expected = "proposal is not approved")]
fn test_execute_unapproved() {
    let env = Env::default();
    let (_, client, admin, guild_id) = setup_guild(&env);

    let input = ProposalInput {
        guild_id,
        action: ProposalAction::Defend,
        resource_amount: 0,
        target_guild_id: 0,
        description: soroban_sdk::String::from_str(&env, "Test"),
    };
    let proposal_id = client.submit_proposal(&admin, &input);

    // Don't vote - proposal is still Active
    client.execute_proposal(&proposal_id);
}

// ============================================================================
// Strategy Commitment Tests (stellar-zk)
// ============================================================================

/// Test the full commit-reveal flow for sealed war plans.
#[test]
fn test_strategy_commitment_and_reveal() {
    let env = Env::default();
    let (contract_id, client, admin, guild_id) = setup_guild(&env);

    let salt = make_test_salt(&env, 42);
    let action_type: u32 = 1; // Attack
    let target_guild_id: u32 = 1;
    let resource_amount: u64 = 200;

    // Compute commitment hash
    let commitment_hash =
        compute_test_hash(&env, action_type, target_guild_id, resource_amount, &salt);

    // Submit commitment (sealed war plan)
    let proof_input = ProofInput {
        commitment_hash: commitment_hash.clone(),
        guild_id,
    };
    client.submit_strategy_commitment(&admin, &proof_input);

    // Verify commitment is stored
    env.as_contract(&contract_id, || {
        let commitment: StrategyCommitment = env
            .storage()
            .persistent()
            .get(&DataKey::Commitment(guild_id, admin.clone()))
            .expect("commitment should exist");
        assert_eq!(commitment.commitment_hash, commitment_hash);
        assert!(!commitment.revealed);
    });

    // Reveal strategy
    let reveal = StrategyReveal {
        action_type,
        target_guild_id,
        resource_amount,
        salt,
    };
    client.reveal_strategy(&admin, &guild_id, &reveal);

    // Verify commitment is now revealed (nullifier consumed)
    env.as_contract(&contract_id, || {
        let commitment: StrategyCommitment = env
            .storage()
            .persistent()
            .get(&DataKey::Commitment(guild_id, admin.clone()))
            .expect("commitment should exist");
        assert!(commitment.revealed);

        // Verify nullifier is marked as used
        let nullifier_used: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Nullifier(commitment_hash.clone()))
            .unwrap_or(false);
        assert!(nullifier_used);
    });
}

/// Test that an invalid reveal (wrong preimage) fails verification.
#[test]
#[should_panic(expected = "reveal does not match commitment")]
fn test_invalid_reveal() {
    let env = Env::default();
    let (_, client, admin, guild_id) = setup_guild(&env);

    let salt = make_test_salt(&env, 42);
    let commitment_hash = compute_test_hash(&env, 1, 1, 200, &salt);

    let proof_input = ProofInput {
        commitment_hash,
        guild_id,
    };
    client.submit_strategy_commitment(&admin, &proof_input);

    // Try to reveal with wrong data
    let wrong_reveal = StrategyReveal {
        action_type: 0, // Wrong action type!
        target_guild_id: 1,
        resource_amount: 200,
        salt,
    };
    client.reveal_strategy(&admin, &guild_id, &wrong_reveal);
}

/// Test that revealing the same commitment twice is prevented (nullifier).
#[test]
#[should_panic(expected = "commitment already revealed")]
fn test_double_reveal_prevention() {
    let env = Env::default();
    let (_, client, admin, guild_id) = setup_guild(&env);

    let salt = make_test_salt(&env, 42);
    let commitment_hash = compute_test_hash(&env, 1, 1, 200, &salt);

    let proof_input = ProofInput {
        commitment_hash,
        guild_id,
    };
    client.submit_strategy_commitment(&admin, &proof_input);

    let reveal = StrategyReveal {
        action_type: 1,
        target_guild_id: 1,
        resource_amount: 200,
        salt: salt.clone(),
    };

    // First reveal succeeds
    client.reveal_strategy(&admin, &guild_id, &reveal);

    // Second reveal should panic - nullifier already consumed
    let reveal2 = StrategyReveal {
        action_type: 1,
        target_guild_id: 1,
        resource_amount: 200,
        salt,
    };
    client.reveal_strategy(&admin, &guild_id, &reveal2);
}

// ============================================================================
// Battle Resolution Tests
// ============================================================================

/// Test battle resolution between two guilds.
#[test]
fn test_battle_resolution() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(GuildTreasuryWarsContract, ());
    let client = GuildTreasuryWarsContractClient::new(&env, &contract_id);

    // Create two guilds
    let admin_a = Address::generate(&env);
    let admin_b = Address::generate(&env);
    let guild_a = client.init_guild(&admin_a);
    let guild_b = client.init_guild(&admin_b);

    // Give guild A attack strength via Upgrade proposal
    let upgrade_input = ProposalInput {
        guild_id: guild_a,
        action: ProposalAction::Upgrade,
        resource_amount: 0,
        target_guild_id: 0,
        description: soroban_sdk::String::from_str(&env, "Sharpen swords"),
    };
    let upgrade_id = client.submit_proposal(&admin_a, &upgrade_input);
    client.vote(&admin_a, &upgrade_id, &true);
    client.execute_proposal(&upgrade_id);

    // Guild A now has attack_strength = UPGRADE_BONUS (10)
    let guild_a_state = client.get_guild(&guild_a);
    assert_eq!(guild_a_state.attack_strength, UPGRADE_BONUS);

    // Guild A commits and reveals an attack strategy
    let salt_a = make_test_salt(&env, 1);
    let hash_a = compute_test_hash(&env, 1, guild_b, 200, &salt_a);
    client.submit_strategy_commitment(
        &admin_a,
        &ProofInput {
            commitment_hash: hash_a,
            guild_id: guild_a,
        },
    );
    client.reveal_strategy(
        &admin_a,
        &guild_a,
        &StrategyReveal {
            action_type: 1,
            target_guild_id: guild_b,
            resource_amount: 200,
            salt: salt_a,
        },
    );

    // Guild B has no defense (defense_strength = 0)
    // Resolve battle - attacker wins (10 > 0)
    let treasury_before_b = client.get_guild(&guild_b).treasury;
    let treasury_before_a = client.get_guild(&guild_a).treasury;

    client.resolve_battle(&admin_a, &admin_b, &guild_a, &guild_b);

    // Verify resource transfer
    let guild_a_after = client.get_guild(&guild_a);
    let guild_b_after = client.get_guild(&guild_b);
    assert!(guild_a_after.treasury > treasury_before_a);
    assert!(guild_b_after.treasury < treasury_before_b);

    // Verify game state updated
    let state = client.get_state();
    assert_eq!(state.round, 1);
}

/// Test that a non-member cannot submit a commitment.
#[test]
#[should_panic(expected = "not a guild member")]
fn test_commitment_non_member() {
    let env = Env::default();
    let (_, client, _, guild_id) = setup_guild(&env);

    let outsider = Address::generate(&env);
    let salt = make_test_salt(&env, 1);
    let hash = compute_test_hash(&env, 1, 1, 200, &salt);

    client.submit_strategy_commitment(
        &outsider,
        &ProofInput {
            commitment_hash: hash,
            guild_id,
        },
    );
}
