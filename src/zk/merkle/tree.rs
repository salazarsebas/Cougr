//! SHA256-based Merkle tree construction.

use crate::zk::error::ZKError;
use alloc::vec::Vec;
use soroban_sdk::{BytesN, Env};

/// Maximum tree depth (2^20 = ~1M leaves).
pub const MAX_DEPTH: u32 = 20;

/// SHA256-based Merkle tree (runtime-only, NOT `#[contracttype]`).
///
/// Constructed from a list of leaf hashes. Supports inclusion proof generation.
///
/// # Example
/// ```
/// use cougr_core::zk::stable::MerkleTree;
/// use soroban_sdk::Env;
///
/// let env = Env::default();
/// let leaves = [[1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32]];
/// let tree = MerkleTree::from_leaves(&env, &leaves)?;
/// let proof = tree.proof(2)?;
/// assert_eq!(proof.leaf_index, 2);
/// # Ok::<(), cougr_core::zk::ZKError>(())
/// ```
pub struct MerkleTree {
    depth: u32,
    leaf_count: u32,
    /// layers[0] = leaves, layers[depth] = [root]
    layers: Vec<Vec<[u8; 32]>>,
}

/// In-memory proof representation.
pub struct MerkleProof {
    /// Sibling hashes along the path from leaf to root.
    pub siblings: Vec<[u8; 32]>,
    /// Direction bits: false = left, true = right.
    pub path_indices: Vec<bool>,
    /// The leaf hash.
    pub leaf: [u8; 32],
    /// Index of the leaf in the tree.
    pub leaf_index: u32,
}

impl MerkleTree {
    /// Build a Merkle tree from leaf hashes.
    ///
    /// Leaves are padded to the next power of 2 with zero hashes.
    pub fn from_leaves(env: &Env, leaves: &[[u8; 32]]) -> Result<Self, ZKError> {
        if leaves.is_empty() {
            return Err(ZKError::EmptyTree);
        }

        // Compute depth
        let leaf_count = leaves.len() as u32;
        let mut depth = 0u32;
        let mut size = 1u32;
        while size < leaf_count {
            depth += 1;
            size *= 2;
        }

        if depth > MAX_DEPTH {
            return Err(ZKError::MaxDepthExceeded);
        }

        // Hash each leaf: H(leaf)
        let mut current_layer: Vec<[u8; 32]> = Vec::new();
        for leaf in leaves {
            current_layer.push(hash_leaf(env, leaf));
        }
        // Pad to next power of 2 with zero hashes
        while (current_layer.len() as u32) < size {
            current_layer.push([0u8; 32]);
        }

        let mut layers = Vec::new();
        layers.push(current_layer.clone());

        // Build layers from bottom up
        while current_layer.len() > 1 {
            let mut next_layer = Vec::new();
            for i in (0..current_layer.len()).step_by(2) {
                let left = &current_layer[i];
                let right = &current_layer[i + 1];
                next_layer.push(hash_pair(env, left, right));
            }
            layers.push(next_layer.clone());
            current_layer = next_layer;
        }

        Ok(Self {
            depth,
            leaf_count,
            layers,
        })
    }

    /// Returns the root hash.
    pub fn root(&self) -> [u8; 32] {
        self.layers.last().unwrap()[0]
    }

    /// Returns the root hash as `BytesN<32>`.
    pub fn root_bytes(&self, env: &Env) -> BytesN<32> {
        BytesN::from_array(env, &self.root())
    }

    /// Generate an inclusion proof for the leaf at `leaf_index`.
    pub fn proof(&self, leaf_index: u32) -> Result<MerkleProof, ZKError> {
        if leaf_index >= self.leaf_count {
            return Err(ZKError::LeafOutOfBounds);
        }

        let mut siblings = Vec::new();
        let mut path_indices = Vec::new();
        let mut idx = leaf_index as usize;

        for layer_idx in 0..self.depth as usize {
            let sibling_idx = if idx.is_multiple_of(2) {
                idx + 1
            } else {
                idx - 1
            };
            siblings.push(self.layers[layer_idx][sibling_idx]);
            path_indices.push(!idx.is_multiple_of(2));
            idx /= 2;
        }

        Ok(MerkleProof {
            siblings,
            path_indices,
            leaf: self.layers[0][leaf_index as usize],
            leaf_index,
        })
    }

