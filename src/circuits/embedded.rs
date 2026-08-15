//! Pipeline-embedded test VKs and proofs for on-chain Groth16 verification.
//!
//! Regenerate via `internal/cougr-core-circuits`: `bun run pipeline`.

#[cfg(any(test, feature = "testutils"))]
use soroban_sdk::BytesN;
use soroban_sdk::Env;

use crate::zk::types::VerificationKey;
#[cfg(any(test, feature = "testutils"))]
use crate::zk::types::{Groth16Proof, Scalar};

use super::generated;
#[cfg(any(test, feature = "testutils"))]
use super::generated_fixtures;
use super::CircuitId;

/// Load the development verification key produced by the Circom pipeline.
pub fn pipeline_verification_key(env: &Env, circuit_id: CircuitId) -> VerificationKey {
    match circuit_id {
        CircuitId::HiddenCards => vk_from_parts(
            env,
            &generated::HIDDEN_CARDS_VK_ALPHA,
            &generated::HIDDEN_CARDS_VK_BETA,
            &generated::HIDDEN_CARDS_VK_GAMMA,
            &generated::HIDDEN_CARDS_VK_DELTA,
            &generated::HIDDEN_CARDS_VK_IC,
            generated::HIDDEN_CARDS_VK_N_PUBLIC,
        ),
        CircuitId::FogOfWar => vk_from_parts(
            env,
            &generated::FOG_OF_WAR_VK_ALPHA,
            &generated::FOG_OF_WAR_VK_BETA,
            &generated::FOG_OF_WAR_VK_GAMMA,
            &generated::FOG_OF_WAR_VK_DELTA,
            &generated::FOG_OF_WAR_VK_IC,
            generated::FOG_OF_WAR_VK_N_PUBLIC,
        ),
        CircuitId::FairDice => vk_from_parts(
            env,
            &generated::FAIR_DICE_VK_ALPHA,
            &generated::FAIR_DICE_VK_BETA,
            &generated::FAIR_DICE_VK_GAMMA,
            &generated::FAIR_DICE_VK_DELTA,
            &generated::FAIR_DICE_VK_IC,
            generated::FAIR_DICE_VK_N_PUBLIC,
        ),
        CircuitId::SealedBid => vk_from_parts(
            env,
            &generated::SEALED_BID_VK_ALPHA,
            &generated::SEALED_BID_VK_BETA,
            &generated::SEALED_BID_VK_GAMMA,
            &generated::SEALED_BID_VK_DELTA,
            &generated::SEALED_BID_VK_IC,
            generated::SEALED_BID_VK_N_PUBLIC,
        ),
    }
}

/// Load a pipeline proof fixture for integration tests.
#[cfg(any(test, feature = "testutils"))]
pub fn pipeline_proof(env: &Env, circuit_id: CircuitId) -> Groth16Proof {
    let (a, b, c) = match circuit_id {
        CircuitId::HiddenCards => (
            &generated_fixtures::HIDDEN_CARDS_PROOF_A,
            &generated_fixtures::HIDDEN_CARDS_PROOF_B,
            &generated_fixtures::HIDDEN_CARDS_PROOF_C,
        ),
        CircuitId::FogOfWar => (
            &generated_fixtures::FOG_OF_WAR_PROOF_A,
            &generated_fixtures::FOG_OF_WAR_PROOF_B,
            &generated_fixtures::FOG_OF_WAR_PROOF_C,
        ),
        CircuitId::FairDice => (
            &generated_fixtures::FAIR_DICE_PROOF_A,
            &generated_fixtures::FAIR_DICE_PROOF_B,
            &generated_fixtures::FAIR_DICE_PROOF_C,
        ),
        CircuitId::SealedBid => (
            &generated_fixtures::SEALED_BID_PROOF_A,
            &generated_fixtures::SEALED_BID_PROOF_B,
            &generated_fixtures::SEALED_BID_PROOF_C,
        ),
    };
    Groth16Proof::from_raw_bytes(env, a, b, c)
}

/// Frozen public inputs from the pipeline witness (big-endian Fr scalars).
#[cfg(any(test, feature = "testutils"))]
pub fn pipeline_public_inputs(env: &Env, circuit_id: CircuitId) -> alloc::vec::Vec<Scalar> {
    let rows: &[[u8; 32]] = match circuit_id {
        CircuitId::HiddenCards => &generated_fixtures::HIDDEN_CARDS_PUBLIC_INPUTS,
        CircuitId::FogOfWar => &generated_fixtures::FOG_OF_WAR_PUBLIC_INPUTS,
        CircuitId::FairDice => &generated_fixtures::FAIR_DICE_PUBLIC_INPUTS,
        CircuitId::SealedBid => &generated_fixtures::SEALED_BID_PUBLIC_INPUTS,
    };
    rows.iter()
        .map(|bytes| Scalar {
            bytes: BytesN::from_array(env, bytes),
        })
        .collect()
}

/// First public input as `BytesN<32>` (e.g. seed or deck commitment).
#[cfg(any(test, feature = "testutils"))]
pub fn pipeline_public_bytes32(env: &Env, circuit_id: CircuitId) -> BytesN<32> {
    let inputs = pipeline_public_inputs(env, circuit_id);
    inputs
        .first()
        .map(|s| s.bytes.clone())
        .unwrap_or_else(|| BytesN::from_array(env, &[0u8; 32]))
}

fn vk_from_parts(
    env: &Env,
    alpha: &[u8; 64],
    beta: &[u8; 128],
    gamma: &[u8; 128],
    delta: &[u8; 128],
    ic: &[[u8; 64]],
    n_public: u32,
) -> VerificationKey {
    let expected_ic = n_public.saturating_add(1);
    debug_assert_eq!(
        ic.len() as u32,
        expected_ic,
        "pipeline VK IC length must equal nPublic + 1"
    );
    let _ = (n_public, ic.len());
    VerificationKey::from_raw_bytes(env, alpha, beta, gamma, delta, ic)
}
