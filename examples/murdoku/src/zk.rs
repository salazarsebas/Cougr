//! Groth16 zero-knowledge proof verification for Murdoku solution proofs.
//!
//! # Circuit Statement (informal)
//!
//! "I know a set of values `cells` such that:
//!   (1) `cells` is a valid Latin square of size N,
//!   (2) Poseidon2(cells || salt) == commitment,
//!   (3) each cell value is in range 1..=N."
//!
//! # Public Inputs Format
//!
//! - `public_inputs[0]`: The solution commitment (`BytesN<32>`) - the Poseidon2
//!   hash of the flat solution vector concatenated with a 32-byte salt chosen by
//!   the puzzle creator.
//! - `public_inputs[1..]`: Additional public inputs as required by the circuit
//!   (e.g., grid size, clue commitments).
//!
//! # Trusted Setup Limitation
//!
//! Groth16 requires a trusted setup ceremony for each circuit. The verifier keys
//! currently used during development are generated from a **development setup**
//! and are **not** production-safe. A proper multi-party trusted setup ceremony
//! must be completed before deploying this module in a production environment.
//! See the Murdoku README for more details.
//!
//! # Dependencies
//!
//! Requires the `zk` feature flag (which enables `hazmat-crypto` on cougr-core):
//! ```toml
//! murdoku = { features = ["zk"] }
//! ```

use alloc::vec::Vec as AllocVec;

use cougr_core::privacy::experimental::{verify_groth16, Groth16Proof, VerificationKey};
use cougr_core::privacy::Scalar;
use soroban_sdk::xdr::FromXdr;
use soroban_sdk::{panic_with_error, Address, Bytes, BytesN, Env, Symbol, Vec};

use crate::PuzzleError;

/// Load the stored Groth16 verifier key for the given puzzle.
pub fn load_verifier_key(env: &Env, puzzle_id: u32) -> VerificationKey {
    let key = (Symbol::new(env, "VKEY"), puzzle_id);
    let vk_bytes: Bytes = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| panic_with_error!(env, PuzzleError::PuzzleNotFound));
    VerificationKey::from_xdr(env, &vk_bytes)
        .unwrap_or_else(|_| panic_with_error!(env, PuzzleError::InvalidSolution))
}

/// Store the verifier key (as XDR-serialized bytes) for a puzzle.
pub fn store_verifier_key(env: &Env, puzzle_id: u32, vk_bytes: &Bytes) {
    let key = (Symbol::new(env, "VKEY"), puzzle_id);
    env.storage().persistent().set(&key, vk_bytes);
}

/// Mark a player's session as solved for the given puzzle.
pub fn mark_solved(env: &Env, puzzle_id: u32, player: &Address) {
    let solved_key = (Symbol::new(env, "SOLVED"), puzzle_id, player.clone());
    env.storage().persistent().set(&solved_key, &true);

    let count_key = (Symbol::new(env, "SOLVER_CNT"), puzzle_id);
    let mut count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);
    count += 1;
    env.storage().persistent().set(&count_key, &count);
}

/// Check whether a player has already solved the given puzzle.
pub fn is_solved(env: &Env, puzzle_id: u32, player: &Address) -> bool {
    let solved_key = (Symbol::new(env, "SOLVED"), puzzle_id, player.clone());
    env.storage().persistent().get(&solved_key).unwrap_or(false)
}

/// Verify a Groth16 solution proof for the given puzzle.
///
/// # Arguments
/// - `env`: Soroban environment
/// - `puzzle_id`: The puzzle ID
/// - `proof`: XDR-serialized `Groth16Proof`
/// - `public_inputs`: Public inputs as 32-byte values (first element should be the
///   solution commitment)
///
/// # Returns
/// - `true` if the Groth16 proof verifies successfully against the stored key
/// - `false` if the proof is malformed, the key is missing, or verification fails
pub fn verify_solution_proof(
    env: &Env,
    puzzle_id: u32,
    proof: Bytes,
    public_inputs: Vec<BytesN<32>>,
) -> bool {
    let vk = match env
        .storage()
        .persistent()
        .get::<_, Bytes>(&(Symbol::new(env, "VKEY"), puzzle_id))
    {
        Some(bytes) => match VerificationKey::from_xdr(env, &bytes) {
            Ok(k) => k,
            Err(_) => return false,
        },
        None => return false,
    };

    let groth16_proof: Groth16Proof = match Groth16Proof::from_xdr(env, &proof) {
        Ok(p) => p,
        Err(_) => return false,
    };

    let mut scalars: AllocVec<Scalar> = AllocVec::with_capacity(public_inputs.len() as usize);
    for i in 0..public_inputs.len() {
        scalars.push(Scalar {
            bytes: public_inputs.get(i).unwrap(),
        });
    }

    matches!(verify_groth16(env, &vk, &groth16_proof, &scalars), Ok(true))
}