    /// Returns the tree depth.
    pub fn depth(&self) -> u32 {
        self.depth
    }

    /// Returns the number of original leaves (before padding).
    pub fn leaf_count(&self) -> u32 {
        self.leaf_count
    }
}

/// Hash a leaf: SHA256(0x00 || leaf_data).
/// The 0x00 prefix distinguishes leaf hashes from internal node hashes.
fn hash_leaf(env: &Env, data: &[u8; 32]) -> [u8; 32] {
    let mut input = [0u8; 33];
    input[0] = 0x00;
    input[1..].copy_from_slice(data);
    let bytes = soroban_sdk::Bytes::from_slice(env, &input);
    let result = env.crypto().sha256(&bytes);
    result.to_array()
}

/// Hash two children: SHA256(0x01 || left || right).
/// The 0x01 prefix distinguishes internal hashes from leaf hashes.
fn hash_pair(env: &Env, left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut input = [0u8; 65];
    input[0] = 0x01;
    input[1..33].copy_from_slice(left);
    input[33..65].copy_from_slice(right);
    let bytes = soroban_sdk::Bytes::from_slice(env, &input);
    let result = env.crypto().sha256(&bytes);
    result.to_array()
}

/// Verify a proof against a known root (standalone function).
pub fn verify_proof(env: &Env, proof: &MerkleProof, expected_root: &[u8; 32]) -> bool {
    let mut current = proof.leaf;

    for i in 0..proof.siblings.len() {
        let sibling = &proof.siblings[i];
        if proof.path_indices[i] {
            // Current node is on the right
            current = hash_pair(env, sibling, &current);
        } else {
            // Current node is on the left
            current = hash_pair(env, &current, sibling);
        }
    }

    current == *expected_root
}

// ─── Poseidon2-based Merkle tree ─────────────────────────────────────

/// Poseidon2-based Merkle tree (runtime-only, NOT `#[contracttype]`).
///
/// Uses Poseidon2 hashing for ZK-friendly proofs (~300 constraints per
/// hash vs ~28,000 for SHA256). Requires the `hazmat-crypto` feature.
///
/// Leaves and internal nodes are represented as `U256` field elements.
#[cfg(feature = "hazmat-crypto")]
pub struct PoseidonMerkleTree {
    depth: u32,
    leaf_count: u32,
    layers: alloc::vec::Vec<alloc::vec::Vec<soroban_sdk::U256>>,
}

/// In-memory proof for Poseidon Merkle trees.
#[cfg(feature = "hazmat-crypto")]
pub struct PoseidonMerkleProof {
    pub siblings: alloc::vec::Vec<soroban_sdk::U256>,
    pub path_indices: alloc::vec::Vec<bool>,
    pub leaf: soroban_sdk::U256,
    pub leaf_index: u32,
}

