use soroban_sdk::contracterror;

/// Error types for the Cougr ZK subsystem.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ZKError {
    /// The submitted proof is structurally invalid.
    InvalidProof = 10,
    /// An elliptic curve point is not on the curve or not in the subgroup.
    InvalidPoint = 11,
    /// A scalar value is out of range for the target field.
    InvalidScalar = 12,
    /// Proof verification failed (valid structure, but proof is incorrect).
    VerificationFailed = 13,
    /// Input data is malformed or has the wrong length.
    InvalidInput = 14,
    /// The verification key is malformed or incompatible with the proof.
    InvalidVerificationKey = 15,
    /// The circuit type does not match the expected verification key.
    CircuitMismatch = 16,
    /// Public inputs do not match the circuit's expected format.
    InvalidPublicInput = 17,
    /// Merkle tree cannot be constructed from empty leaves.
    EmptyTree = 18,
    /// Leaf index is out of bounds for the tree.
    LeafOutOfBounds = 19,
    /// Merkle proof has invalid length (doesn't match tree depth).
    InvalidProofLength = 20,
    /// Merkle inclusion proof verification failed.
    MerkleVerificationFailed = 21,
    /// Tree depth exceeds the maximum allowed depth.
    MaxDepthExceeded = 22,
    /// Leaf data is invalid or malformed.
    InvalidLeaf = 23,
    /// A state transition violates the explicit phase-3 orchestration contract.
    InvalidStateTransition = 24,
    /// A transition arrived after its allowed dispute or reveal window.
    DeadlineExpired = 25,
    /// The requested operation cannot proceed because the channel is already closed.
    ChannelClosed = 26,
    /// A fog-of-war exploration target lies outside the allowed visibility window.
    InvalidVisibility = 27,
    /// A recursive composition descriptor is malformed or exceeds declared bounds.
    InvalidProofComposition = 28,
    /// A Merkle proof's path does not lead to the leaf index it claims.
    LeafIndexMismatch = 29,
}
