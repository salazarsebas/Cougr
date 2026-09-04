//! Experimental phase-3 ZK patterns.
//!
//! These APIs make the phase-3 roadmap concrete without overstating maturity:
//!
//! - fog-of-war orchestration around Merkle roots
//! - multiplayer state-channel transition contracts
//! - recursive proof-composition descriptors
//!
//! They remain part of `zk::experimental` because the repository is only
//! committing to explicit orchestration and public-input contracts here, not to
//! production-ready confidentiality guarantees.

use alloc::vec::Vec;
use soroban_sdk::{contracttype, BytesN, Env};

use super::error::ZKError;
use super::merkle::tree::MerkleTree;
use super::traits::{
    bytes32_to_scalar, field_i32_to_scalar, field_u32_to_scalar, u32_to_scalar, u64_to_scalar,
    GameCircuit,
};
use super::types::{Groth16Proof, Scalar, VerificationKey};

/// Snapshot of a player's currently visible fog-of-war state.
#[contracttype]
#[derive(Clone, Debug)]
pub struct FogOfWarSnapshot {
    /// Merkle root of the hidden map or board state.
    pub map_root: BytesN<32>,
    /// Merkle root of the tiles the player has already explored.
    pub explored_root: BytesN<32>,
    /// Player origin used by the exploration circuit.
    pub origin_x: i32,
    pub origin_y: i32,
    /// Maximum Euclidean distance the player may reveal from the origin.
    pub visibility_radius: u32,
}

impl FogOfWarSnapshot {
    /// Returns `true` when the target tile is within the visible window.
    pub fn can_reveal(&self, tile_x: i32, tile_y: i32) -> bool {
        let dx = i64::from(tile_x) - i64::from(self.origin_x);
        let dy = i64::from(tile_y) - i64::from(self.origin_y);
        let radius = i64::from(self.visibility_radius);

        dx * dx + dy * dy <= radius * radius
    }
}

/// Root transition for a single fog-of-war exploration update.
#[contracttype]
#[derive(Clone, Debug)]
pub struct FogOfWarTransition {
    pub prior_explored_root: BytesN<32>,
    pub next_explored_root: BytesN<32>,
    pub tile_x: i32,
    pub tile_y: i32,
}

/// Apply a validated exploration update to a fog-of-war snapshot.
pub fn apply_fog_of_war_transition(
    snapshot: &FogOfWarSnapshot,
    transition: &FogOfWarTransition,
) -> Result<FogOfWarSnapshot, ZKError> {
    if snapshot.explored_root != transition.prior_explored_root {
        return Err(ZKError::InvalidStateTransition);
    }

    if !snapshot.can_reveal(transition.tile_x, transition.tile_y) {
        return Err(ZKError::InvalidVisibility);
    }

    let mut updated = snapshot.clone();
    updated.explored_root = transition.next_explored_root.clone();
    Ok(updated)
}

/// Experimental circuit contract for fog-of-war exploration proofs.
///
/// Prefer [`crate::circuits::fog_of_war`] for new integrations - it returns a
/// [`crate::circuits::GameCircuitSpec`] with the same verification path.
pub struct FogOfWarCircuit {
    pub vk: VerificationKey,
    pub max_visibility_radius: u32,
}

impl GameCircuit for FogOfWarCircuit {
    fn verification_key(&self) -> &VerificationKey {
        &self.vk
    }
}

impl FogOfWarCircuit {
    pub fn new(vk: VerificationKey, max_visibility_radius: u32) -> Self {
        Self {
            vk,
            max_visibility_radius,
        }
    }

    /// Verify that a fog-of-war transition is valid for the provided snapshot.
    ///
    /// Public inputs:
    /// `[map_root, prior_explored_root, next_explored_root, origin_x, origin_y, tile_x, tile_y, visibility_radius]`.
    pub fn verify_exploration(
        &self,
        env: &Env,
        proof: &Groth16Proof,
        snapshot: &FogOfWarSnapshot,
        transition: &FogOfWarTransition,
    ) -> Result<bool, ZKError> {
        if snapshot.visibility_radius > self.max_visibility_radius {
            return Err(ZKError::InvalidVisibility);
        }

        let _ = apply_fog_of_war_transition(snapshot, transition)?;

        let public_inputs = Vec::from([
            bytes32_to_scalar(&snapshot.map_root),
            bytes32_to_scalar(&transition.prior_explored_root),
            bytes32_to_scalar(&transition.next_explored_root),
            field_i32_to_scalar(env, snapshot.origin_x),
            field_i32_to_scalar(env, snapshot.origin_y),
            field_i32_to_scalar(env, transition.tile_x),
            field_i32_to_scalar(env, transition.tile_y),
            field_u32_to_scalar(env, snapshot.visibility_radius),
        ]);

        self.verify_with_inputs(env, proof, &public_inputs)
    }
}