#[cfg(feature = "hazmat-crypto")]
impl PoseidonMerkleTree {
    /// Build a Poseidon Merkle tree from leaf field elements.
    ///
    /// Leaves are padded to the next power of 2 with zero elements.
    pub fn from_leaves(
        env: &Env,
        params: &crate::zk::crypto::Poseidon2Params,
        leaves: &[soroban_sdk::U256],
    ) -> Result<Self, ZKError> {
        if leaves.is_empty() {
            return Err(ZKError::EmptyTree);
        }

        let leaf_count = leaves.len() as u32;
        let mut depth = 0u32;
        let mut size = 1u32;
        while size < leaf_count {
            depth += 1;
            size *= 2;
        }

        if depth > MAX_DEPTH {
            return Err(ZKError::MaxDepthExceeded);
        }

        // Hash each leaf: H(leaf, 0) - domain-separated by zero second input
        let zero = soroban_sdk::U256::from_u32(env, 0);
        let mut current_layer: alloc::vec::Vec<soroban_sdk::U256> = alloc::vec::Vec::new();
        for leaf in leaves {
            current_layer.push(crate::zk::crypto::poseidon2_hash(env, params, leaf, &zero));
        }
        // Pad to next power of 2 with zero elements
        while (current_layer.len() as u32) < size {
            current_layer.push(zero.clone());
        }

        let mut layers = alloc::vec::Vec::new();
        layers.push(current_layer.clone());

        // Build layers from bottom up
        while current_layer.len() > 1 {
            let mut next_layer = alloc::vec::Vec::new();
            for i in (0..current_layer.len()).step_by(2) {
                let left = &current_layer[i];
                let right = &current_layer[i + 1];
                next_layer.push(crate::zk::crypto::poseidon2_hash(env, params, left, right));
            }
            layers.push(next_layer.clone());
            current_layer = next_layer;
        }

        Ok(Self {
            depth,
            leaf_count,
            layers,
        })
    }

    /// Returns the root hash as a `U256` field element.
    pub fn root(&self) -> soroban_sdk::U256 {
        self.layers.last().unwrap()[0].clone()
    }

    /// Generate an inclusion proof for the leaf at `leaf_index`.
    pub fn proof(&self, leaf_index: u32) -> Result<PoseidonMerkleProof, ZKError> {
        if leaf_index >= self.leaf_count {
            return Err(ZKError::LeafOutOfBounds);
        }

        let mut siblings = alloc::vec::Vec::new();
        let mut path_indices = alloc::vec::Vec::new();
        let mut idx = leaf_index as usize;

        for layer_idx in 0..self.depth as usize {
            let sibling_idx = if idx.is_multiple_of(2) {
                idx + 1
            } else {
                idx - 1
            };
            siblings.push(self.layers[layer_idx][sibling_idx].clone());
            path_indices.push(!idx.is_multiple_of(2));
            idx /= 2;
        }

        Ok(PoseidonMerkleProof {
            siblings,
            path_indices,
            leaf: self.layers[0][leaf_index as usize].clone(),
            leaf_index,
        })
    }

    /// Returns the tree depth.
    pub fn depth(&self) -> u32 {
        self.depth
    }

    /// Returns the number of original leaves (before padding).
    pub fn leaf_count(&self) -> u32 {
        self.leaf_count
    }
}

