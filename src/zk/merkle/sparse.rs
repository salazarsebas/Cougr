//! Sparse Merkle tree for key-value state.
//!
//! A sparse Merkle tree (SMT) represents a key-value map where
//! keys are 256-bit hashes. Most of the tree is "empty" (default values),
//! and only non-empty paths are stored.

use crate::zk::error::ZKError;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use soroban_sdk::{BytesN, Env};

use super::proof::OnChainMerkleProof;

/// Fixed depth for the sparse Merkle tree (256 bits = SHA256 output size).
/// In practice we use a smaller depth for gas efficiency.
const SMT_DEPTH: u32 = 16; // 2^16 = 65536 slots

/// Sparse Merkle tree for key-value state (runtime-only).
///
/// Uses a fixed depth and precomputed default hashes for empty subtrees.
/// Only non-default nodes are stored, keeping memory usage proportional
/// to the number of actual entries.
pub struct SparseMerkleTree {
    root: [u8; 32],
    nodes: BTreeMap<(u32, u32), [u8; 32]>, // (level, index) -> hash
    defaults: Vec<[u8; 32]>,               // precomputed default hashes per level
}

impl SparseMerkleTree {
    /// Create a new empty sparse Merkle tree.
    pub fn new(env: &Env) -> Self {
        let defaults = precompute_defaults(env);
        let root = defaults[SMT_DEPTH as usize];

        Self {
            root,
            nodes: BTreeMap::new(),
            defaults,
        }
    }

    /// Returns the root hash.
    pub fn root(&self) -> [u8; 32] {
        self.root
    }

    /// Returns the root hash as `BytesN<32>`.
    pub fn root_bytes(&self, env: &Env) -> BytesN<32> {
        BytesN::from_array(env, &self.root)
    }

    /// Insert or update a key-value pair and recompute the root.
    ///
    /// The key determines the leaf position (lower 16 bits used as index).
    pub fn insert(&mut self, env: &Env, key: &[u8; 32], value: &[u8; 32]) -> Result<(), ZKError> {
        let leaf_index = key_to_index(key);
        let leaf_hash = hash_leaf(env, value);

        // Set the leaf
        self.nodes.insert((0, leaf_index), leaf_hash);

        // Recompute path from leaf to root
        let mut idx = leaf_index;
        for level in 0..SMT_DEPTH {
            let sibling_idx = if idx.is_multiple_of(2) {
                idx + 1
            } else {
                idx - 1
            };
            let left_idx = if idx.is_multiple_of(2) {
                idx
            } else {
                sibling_idx
            };
            let right_idx = if idx.is_multiple_of(2) {
                sibling_idx
            } else {
                idx
            };

            let left = self.get_node(level, left_idx);
            let right = self.get_node(level, right_idx);
            let parent = hash_pair(env, &left, &right);

            idx /= 2;
            self.nodes.insert((level + 1, idx), parent);
        }

        self.root = self.get_node(SMT_DEPTH, 0);
        Ok(())
    }

    /// Get a value by key, if it exists.
    pub fn get(&self, key: &[u8; 32]) -> Option<[u8; 32]> {
        let leaf_index = key_to_index(key);
        self.nodes.get(&(0, leaf_index)).copied()
    }

    /// Generate an inclusion proof for a key.
    pub fn prove(&self, env: &Env, key: &[u8; 32]) -> OnChainMerkleProof {
        let leaf_index = key_to_index(key);
        let leaf = self.get_node(0, leaf_index);

        let mut siblings: soroban_sdk::Vec<BytesN<32>> = soroban_sdk::Vec::new(env);
        let mut path_bits: u32 = 0;
        let mut idx = leaf_index;

        for level in 0..SMT_DEPTH {
            let sibling_idx = if idx.is_multiple_of(2) {
                idx + 1
            } else {
                idx - 1
            };
            let sibling = self.get_node(level, sibling_idx);
            siblings.push_back(BytesN::from_array(env, &sibling));

            if !idx.is_multiple_of(2) {
                path_bits |= 1 << level;
            }
            idx /= 2;
        }

        OnChainMerkleProof {
            siblings,
            path_bits,
            leaf: BytesN::from_array(env, &leaf),
            leaf_index,
            depth: SMT_DEPTH,
        }
    }

    /// Get a node hash, falling back to the default for that level.
    fn get_node(&self, level: u32, index: u32) -> [u8; 32] {
        self.nodes
            .get(&(level, index))
            .copied()
            .unwrap_or(self.defaults[level as usize])
    }
}