/// Off-chain state channel tracked by on-chain commitments and dispute metadata.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ZkStateChannel {
    pub channel_id: BytesN<32>,
    pub participants_root: BytesN<32>,
    pub state_root: BytesN<32>,
    pub round: u64,
    pub dispute_deadline: u64,
    pub closed: bool,
}

/// Proposed state transition for a multiplayer ZK state channel.
#[contracttype]
#[derive(Clone, Debug)]
pub struct StateChannelTransition {
    pub prior_state_root: BytesN<32>,
    pub next_state_root: BytesN<32>,
    pub round: u64,
    pub submitted_at: u64,
}

/// Open a new experimental ZK state channel.
pub fn open_state_channel(
    channel_id: BytesN<32>,
    participants_root: BytesN<32>,
    initial_state_root: BytesN<32>,
    dispute_deadline: u64,
) -> Result<ZkStateChannel, ZKError> {
    if dispute_deadline == 0 {
        return Err(ZKError::InvalidInput);
    }

    Ok(ZkStateChannel {
        channel_id,
        participants_root,
        state_root: initial_state_root,
        round: 0,
        dispute_deadline,
        closed: false,
    })
}

/// Apply a verified transition to the state channel.
pub fn apply_state_channel_transition(
    channel: &ZkStateChannel,
    transition: &StateChannelTransition,
) -> Result<ZkStateChannel, ZKError> {
    if channel.closed {
        return Err(ZKError::ChannelClosed);
    }

    if transition.prior_state_root != channel.state_root {
        return Err(ZKError::InvalidStateTransition);
    }

    let expected_round = channel
        .round
        .checked_add(1)
        .ok_or(ZKError::InvalidStateTransition)?;
    if transition.round != expected_round {
        return Err(ZKError::InvalidStateTransition);
    }

    if transition.submitted_at > channel.dispute_deadline {
        return Err(ZKError::DeadlineExpired);
    }

    let mut updated = channel.clone();
    updated.state_root = transition.next_state_root.clone();
    updated.round = transition.round;
    Ok(updated)
}

/// Close a channel with the latest accepted state root.
pub fn close_state_channel(
    channel: &ZkStateChannel,
    final_state_root: &BytesN<32>,
    final_round: u64,
    closed_at: u64,
) -> Result<ZkStateChannel, ZKError> {
    if channel.closed {
        return Err(ZKError::ChannelClosed);
    }

    if final_round < channel.round {
        return Err(ZKError::InvalidStateTransition);
    }

    let mut closed = channel.clone();
    closed.state_root = final_state_root.clone();
    closed.round = final_round;
    closed.dispute_deadline = closed_at;
    closed.closed = true;
    Ok(closed)
}

/// Experimental circuit contract for channel transition proofs.
pub struct StateChannelCircuit {
    pub vk: VerificationKey,
}

impl GameCircuit for StateChannelCircuit {
    fn verification_key(&self) -> &VerificationKey {
        &self.vk
    }
}

impl StateChannelCircuit {
    pub fn new(vk: VerificationKey) -> Self {
        Self { vk }
    }

    /// Verify a state transition for a channel.
    ///
    /// Public inputs:
    /// `[channel_id, participants_root, prior_state_root, next_state_root, round, submitted_at]`.
    pub fn verify_transition(
        &self,
        env: &Env,
        proof: &Groth16Proof,
        channel: &ZkStateChannel,
        transition: &StateChannelTransition,
    ) -> Result<bool, ZKError> {
        if channel.closed {
            return Err(ZKError::ChannelClosed);
        }

        let _ = apply_state_channel_transition(channel, transition)?;
        let public_inputs = Vec::from([
            bytes32_to_scalar(&channel.channel_id),
            bytes32_to_scalar(&channel.participants_root),
            bytes32_to_scalar(&transition.prior_state_root),
            bytes32_to_scalar(&transition.next_state_root),
            u64_to_scalar(env, transition.round),
            u64_to_scalar(env, transition.submitted_at),
        ]);

        self.verify_with_inputs(env, proof, &public_inputs)
    }
}

