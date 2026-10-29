use soroban_sdk::{contracttype, Address, BytesN, Symbol, Vec};

use super::types::{Groth16Proof, Scalar};

/// Stores a Poseidon hash commitment of private game state.
///
/// Used for fog-of-war, hidden inventories, or any state that
/// should be verifiable without revealing the actual data.
#[contracttype]
#[derive(Clone, Debug)]
pub struct HiddenState {
    /// Poseidon hash of the private state.
    pub commitment: BytesN<32>,
    /// Owner of this hidden state.
    pub owner: Address,
}

/// A pending proof submission attached to an entity.
///
/// The proof must be verified before the deadline. Once verified,
/// a `VerifiedMarker` is added and this component is removed.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProofSubmission {
    /// The Groth16 proof to be verified.
    pub proof: Groth16Proof,
    /// Public inputs for verification.
    pub public_inputs: Vec<Scalar>,
    /// Ledger timestamp when the proof was submitted.
    pub submitted_at: u64,
    /// Deadline ledger timestamp for verification.
    pub deadline: u64,
    /// Whether this proof has been verified.
    pub verified: bool,
}

/// Marker component for entities with verified proofs.
///
/// Added after a `ProofSubmission` passes verification.
/// Can be cleaned up after a configurable age.
#[contracttype]
#[derive(Clone, Debug)]
pub struct VerifiedMarker {
    /// Ledger timestamp when verification occurred.
    pub verified_at: u64,
    /// Type/category of the proof that was verified.
    pub proof_type: Symbol,
}

/// Commit-reveal two-phase pattern component.
///
/// Phase 1 (Commit): Player submits a hash of their action.
/// Phase 2 (Reveal): Player reveals the actual action + nonce.
/// If the reveal deadline passes without a reveal, the commitment expires.
#[contracttype]
#[derive(Clone, Debug)]
pub struct CommitReveal {
    /// Hash of the committed action (e.g., Poseidon(action || nonce)).
    pub commitment: BytesN<32>,
    /// Deadline for revealing the committed action.
    pub reveal_deadline: u64,
    /// Whether the action has been revealed.
    pub revealed: bool,
}

/// Well-known component type symbol for `HiddenState`.
pub const HIDDEN_STATE_TYPE: &str = "zk_hidden";

/// Well-known component type symbol for `ProofSubmission`.
pub const PROOF_SUBMISSION_TYPE: &str = "zk_proof";

/// Well-known component type symbol for `VerifiedMarker`.
pub const VERIFIED_MARKER_TYPE: &str = "zk_veri";

/// Well-known component type symbol for `CommitReveal`.
pub const COMMIT_REVEAL_TYPE: &str = "zk_cr";

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, testutils::Address as _, vec, BytesN, Env};

    #[test]
    fn test_hidden_state_creation() {
        let env = Env::default();
        let state = HiddenState {
            commitment: BytesN::from_array(&env, &[0xABu8; 32]),
            owner: Address::generate(&env),
        };
        assert_eq!(state.commitment.len(), 32);
    }

    #[test]
    fn test_proof_submission_creation() {
        let env = Env::default();
        let g1 = super::super::types::G1Point {
            bytes: BytesN::from_array(&env, &[0u8; 64]),
        };
        let g2 = super::super::types::G2Point {
            bytes: BytesN::from_array(&env, &[0u8; 128]),
        };
        let proof = Groth16Proof {
            a: g1.clone(),
            b: g2,
            c: g1,
        };
        let submission = ProofSubmission {
            proof,
            public_inputs: vec![&env],
            submitted_at: 100,
            deadline: 200,
            verified: false,
        };
        assert!(!submission.verified);
        assert_eq!(submission.submitted_at, 100);
    }

    #[test]
    fn test_verified_marker() {
        let _env = Env::default();
        let marker = VerifiedMarker {
            verified_at: 150,
            proof_type: symbol_short!("movement"),
        };
        assert_eq!(marker.verified_at, 150);
    }

    #[test]
    fn test_commit_reveal() {
        let env = Env::default();
        let cr = CommitReveal {
            commitment: BytesN::from_array(&env, &[0xCDu8; 32]),
            reveal_deadline: 500,
            revealed: false,
        };
        assert!(!cr.revealed);
        assert_eq!(cr.reveal_deadline, 500);
    }
}