/// Map a 32-byte key to a leaf index (lower bits).
fn key_to_index(key: &[u8; 32]) -> u32 {
    let b0 = key[0] as u32;
    let b1 = key[1] as u32;
    (b0 | (b1 << 8)) % (1 << SMT_DEPTH)
}

/// Precompute default hashes for each level of the tree.
/// Level 0 default = all zeros (empty leaf).
/// Level n default = H(default[n-1], default[n-1]).
fn precompute_defaults(env: &Env) -> Vec<[u8; 32]> {
    let mut defaults = Vec::with_capacity(SMT_DEPTH as usize + 1);
    defaults.push([0u8; 32]); // level 0: empty leaf

    for _ in 0..SMT_DEPTH {
        let prev = defaults.last().unwrap();
        defaults.push(hash_pair(env, prev, prev));
    }

    defaults
}

/// Hash a leaf: SHA256(0x00 || data).
fn hash_leaf(env: &Env, data: &[u8; 32]) -> [u8; 32] {
    let mut input = [0u8; 33];
    input[0] = 0x00;
    input[1..].copy_from_slice(data);
    let bytes = soroban_sdk::Bytes::from_slice(env, &input);
    env.crypto().sha256(&bytes).to_array()
}

/// Hash two children: SHA256(0x01 || left || right).
fn hash_pair(env: &Env, left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut input = [0u8; 65];
    input[0] = 0x01;
    input[1..33].copy_from_slice(left);
    input[33..65].copy_from_slice(right);
    let bytes = soroban_sdk::Bytes::from_slice(env, &input);
    env.crypto().sha256(&bytes).to_array()
}

// ─── Poseidon2-based Sparse Merkle Tree ─────────────────────────────

/// Poseidon2-based sparse Merkle tree for ZK-friendly key-value state.
///
/// Requires the `hazmat-crypto` feature.
#[cfg(feature = "hazmat-crypto")]
pub struct PoseidonSparseMerkleTree {
    root: soroban_sdk::U256,
    nodes: BTreeMap<(u32, u32), soroban_sdk::U256>,
    defaults: Vec<soroban_sdk::U256>,
}

#[cfg(feature = "hazmat-crypto")]
impl PoseidonSparseMerkleTree {
    /// Create a new empty Poseidon sparse Merkle tree.
    pub fn new(env: &Env, params: &crate::zk::crypto::Poseidon2Params) -> Self {
        let defaults = precompute_poseidon_defaults(env, params);
        let root = defaults[SMT_DEPTH as usize].clone();

        Self {
            root,
            nodes: BTreeMap::new(),
            defaults,
        }
    }

    /// Returns the root hash.
    pub fn root(&self) -> soroban_sdk::U256 {
        self.root.clone()
    }

    /// Insert or update a key-value pair and recompute the root.
    pub fn insert(
        &mut self,
        env: &Env,
        params: &crate::zk::crypto::Poseidon2Params,
        key: &[u8; 32],
        value: &soroban_sdk::U256,
    ) -> Result<(), crate::zk::error::ZKError> {
        let leaf_index = key_to_index(key);
        let zero = soroban_sdk::U256::from_u32(env, 0);
        let leaf_hash = crate::zk::crypto::poseidon2_hash(env, params, value, &zero);

        self.nodes.insert((0, leaf_index), leaf_hash);

        let mut idx = leaf_index;
        for level in 0..SMT_DEPTH {
            let sibling_idx = if idx.is_multiple_of(2) {
                idx + 1
            } else {
                idx - 1
            };
            let left_idx = if idx.is_multiple_of(2) {
                idx
            } else {
                sibling_idx
            };
            let right_idx = if idx.is_multiple_of(2) {
                sibling_idx
            } else {
                idx
            };

            let left = self.get_node(level, left_idx);
            let right = self.get_node(level, right_idx);
            let parent = crate::zk::crypto::poseidon2_hash(env, params, &left, &right);

            idx /= 2;
            self.nodes.insert((level + 1, idx), parent);
        }

        self.root = self.get_node(SMT_DEPTH, 0);
        Ok(())
    }

    /// Get a node hash, falling back to the default for that level.
    fn get_node(&self, level: u32, index: u32) -> soroban_sdk::U256 {
        self.nodes
            .get(&(level, index))
            .cloned()
            .unwrap_or_else(|| self.defaults[level as usize].clone())
    }
}

