//! Fair dice helpers for Cougr ZK circuits.
//!
//! WARNING: This module uses embedded proving keys that are for TESTING ONLY.
//! Do NOT use these keys in a production game; a trusted setup is required.

use cougr_core::circuits::fair_dice;
use cougr_core::zk::Groth16Proof;
use soroban_sdk::{BytesN, Env};

/// Verify a dice roll against a seed commitment.
pub fn verify_roll(
    env: &Env,
    sides: u32,
    seed_commitment: &BytesN<32>,
    roll_result: u32,
    nonce: u32,
    proof: &Groth16Proof,
) -> bool {
    fair_dice(env, sides, seed_commitment)
        .map(|spec| spec.verify_dice_roll(env, proof, roll_result, nonce).unwrap_or(false))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cougr_core::circuits::{test_fixtures, CircuitId};
    use soroban_sdk::Env;

    #[test]
    fn test_verify_roll_fixture() {
        let env = Env::default();
        let seed = test_fixtures::pipeline_public_bytes32(&env, CircuitId::FairDice);
        let proof = test_fixtures::pipeline_proof(&env, CircuitId::FairDice);
        
        let verified = verify_roll(&env, 6, &seed, 6, 5, &proof);
        assert!(verified, "Fixture roll should verify");
    }

    #[test]
    fn test_reject_tampered_roll() {
        let env = Env::default();
        let seed = test_fixtures::pipeline_public_bytes32(&env, CircuitId::FairDice);
        let proof = test_fixtures::pipeline_proof(&env, CircuitId::FairDice);
        
        let verified = verify_roll(&env, 6, &seed, 5, 5, &proof);
        assert!(!verified, "Tampered roll should be rejected");
    }
}
