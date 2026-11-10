//! Merkle tree utilities for on-chain state verification.
//!
//! Provides Merkle tree construction, inclusion proofs,
//! and sparse Merkle tree for key-value state spaces.
//!
//! # Architecture
//!
//! - **`tree`**: In-memory Merkle tree construction from leaves (SHA256 + Poseidon2)
//! - **`proof`**: On-chain proof types and verification
//! - **`sparse`**: Sparse Merkle tree for large state spaces (SHA256 + Poseidon2)
//!
//! Trees are computed in-memory; only the root is stored on-chain.
//! Proofs are compact and can be verified on-chain.
//!
//! ## Poseidon2 variants
//!
//! Enable the `hazmat-crypto` feature to use `PoseidonMerkleTree` and
//! `PoseidonSparseMerkleTree` for ZK-friendly proofs (~300 constraints
//! per hash vs ~28,000 for SHA256).

pub mod proof;
pub mod sparse;
pub mod tree;

pub use proof::{verify_inclusion, verify_inclusion_at_depth, OnChainMerkleProof};
pub use sparse::SparseMerkleTree;
pub use tree::{MerkleProof, MerkleTree};

#[cfg(feature = "hazmat-crypto")]
pub use sparse::PoseidonSparseMerkleTree;
#[cfg(feature = "hazmat-crypto")]
pub use tree::{verify_poseidon_proof, PoseidonMerkleProof, PoseidonMerkleTree};