/// Experimental descriptor for a recursive proof batch.
#[contracttype]
#[derive(Clone, Debug)]
pub struct RecursiveProofLayout {
    pub initial_state_root: BytesN<32>,
    pub final_state_root: BytesN<32>,
    pub accumulator_root: BytesN<32>,
    pub proof_count: u32,
}

impl RecursiveProofLayout {
    /// Build a layout by folding per-step statement roots into a Merkle accumulator.
    pub fn from_step_roots(
        env: &Env,
        initial_state_root: BytesN<32>,
        final_state_root: BytesN<32>,
        step_roots: &[BytesN<32>],
    ) -> Result<Self, ZKError> {
        let accumulator_root = compose_statement_roots(env, step_roots)?;
        Ok(Self {
            initial_state_root,
            final_state_root,
            accumulator_root,
            proof_count: step_roots.len() as u32,
        })
    }
}

/// Fold per-step statement roots into a deterministic Merkle accumulator root.
pub fn compose_statement_roots(
    env: &Env,
    step_roots: &[BytesN<32>],
) -> Result<BytesN<32>, ZKError> {
    if step_roots.is_empty() {
        return Err(ZKError::InvalidProofComposition);
    }

    let mut leaves = Vec::with_capacity(step_roots.len());
    for root in step_roots {
        leaves.push(root.to_array());
    }

    let tree = MerkleTree::from_leaves(env, &leaves)?;
    Ok(tree.root_bytes(env))
}

/// Experimental circuit contract for recursive proof aggregation.
pub struct RecursiveProofCircuit {
    pub vk: VerificationKey,
    pub max_proof_count: u32,
}

impl GameCircuit for RecursiveProofCircuit {
    fn verification_key(&self) -> &VerificationKey {
        &self.vk
    }
}

impl RecursiveProofCircuit {
    pub fn new(vk: VerificationKey, max_proof_count: u32) -> Self {
        Self {
            vk,
            max_proof_count,
        }
    }

