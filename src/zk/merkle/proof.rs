//! On-chain Merkle proof types and verification.

use crate::zk::error::ZKError;
use soroban_sdk::{contracttype, BytesN, Env, Vec};

use super::tree::{MerkleProof, MAX_DEPTH};

/// On-chain proof representation (`#[contracttype]` for contract arguments).
#[contracttype]
#[derive(Clone, Debug)]
pub struct OnChainMerkleProof {
    /// Sibling hashes along the path.
    pub siblings: Vec<BytesN<32>>,
    /// Packed direction bits (bit i = 1 means current node was on the right).
    pub path_bits: u32,
    /// The leaf hash.
    pub leaf: BytesN<32>,
    /// Index of the leaf in the tree.
    pub leaf_index: u32,
    /// Tree depth.
    pub depth: u32,
}

/// Convert an in-memory proof to the on-chain format.
pub fn to_on_chain_proof(proof: &MerkleProof, env: &Env) -> OnChainMerkleProof {
    let mut siblings: Vec<BytesN<32>> = Vec::new(env);
    let mut path_bits: u32 = 0;

    for (i, sibling) in proof.siblings.iter().enumerate() {
        siblings.push_back(BytesN::from_array(env, sibling));
        if proof.path_indices[i] {
            path_bits |= 1 << i;
        }
    }

    OnChainMerkleProof {
        siblings,
        path_bits,
        leaf: BytesN::from_array(env, &proof.leaf),
        leaf_index: proof.leaf_index,
        depth: proof.siblings.len() as u32,
    }
}

/// Verify a Merkle inclusion proof against a known root (on-chain function).
///
/// Uses SHA256 hashing with domain separation:
/// - Leaf hash: SHA256(0x00 || leaf_data)
/// - Internal hash: SHA256(0x01 || left || right)
pub fn verify_inclusion(
    env: &Env,
    proof: &OnChainMerkleProof,
    expected_root: &BytesN<32>,
) -> Result<bool, ZKError> {
    if proof.depth > MAX_DEPTH {
        return Err(ZKError::MaxDepthExceeded);
    }

    if proof.siblings.len() != proof.depth {
        return Err(ZKError::InvalidProofLength);
    }

    if proof.leaf_index >= (1u32 << proof.depth) {
        return Err(ZKError::LeafOutOfBounds);
    }

    let path_mask = (1u32 << proof.depth) - 1;
    if proof.path_bits & path_mask != proof.leaf_index {
        return Err(ZKError::LeafIndexMismatch);
    }

    let mut current = proof.leaf.to_array();

    for i in 0..proof.depth {
        let sibling = proof.siblings.get(i).ok_or(ZKError::InvalidProofLength)?;
        let sibling_arr = sibling.to_array();

        let is_right = (proof.path_bits >> i) & 1 == 1;

        if is_right {
            current = hash_pair_raw(env, &sibling_arr, &current);
        } else {
            current = hash_pair_raw(env, &current, &sibling_arr);
        }
    }

    Ok(current == expected_root.to_array())
}

/// Verify a Merkle inclusion proof, additionally pinning the proof to an
/// exact expected depth.
///
/// Use this instead of [`verify_inclusion`] whenever the caller knows the
/// tree's depth ahead of time (for example, a fixed-size game board). A
/// proof whose `depth` does not match `expected_depth` is rejected before
/// any hashing happens, closing the gap where a shorter-than-expected proof
/// (including a zero-depth proof with `leaf == root`) would otherwise verify.
pub fn verify_inclusion_at_depth(
    env: &Env,
    proof: &OnChainMerkleProof,
    expected_root: &BytesN<32>,
    expected_depth: u32,
) -> Result<bool, ZKError> {
    if proof.depth != expected_depth {
        return Err(ZKError::InvalidProofLength);
    }

    verify_inclusion(env, proof, expected_root)
}