/// Verify a Poseidon Merkle proof against a known root.
#[cfg(feature = "hazmat-crypto")]
pub fn verify_poseidon_proof(
    env: &Env,
    params: &crate::zk::crypto::Poseidon2Params,
    proof: &PoseidonMerkleProof,
    expected_root: &soroban_sdk::U256,
) -> bool {
    let mut current = proof.leaf.clone();

    for i in 0..proof.siblings.len() {
        let sibling = &proof.siblings[i];
        if proof.path_indices[i] {
            // Current node is on the right
            current = crate::zk::crypto::poseidon2_hash(env, params, sibling, &current);
        } else {
            // Current node is on the left
            current = crate::zk::crypto::poseidon2_hash(env, params, &current, sibling);
        }
    }

    current == *expected_root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_leaves_error() {
        let env = Env::default();
        let result = MerkleTree::from_leaves(&env, &[]);
        match result {
            Err(e) => assert_eq!(e, ZKError::EmptyTree),
            Ok(_) => panic!("Expected EmptyTree error"),
        }
    }

    #[test]
    fn test_single_leaf() {
        let env = Env::default();
        let leaf = [1u8; 32];
        let tree = MerkleTree::from_leaves(&env, &[leaf]).unwrap();

        assert_eq!(tree.depth(), 0);
        assert_eq!(tree.leaf_count(), 1);

        // Root is the hash of the single leaf
        let expected_root = hash_leaf(&env, &leaf);
        assert_eq!(tree.root(), expected_root);
    }

    #[test]
    fn test_two_leaves() {
        let env = Env::default();
        let leaves = [[1u8; 32], [2u8; 32]];
        let tree = MerkleTree::from_leaves(&env, &leaves).unwrap();

        assert_eq!(tree.depth(), 1);
        assert_eq!(tree.leaf_count(), 2);

        let h0 = hash_leaf(&env, &leaves[0]);
        let h1 = hash_leaf(&env, &leaves[1]);
        let expected_root = hash_pair(&env, &h0, &h1);
        assert_eq!(tree.root(), expected_root);
    }

    #[test]
    fn test_four_leaves() {
        let env = Env::default();
        let leaves = [[1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32]];
        let tree = MerkleTree::from_leaves(&env, &leaves).unwrap();

        assert_eq!(tree.depth(), 2);
        assert_eq!(tree.leaf_count(), 4);
    }

    #[test]
    fn test_three_leaves_padded() {
        let env = Env::default();
        let leaves = [[1u8; 32], [2u8; 32], [3u8; 32]];
        let tree = MerkleTree::from_leaves(&env, &leaves).unwrap();

        // 3 leaves → depth 2 (padded to 4)
        assert_eq!(tree.depth(), 2);
        assert_eq!(tree.leaf_count(), 3);
    }

    #[test]
    fn test_proof_generation() {
        let env = Env::default();
        let leaves = [[1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32]];
        let tree = MerkleTree::from_leaves(&env, &leaves).unwrap();

        let proof = tree.proof(0).unwrap();
        assert_eq!(proof.leaf_index, 0);
        assert_eq!(proof.siblings.len(), 2);
        assert_eq!(proof.path_indices.len(), 2);
    }

    #[test]
    fn test_proof_out_of_bounds() {
        let env = Env::default();
        let leaves = [[1u8; 32], [2u8; 32]];
        let tree = MerkleTree::from_leaves(&env, &leaves).unwrap();

        let result = tree.proof(2);
        match result {
            Err(e) => assert_eq!(e, ZKError::LeafOutOfBounds),
            Ok(_) => panic!("Expected LeafOutOfBounds error"),
        }
    }

    #[test]
    fn test_proof_verification_valid() {
        let env = Env::default();
        let leaves = [[1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32]];
        let tree = MerkleTree::from_leaves(&env, &leaves).unwrap();
        let root = tree.root();

        // Verify proof for each leaf
        for i in 0..4u32 {
            let proof = tree.proof(i).unwrap();
            assert!(verify_proof(&env, &proof, &root));
        }
    }

    #[test]
    fn test_proof_verification_wrong_root() {
        let env = Env::default();
        let leaves = [[1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32]];
        let tree = MerkleTree::from_leaves(&env, &leaves).unwrap();
        let proof = tree.proof(0).unwrap();

        let wrong_root = [0xFFu8; 32];
        assert!(!verify_proof(&env, &proof, &wrong_root));
    }

    #[test]
    fn test_root_bytes() {
        let env = Env::default();
        let leaves = [[1u8; 32], [2u8; 32]];
        let tree = MerkleTree::from_leaves(&env, &leaves).unwrap();

        let root_bytes = tree.root_bytes(&env);
        assert_eq!(root_bytes.to_array(), tree.root());
    }

    #[test]
    fn test_eight_leaves() {
        let env = Env::default();
        let mut leaves = Vec::new();
        for i in 0..8u8 {
            leaves.push([i; 32]);
        }

        let tree = MerkleTree::from_leaves(&env, &leaves).unwrap();
        assert_eq!(tree.depth(), 3);
        assert_eq!(tree.leaf_count(), 8);

        // Verify all proofs
        let root = tree.root();
        for i in 0..8u32 {
            let proof = tree.proof(i).unwrap();
            assert!(verify_proof(&env, &proof, &root));
        }
    }
}