    /// Verify an aggregated recursive-proof layout.
    ///
    /// Public inputs:
    /// `[initial_state_root, final_state_root, accumulator_root, proof_count]`.
    pub fn verify_composition(
        &self,
        env: &Env,
        proof: &Groth16Proof,
        layout: &RecursiveProofLayout,
    ) -> Result<bool, ZKError> {
        if layout.proof_count == 0 || layout.proof_count > self.max_proof_count {
            return Err(ZKError::InvalidProofComposition);
        }

        let public_inputs: [Scalar; 4] = [
            bytes32_to_scalar(&layout.initial_state_root),
            bytes32_to_scalar(&layout.final_state_root),
            bytes32_to_scalar(&layout.accumulator_root),
            u32_to_scalar(env, layout.proof_count),
        ];

        self.verify_with_inputs(env, proof, &public_inputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zk::types::{G1Point, G2Point};

    fn make_vk(env: &Env, ic_count: u32) -> VerificationKey {
        let g1 = G1Point {
            bytes: BytesN::from_array(env, &[0u8; 64]),
        };
        let g2 = G2Point {
            bytes: BytesN::from_array(env, &[0u8; 128]),
        };
        let mut ic = soroban_sdk::Vec::new(env);
        for _ in 0..ic_count {
            ic.push_back(g1.clone());
        }

        VerificationKey {
            alpha: g1.clone(),
            beta: g2.clone(),
            gamma: g2.clone(),
            delta: g2,
            ic,
        }
    }

    fn make_proof(env: &Env) -> Groth16Proof {
        let g1 = G1Point {
            bytes: BytesN::from_array(env, &[0u8; 64]),
        };
        let g2 = G2Point {
            bytes: BytesN::from_array(env, &[0u8; 128]),
        };

        Groth16Proof {
            a: g1.clone(),
            b: g2,
            c: g1,
        }
    }

    #[test]
    fn test_apply_fog_of_war_transition_rejects_hidden_tile() {
        let env = Env::default();
        let snapshot = FogOfWarSnapshot {
            map_root: BytesN::from_array(&env, &[1u8; 32]),
            explored_root: BytesN::from_array(&env, &[2u8; 32]),
            origin_x: 0,
            origin_y: 0,
            visibility_radius: 2,
        };
        let transition = FogOfWarTransition {
            prior_explored_root: snapshot.explored_root.clone(),
            next_explored_root: BytesN::from_array(&env, &[3u8; 32]),
            tile_x: 3,
            tile_y: 0,
        };

        let result = apply_fog_of_war_transition(&snapshot, &transition);
        assert!(matches!(result, Err(ZKError::InvalidVisibility)));
    }

    #[test]
    fn test_fog_of_war_circuit_rejects_snapshot_above_max_radius() {
        let env = Env::default();
        let circuit = FogOfWarCircuit::new(make_vk(&env, 9), 3);
        let snapshot = FogOfWarSnapshot {
            map_root: BytesN::from_array(&env, &[1u8; 32]),
            explored_root: BytesN::from_array(&env, &[2u8; 32]),
            origin_x: 0,
            origin_y: 0,
            visibility_radius: 4,
        };
        let transition = FogOfWarTransition {
            prior_explored_root: snapshot.explored_root.clone(),
            next_explored_root: BytesN::from_array(&env, &[3u8; 32]),
            tile_x: 1,
            tile_y: 1,
        };

        let result = circuit.verify_exploration(&env, &make_proof(&env), &snapshot, &transition);
        assert_eq!(result, Err(ZKError::InvalidVisibility));
    }

    #[test]
    fn test_open_state_channel_requires_deadline() {
        let env = Env::default();
        let result = open_state_channel(
            BytesN::from_array(&env, &[1u8; 32]),
            BytesN::from_array(&env, &[2u8; 32]),
            BytesN::from_array(&env, &[3u8; 32]),
            0,
        );

        assert!(matches!(result, Err(ZKError::InvalidInput)));
    }

    #[test]
    fn test_apply_state_channel_transition_rejects_wrong_round() {
        let env = Env::default();
        let channel = open_state_channel(
            BytesN::from_array(&env, &[1u8; 32]),
            BytesN::from_array(&env, &[2u8; 32]),
            BytesN::from_array(&env, &[3u8; 32]),
            10,
        )
        .unwrap();
        let transition = StateChannelTransition {
            prior_state_root: channel.state_root.clone(),
            next_state_root: BytesN::from_array(&env, &[4u8; 32]),
            round: 2,
            submitted_at: 5,
        };

        let result = apply_state_channel_transition(&channel, &transition);
        assert!(matches!(result, Err(ZKError::InvalidStateTransition)));
    }

    #[test]
    fn test_state_channel_circuit_rejects_closed_channel() {
        let env = Env::default();
        let channel = ZkStateChannel {
            channel_id: BytesN::from_array(&env, &[1u8; 32]),
            participants_root: BytesN::from_array(&env, &[2u8; 32]),
            state_root: BytesN::from_array(&env, &[3u8; 32]),
            round: 1,
            dispute_deadline: 5,
            closed: true,
        };
        let transition = StateChannelTransition {
            prior_state_root: channel.state_root.clone(),
            next_state_root: BytesN::from_array(&env, &[4u8; 32]),
            round: 2,
            submitted_at: 5,
        };
        let circuit = StateChannelCircuit::new(make_vk(&env, 7));

        let result = circuit.verify_transition(&env, &make_proof(&env), &channel, &transition);
        assert_eq!(result, Err(ZKError::ChannelClosed));
    }

    #[test]
    fn test_compose_statement_roots_is_deterministic() {
        let env = Env::default();
        let steps = [
            BytesN::from_array(&env, &[1u8; 32]),
            BytesN::from_array(&env, &[2u8; 32]),
            BytesN::from_array(&env, &[3u8; 32]),
        ];

        let root_a = compose_statement_roots(&env, &steps).unwrap();
        let root_b = compose_statement_roots(&env, &steps).unwrap();
        assert_eq!(root_a, root_b);
    }

    #[test]
    fn test_recursive_proof_layout_requires_non_empty_steps() {
        let env = Env::default();
        let result = RecursiveProofLayout::from_step_roots(
            &env,
            BytesN::from_array(&env, &[1u8; 32]),
            BytesN::from_array(&env, &[2u8; 32]),
            &[],
        );

        assert!(matches!(result, Err(ZKError::InvalidProofComposition)));
    }

    #[test]
    fn test_recursive_proof_circuit_rejects_out_of_bounds_proof_count() {
        let env = Env::default();
        let circuit = RecursiveProofCircuit::new(make_vk(&env, 5), 2);
        let layout = RecursiveProofLayout {
            initial_state_root: BytesN::from_array(&env, &[1u8; 32]),
            final_state_root: BytesN::from_array(&env, &[2u8; 32]),
            accumulator_root: BytesN::from_array(&env, &[3u8; 32]),
            proof_count: 3,
        };

        let result = circuit.verify_composition(&env, &make_proof(&env), &layout);
        assert_eq!(result, Err(ZKError::InvalidProofComposition));
    }
}
