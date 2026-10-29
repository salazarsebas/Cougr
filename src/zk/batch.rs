//! Proof batching state helpers for `TurnSequenceCircuit` patterns.
//!
//! [`BatchProofContext`] carries the inputs needed to generate a
//! `TurnSequenceCircuit` proof off-chain. The on-chain contract creates a
//! context at the start of a turn batch, calls [`BatchProofContext::record_action`]
//! for each validated action, then calls [`BatchProofContext::finalize`] to
//! obtain the `(initial_root, final_root, action_count)` tuple expected by the
//! circuit's public inputs.
//!
//! [`hash_state_root`] combines component byte blobs into a single SHA-256
//! digest. No `hazmat-crypto` feature is required.
//!
//! ## Example
//! ```no_run
//! use cougr_core::zk::experimental::{BatchProofContext, hash_state_root};
//! use soroban_sdk::{Bytes, Env};
//!
//! let env = Env::default();
//! let components: &[Bytes] = &[];
//! let initial_root = hash_state_root(&env, components);
//! let ctx = BatchProofContext::new(initial_root)
//!     .record_action()
//!     .record_action();
//! let final_root = hash_state_root(&env, components);
//! let (init, fin, count) = ctx.finalize(final_root);
//! assert_eq!(count, 2);
//! ```

use soroban_sdk::{Bytes, BytesN, Env};

/// Hash a sequence of component byte blobs into a single 32-byte state root.
///
/// All component bytes are concatenated in order and then SHA-256 hashed.
/// The result is deterministic and stable across contract invocations.
pub fn hash_state_root(env: &Env, components: &[Bytes]) -> BytesN<32> {
    let mut combined = Bytes::new(env);
    for c in components {
        combined.append(c);
    }
    env.crypto().sha256(&combined).into()
}

/// Tracks the inputs needed to generate a `TurnSequenceCircuit` proof off-chain.
///
/// Create at the start of a batch, record each action, then finalize to get
/// the public inputs tuple for the circuit.
pub struct BatchProofContext {
    pub initial_state_root: BytesN<32>,
    pub action_count: u32,
}

impl BatchProofContext {
    /// Create a new context from the state root computed before any actions.
    pub fn new(initial_state_root: BytesN<32>) -> Self {
        Self {
            initial_state_root,
            action_count: 0,
        }
    }

    /// Record that one action was validated. Consumes and returns self for
    /// chaining.
    pub fn record_action(mut self) -> Self {
        self.action_count += 1;
        self
    }

    /// Finalize the batch. Returns `(initial_root, current_root, action_count)`
    /// for submission as circuit public inputs.
    pub fn finalize(self, current_state_root: BytesN<32>) -> (BytesN<32>, BytesN<32>, u32) {
        (
            self.initial_state_root,
            current_state_root,
            self.action_count,
        )
    }
}
