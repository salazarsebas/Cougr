//! Zero-knowledge proof support for Cougr.
//!
//! This module provides ergonomic wrappers around Stellar Protocol 25 (X-Ray)
//! cryptographic host functions for use in on-chain game verification.
//!
//! ## Architecture
//!
//! - **`stable`**: stable privacy primitives and interfaces
//! - **`experimental`**: advanced proof-verification flows and automation
//! - **`types`**: core ZK types (`G1Point`, `G2Point`, `Scalar`, `Groth16Proof`, `VerificationKey`)
//! - **`crypto`**: low-level BN254 and Poseidon wrappers
//! - **`groth16`**: Groth16 proof verification contract (experimental implementation)
//! - **`error`**: ZK-specific error types
//! - **`testing`**: Mock types for unit testing without real proofs
//!
//! ## Maturity Split
//!
//! Stable privacy surface:
//!
//! - commitments
//! - commit-reveal
//! - hidden-state encoding
//! - Merkle inclusion and sparse Merkle utilities
//! - privacy interfaces such as `CommitmentScheme`, `MerkleProofVerifier`, and `HiddenStateCodec`
//!
//! Experimental privacy surface:
//!
//! - Groth16 verification flows
//! - proof-submission execution helpers
//! - prebuilt verification circuits
//! - hazmat Poseidon-based privacy tooling
//!
//! ## Usage
//!
//! ```no_run
//! use cougr_core::zk::experimental::{verify_groth16, Groth16Proof, VerificationKey};
//! use cougr_core::zk::{G1Point, G2Point, Scalar};
//! use soroban_sdk::{BytesN, Env, Vec};
//!
//! let env = Env::default();
//! let g1 = G1Point { bytes: BytesN::from_array(&env, &[0u8; 64]) };
//! let g2 = G2Point { bytes: BytesN::from_array(&env, &[0u8; 128]) };
//! let vk = VerificationKey {
//!     alpha: g1.clone(),
//!     beta: g2.clone(),
//!     gamma: g2.clone(),
//!     delta: g2,
//!     ic: Vec::from_array(&env, [g1.clone()]),
//! };
//! let proof = Groth16Proof { a: g1.clone(), b: vk.beta.clone(), c: g1 };
//! let public_inputs: [Scalar; 0] = [];
//! let _result = verify_groth16(&env, &vk, &proof, &public_inputs);
//! ```

pub(crate) mod advanced;
pub(crate) mod batch;
pub(crate) mod bls12_381;
pub(crate) mod circuits;
pub(crate) mod commitment;
pub(crate) mod components;
pub(crate) mod crypto;
pub(crate) mod error;
pub mod experimental;
pub(crate) mod groth16;
pub(crate) mod interfaces;
pub(crate) mod interop;
pub(crate) mod merkle;
pub mod stable;
pub(crate) mod systems;
pub mod testing;
pub(crate) mod traits;
pub(crate) mod types;

// Stable-by-default root exports.
pub use error::ZKError;
pub use stable::{
    commit_reveal_deadline_system, encode_commit_reveal, pedersen_commit, pedersen_verify,
    verify_inclusion, verify_inclusion_at_depth, Bytes32HiddenStateCodec, CommitReveal,
    CommitmentScheme, HiddenState, HiddenStateCodec, MerkleProof, MerkleProofVerifier, MerkleTree,
    OnChainMerkleProof, PedersenCommitment, PedersenCommitmentScheme, PedersenParams,
    ProofVerifier, Sha256MerkleProofVerifier, SparseMerkleTree, COMMIT_REVEAL_TYPE,
    HIDDEN_STATE_TYPE,
};
pub use types::{
    Bls12381G1Point, Bls12381G2Point, Bls12381Scalar, G1Point, G2Point, Groth16Proof, Scalar,
    VerificationKey,
};