/// Hash two children: SHA256(0x01 || left || right).
fn hash_pair_raw(env: &Env, left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut input = [0u8; 65];
    input[0] = 0x01;
    input[1..33].copy_from_slice(left);
    input[33..65].copy_from_slice(right);
    let bytes = soroban_sdk::Bytes::from_slice(env, &input);
    let result = env.crypto().sha256(&bytes);
    result.to_array()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zk::merkle::tree::MerkleTree;

    #[test]
    fn test_to_on_chain_proof_and_verify() {
        let env = Env::default();
        let leaves = [[1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32]];
        let tree = MerkleTree::from_leaves(&env, &leaves).unwrap();
        let root = tree.root_bytes(&env);

        for i in 0..4u32 {
            let proof = tree.proof(i).unwrap();
            let on_chain = to_on_chain_proof(&proof, &env);

            let result = verify_inclusion(&env, &on_chain, &root).unwrap();
            assert!(result, "Proof verification failed for leaf {}", i);
        }
    }

    #[test]
    fn test_verify_inclusion_wrong_root() {
        let env = Env::default();
        let leaves = [[1u8; 32], [2u8; 32]];
        let tree = MerkleTree::from_leaves(&env, &leaves).unwrap();

        let proof = tree.proof(0).unwrap();
        let on_chain = to_on_chain_proof(&proof, &env);

        let wrong_root = BytesN::from_array(&env, &[0xFFu8; 32]);
        let result = verify_inclusion(&env, &on_chain, &wrong_root).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_verify_inclusion_invalid_depth() {
        let env = Env::default();
        let proof = OnChainMerkleProof {
            siblings: Vec::new(&env),
            path_bits: 0,
            leaf: BytesN::from_array(&env, &[1u8; 32]),
            leaf_index: 0,
            depth: 5, // mismatch: 0 siblings but depth 5
        };

        let root = BytesN::from_array(&env, &[0u8; 32]);
        let result = verify_inclusion(&env, &proof, &root);
        assert_eq!(result.unwrap_err(), ZKError::InvalidProofLength);
    }

    #[test]
    fn test_on_chain_proof_path_bits_packing() {
        let env = Env::default();
        let leaves = [[1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32]];
        let tree = MerkleTree::from_leaves(&env, &leaves).unwrap();

        // Leaf 0: path = [false, false] → path_bits = 0b00
        let p0 = to_on_chain_proof(&tree.proof(0).unwrap(), &env);
        assert_eq!(p0.path_bits, 0);

        // Leaf 1: path = [true, false] → path_bits = 0b01
        let p1 = to_on_chain_proof(&tree.proof(1).unwrap(), &env);
        assert_eq!(p1.path_bits, 1);

        // Leaf 2: path = [false, true] → path_bits = 0b10
        let p2 = to_on_chain_proof(&tree.proof(2).unwrap(), &env);
        assert_eq!(p2.path_bits, 2);

        // Leaf 3: path = [true, true] → path_bits = 0b11
        let p3 = to_on_chain_proof(&tree.proof(3).unwrap(), &env);
        assert_eq!(p3.path_bits, 3);
    }

    #[test]
    fn test_verify_inclusion_rejects_relabelled_leaf_index() {
        let env = Env::default();
        let leaves = [[1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32]];
        let tree = MerkleTree::from_leaves(&env, &leaves).unwrap();
        let root = tree.root_bytes(&env);

        // A genuine proof for leaf 1, relabelled to claim leaf 2. The path
        // bits still walk position 1, so the two no longer agree.
        let mut proof = to_on_chain_proof(&tree.proof(1).unwrap(), &env);
        proof.leaf_index = 2;

        let result = verify_inclusion(&env, &proof, &root);
        assert_eq!(result.unwrap_err(), ZKError::LeafIndexMismatch);
    }

    #[test]
    fn test_verify_inclusion_at_depth_rejects_short_proof() {
        let env = Env::default();
        let leaves = [[1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32]];
        let tree = MerkleTree::from_leaves(&env, &leaves).unwrap();
        let root = tree.root_bytes(&env);

        // A degenerate depth-0 proof claiming the root itself is the leaf.
        // `verify_inclusion` alone finds it internally consistent (0
        // siblings, leaf_index 0 < 2^0, path_bits 0 == leaf_index), so it
        // only becomes wrong once the caller states the depth it expects.
        let degenerate = OnChainMerkleProof {
            siblings: Vec::new(&env),
            path_bits: 0,
            leaf: root.clone(),
            leaf_index: 0,
            depth: 0,
        };
        assert!(verify_inclusion(&env, &degenerate, &root).unwrap());

        let result = verify_inclusion_at_depth(&env, &degenerate, &root, tree.depth());
        assert_eq!(result.unwrap_err(), ZKError::InvalidProofLength);
    }

    #[test]
    fn test_verify_inclusion_rejects_depth_over_max_without_panicking() {
        let env = Env::default();
        let proof = OnChainMerkleProof {
            siblings: Vec::new(&env),
            path_bits: 0,
            leaf: BytesN::from_array(&env, &[0u8; 32]),
            leaf_index: 0,
            // Exceeds both MAX_DEPTH (20) and the 32-bit width of path_bits;
            // must return an error rather than panic on the shift below.
            depth: 33,
        };
        let root = BytesN::from_array(&env, &[0u8; 32]);

        let result = verify_inclusion(&env, &proof, &root);
        assert_eq!(result.unwrap_err(), ZKError::MaxDepthExceeded);
    }

    #[test]
    fn test_verify_inclusion_rejects_out_of_bounds_leaf_index() {
        let env = Env::default();
        let leaves = [[1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32]];
        let tree = MerkleTree::from_leaves(&env, &leaves).unwrap();
        let root = tree.root_bytes(&env);

        let mut proof = to_on_chain_proof(&tree.proof(1).unwrap(), &env);
        // depth 2 allows indices 0..=3; 4 is out of bounds regardless of
        // what path_bits says.
        proof.leaf_index = 4;

        let result = verify_inclusion(&env, &proof, &root);
        assert_eq!(result.unwrap_err(), ZKError::LeafOutOfBounds);
    }
}