/// Precompute default Poseidon hashes for each level of the sparse tree.
#[cfg(feature = "hazmat-crypto")]
fn precompute_poseidon_defaults(
    env: &Env,
    params: &crate::zk::crypto::Poseidon2Params,
) -> Vec<soroban_sdk::U256> {
    let mut defaults = Vec::with_capacity(SMT_DEPTH as usize + 1);
    defaults.push(soroban_sdk::U256::from_u32(env, 0)); // level 0: empty leaf

    for _ in 0..SMT_DEPTH {
        let prev = defaults.last().unwrap();
        defaults.push(crate::zk::crypto::poseidon2_hash(env, params, prev, prev));
    }

    defaults
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zk::merkle::proof::verify_inclusion;

    #[test]
    fn test_empty_smt() {
        let env = Env::default();
        let smt = SparseMerkleTree::new(&env);
        let root = smt.root();
        // Root of empty tree is the precomputed default at depth 16
        assert_ne!(root, [0u8; 32]); // it's H(H(H(...))) not raw zeros
    }

    #[test]
    fn test_insert_and_get() {
        let env = Env::default();
        let mut smt = SparseMerkleTree::new(&env);

        let key = [1u8; 32];
        let value = [42u8; 32];

        smt.insert(&env, &key, &value).unwrap();
        let stored = smt.get(&key);
        // get() returns the leaf hash, not the raw value
        assert!(stored.is_some());
    }

    #[test]
    fn test_insert_changes_root() {
        let env = Env::default();
        let mut smt = SparseMerkleTree::new(&env);
        let initial_root = smt.root();

        smt.insert(&env, &[1u8; 32], &[42u8; 32]).unwrap();
        assert_ne!(smt.root(), initial_root);
    }

    #[test]
    fn test_different_keys_different_roots() {
        let env = Env::default();

        let mut smt1 = SparseMerkleTree::new(&env);
        smt1.insert(&env, &[1u8; 32], &[42u8; 32]).unwrap();

        let mut smt2 = SparseMerkleTree::new(&env);
        smt2.insert(&env, &[2u8; 32], &[42u8; 32]).unwrap();

        assert_ne!(smt1.root(), smt2.root());
    }

    #[test]
    fn test_prove_and_verify() {
        let env = Env::default();
        let mut smt = SparseMerkleTree::new(&env);

        let key = [5u8; 32];
        let value = [99u8; 32];
        smt.insert(&env, &key, &value).unwrap();

        let root = smt.root_bytes(&env);
        let proof = smt.prove(&env, &key);

        let result = verify_inclusion(&env, &proof, &root).unwrap();
        assert!(result);
    }

    #[test]
    fn test_prove_empty_key() {
        let env = Env::default();
        let smt = SparseMerkleTree::new(&env);

        let key = [0u8; 32];
        let root = smt.root_bytes(&env);
        let proof = smt.prove(&env, &key);

        // Proof for empty key should verify (it's a valid default path)
        let result = verify_inclusion(&env, &proof, &root).unwrap();
        assert!(result);
    }

    #[test]
    fn test_multiple_inserts() {
        let env = Env::default();
        let mut smt = SparseMerkleTree::new(&env);

        for i in 0..10u8 {
            let mut key = [0u8; 32];
            key[0] = i;
            let mut value = [0u8; 32];
            value[0] = i + 100;
            smt.insert(&env, &key, &value).unwrap();
        }

        // Verify all 10 proofs
        let root = smt.root_bytes(&env);
        for i in 0..10u8 {
            let mut key = [0u8; 32];
            key[0] = i;
            let proof = smt.prove(&env, &key);
            let result = verify_inclusion(&env, &proof, &root).unwrap();
            assert!(result, "Proof failed for key {}", i);
        }
    }

    #[test]
    fn test_update_existing_key() {
        let env = Env::default();
        let mut smt = SparseMerkleTree::new(&env);

        let key = [1u8; 32];
        smt.insert(&env, &key, &[10u8; 32]).unwrap();
        let root1 = smt.root();

        smt.insert(&env, &key, &[20u8; 32]).unwrap();
        let root2 = smt.root();

        // Updating value should change root
        assert_ne!(root1, root2);

        // New proof should verify
        let proof = smt.prove(&env, &key);
        let root = smt.root_bytes(&env);
        assert!(verify_inclusion(&env, &proof, &root).unwrap());
    }
}
