use soroban_sdk::{Bytes, BytesN, Env};

use super::commitment::{pedersen_commit, pedersen_verify, PedersenCommitment, PedersenParams};
use super::error::ZKError;
use super::groth16::verify_groth16;
use super::merkle::proof::{verify_inclusion, OnChainMerkleProof};
use super::types::{Groth16Proof, Scalar, VerificationKey};

/// Stable interface for commitment schemes used by privacy primitives.
///
/// The trait contract is intentionally narrow:
/// - implementations must define exact parameter and opening semantics
/// - malformed inputs must return `Err(ZKError)` rather than silently succeeding
/// - `verify` must return `Ok(false)` only for a well-formed but invalid opening
pub trait CommitmentScheme {
    type Parameters;
    type Value;
    type Opening;
    type Commitment;

    fn commit(
        &self,
        env: &Env,
        params: &Self::Parameters,
        value: &Self::Value,
        opening: &Self::Opening,
    ) -> Result<Self::Commitment, ZKError>;

    fn verify(
        &self,
        env: &Env,
        params: &Self::Parameters,
        commitment: &Self::Commitment,
        value: &Self::Value,
        opening: &Self::Opening,
    ) -> Result<bool, ZKError>;
}

/// Stable interface for Merkle inclusion proof verification.
///
/// Implementations must reject malformed proofs with `Err(ZKError)` and
/// return `Ok(false)` only when the proof is well-formed but does not match
/// the expected root.
pub trait MerkleProofVerifier {
    type Proof;
    type Root;

    fn verify(
        &self,
        env: &Env,
        proof: &Self::Proof,
        expected_root: &Self::Root,
    ) -> Result<bool, ZKError>;
}

/// Stable interface for hidden-state encoding.
///
/// Hidden-state codecs define the exact byte-level representation used before
/// a caller commits to or stores private state metadata.
pub trait HiddenStateCodec {
    type State;

    fn encode(&self, env: &Env, state: &Self::State) -> Result<Bytes, ZKError>;
    fn decode(&self, env: &Env, encoded: &Bytes) -> Result<Self::State, ZKError>;
}

/// Stable interface for proof verifiers.
///
/// The interface itself is stable even when specific proof systems are not.
/// This allows callers to depend on a narrow verification contract while
/// choosing whether an implementation belongs to the stable or experimental
/// privacy surface.
pub trait ProofVerifier {
    type VerificationKey;
    type Proof;
    type PublicInput;

    fn verify(
        &self,
        env: &Env,
        verification_key: &Self::VerificationKey,
        proof: &Self::Proof,
        public_inputs: &[Self::PublicInput],
    ) -> Result<bool, ZKError>;
}

/// Stable Pedersen commitment scheme adapter.
#[derive(Clone, Copy, Debug, Default)]
pub struct PedersenCommitmentScheme;

impl CommitmentScheme for PedersenCommitmentScheme {
    type Parameters = PedersenParams;
    type Value = Scalar;
    type Opening = Scalar;
    type Commitment = PedersenCommitment;

    fn commit(
        &self,
        env: &Env,
        params: &Self::Parameters,
        value: &Self::Value,
        opening: &Self::Opening,
    ) -> Result<Self::Commitment, ZKError> {
        pedersen_commit(env, params, value, opening)
    }

    fn verify(
        &self,
        env: &Env,
        params: &Self::Parameters,
        commitment: &Self::Commitment,
        value: &Self::Value,
        opening: &Self::Opening,
    ) -> Result<bool, ZKError> {
        pedersen_verify(env, params, commitment, value, opening)
    }
}

/// Stable SHA256 Merkle inclusion verifier adapter.
#[derive(Clone, Copy, Debug, Default)]
pub struct Sha256MerkleProofVerifier;

impl MerkleProofVerifier for Sha256MerkleProofVerifier {
    type Proof = OnChainMerkleProof;
    type Root = BytesN<32>;

    fn verify(
        &self,
        env: &Env,
        proof: &Self::Proof,
        expected_root: &Self::Root,
    ) -> Result<bool, ZKError> {
        verify_inclusion(env, proof, expected_root)
    }
}

/// Fixed-width hidden-state codec for 32-byte payloads.
///
/// This is intentionally conservative: it only accepts a `BytesN<32>` state,
/// ensuring the encoded representation is deterministic and length-safe.
#[derive(Clone, Copy, Debug, Default)]
pub struct Bytes32HiddenStateCodec;

impl HiddenStateCodec for Bytes32HiddenStateCodec {
    type State = BytesN<32>;

    fn encode(&self, env: &Env, state: &Self::State) -> Result<Bytes, ZKError> {
        Ok(Bytes::from_slice(env, &state.to_array()))
    }

    fn decode(&self, env: &Env, encoded: &Bytes) -> Result<Self::State, ZKError> {
        if encoded.len() != 32 {
            return Err(ZKError::InvalidInput);
        }

        let mut bytes = [0u8; 32];
        for i in 0..32u32 {
            bytes[i as usize] = encoded.get(i).ok_or(ZKError::InvalidInput)?;
        }
        Ok(BytesN::from_array(env, &bytes))
    }
}

/// Experimental Groth16 verifier adapter.
///
/// The interface is explicit, but the underlying proof system remains
/// experimental until its assumptions and host-function behavior are hardened
/// further for a stable contract claim.
#[derive(Clone, Copy, Debug, Default)]
pub struct Groth16ProofVerifier;

impl ProofVerifier for Groth16ProofVerifier {
    type VerificationKey = VerificationKey;
    type Proof = Groth16Proof;
    type PublicInput = Scalar;

    fn verify(
        &self,
        env: &Env,
        verification_key: &Self::VerificationKey,
        proof: &Self::Proof,
        public_inputs: &[Self::PublicInput],
    ) -> Result<bool, ZKError> {
        verify_groth16(env, verification_key, proof, public_inputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{BytesN, Env, Vec};

    #[test]
    fn test_bytes32_hidden_state_codec_roundtrip() {
        let env = Env::default();
        let codec = Bytes32HiddenStateCodec;
        let state = BytesN::from_array(&env, &[0xAB; 32]);

        let encoded = codec.encode(&env, &state).unwrap();
        let decoded = codec.decode(&env, &encoded).unwrap();

        assert_eq!(decoded, state);
    }

    #[test]
    fn test_bytes32_hidden_state_codec_rejects_wrong_length() {
        let env = Env::default();
        let codec = Bytes32HiddenStateCodec;
        let encoded = Bytes::from_slice(&env, &[1, 2, 3]);

        let result = codec.decode(&env, &encoded);
        assert_eq!(result, Err(ZKError::InvalidInput));
    }

    #[test]
    fn test_sha256_merkle_verifier_rejects_malformed_proof() {
        let env = Env::default();
        let verifier = Sha256MerkleProofVerifier;
        let proof = OnChainMerkleProof {
            siblings: Vec::new(&env),
            path_bits: 0,
            leaf: BytesN::from_array(&env, &[1u8; 32]),
            leaf_index: 0,
            depth: 1,
        };
        let root = BytesN::from_array(&env, &[0u8; 32]);

        let result = verifier.verify(&env, &proof, &root);
        assert_eq!(result, Err(ZKError::InvalidProofLength));
    }
}
