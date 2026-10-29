use crate::simple_world::SimpleWorld;
use soroban_sdk::{Bytes, BytesN, Env, Symbol};

use super::components::{COMMIT_REVEAL_TYPE, VERIFIED_MARKER_TYPE};
use super::interfaces::{Groth16ProofVerifier, ProofVerifier};
use super::types::{Groth16Proof, Scalar, VerificationKey};

/// Read a big-endian `u64` from `data` at byte offset `offset`.
fn read_u64(data: &Bytes, offset: u32) -> u64 {
    let mut arr = [0u8; 8];
    for i in 0..8u32 {
        arr[i as usize] = data.get(offset + i).unwrap();
    }
    u64::from_be_bytes(arr)
}

/// Read a boolean from `data` at byte offset `offset`.
fn read_bool(data: &Bytes, offset: u32) -> bool {
    data.get(offset).unwrap() != 0
}

/// Encode a verified-marker component as `Bytes`.
///
/// Layout: `[verified_at: u64 (8 bytes, big-endian)]`.
pub fn encode_verified_marker(env: &Env, verified_at: u64) -> Bytes {
    Bytes::from_slice(env, &verified_at.to_be_bytes())
}

/// Decode a verified-marker component's `verified_at` timestamp.
pub fn decode_verified_at(data: &Bytes) -> u64 {
    read_u64(data, 0)
}

/// Encode a commit-reveal component as `Bytes`.
///
/// Layout: `[commitment: 32 bytes | reveal_deadline: u64 (8 bytes) | revealed: u8 (1 byte)]`.
/// Total: 41 bytes.
pub fn encode_commit_reveal(
    env: &Env,
    commitment: &BytesN<32>,
    reveal_deadline: u64,
    revealed: bool,
) -> Bytes {
    let mut b: Bytes = commitment.clone().into();
    b.append(&Bytes::from_slice(env, &reveal_deadline.to_be_bytes()));
    b.push_back(if revealed { 1 } else { 0 });
    b
}

/// Verify a proof for a specific entity and mark it as verified on success.
///
/// Unlike a world-scanning system, this takes the proof as a parameter
/// (proofs are large and should not be stored in the ECS). On successful
/// verification, a `VerifiedMarker` component is added to the entity.
///
/// Returns `true` if the proof was valid.
pub fn verify_proofs_with<
    V: ProofVerifier<VerificationKey = VerificationKey, Proof = Groth16Proof, PublicInput = Scalar>,
>(
    world: &mut SimpleWorld,
    env: &Env,
    entity_id: u32,
    verifier: &V,
    vk: &VerificationKey,
    proof: &Groth16Proof,
    public_inputs: &[Scalar],
) -> Result<bool, super::error::ZKError> {
    let verified_sym = Symbol::new(env, VERIFIED_MARKER_TYPE);
    let is_valid = verifier.verify(env, vk, proof, public_inputs)?;

    if is_valid {
        let now = env.ledger().timestamp();
        let marker_data = encode_verified_marker(env, now);
        world.add_component(entity_id, verified_sym, marker_data);
    }

    Ok(is_valid)
}

pub fn verify_proofs_system(
    world: &mut SimpleWorld,
    env: &Env,
    entity_id: u32,
    vk: &VerificationKey,
    proof: &Groth16Proof,
    public_inputs: &[Scalar],
) -> bool {
    verify_proofs_with(
        world,
        env,
        entity_id,
        &Groth16ProofVerifier,
        vk,
        proof,
        public_inputs,
    )
    .unwrap_or(false)
}

/// Check for expired commit-reveal deadlines.
///
/// Removes `CommitReveal` components that have passed their deadline
/// without being revealed.
pub fn commit_reveal_deadline_system(world: &mut SimpleWorld, env: &Env) {
    let cr_sym = Symbol::new(env, COMMIT_REVEAL_TYPE);
    let entities = world.get_entities_with_component(&cr_sym, env);
    let now = env.ledger().timestamp();

    for i in 0..entities.len() {
        let entity_id = entities.get(i).unwrap();

        if let Some(data) = world.get_component(entity_id, &cr_sym) {
            let deadline = read_u64(&data, 32);
            let revealed = read_bool(&data, 40);

            if !revealed && now > deadline {
                world.remove_component(entity_id, &cr_sym);
            }
        }
    }
}

/// Remove `VerifiedMarker` components older than `max_age` ledgers.
///
/// This prevents verified markers from accumulating indefinitely.
pub fn cleanup_verified_system(world: &mut SimpleWorld, env: &Env, max_age: u64) {
    let verified_sym = Symbol::new(env, VERIFIED_MARKER_TYPE);
    let entities = world.get_entities_with_component(&verified_sym, env);
    let now = env.ledger().timestamp();

    for i in 0..entities.len() {
        let entity_id = entities.get(i).unwrap();

        if let Some(data) = world.get_component(entity_id, &verified_sym) {
            let verified_at = read_u64(&data, 0);

            if now.saturating_sub(verified_at) > max_age {
                world.remove_component(entity_id, &verified_sym);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zk::error::ZKError;
    use soroban_sdk::Env;

    struct RejectingVerifier;

    impl ProofVerifier for RejectingVerifier {
        type VerificationKey = VerificationKey;
        type Proof = Groth16Proof;
        type PublicInput = Scalar;

        fn verify(
            &self,
            _env: &Env,
            _verification_key: &Self::VerificationKey,
            _proof: &Self::Proof,
            _public_inputs: &[Self::PublicInput],
        ) -> Result<bool, ZKError> {
            Ok(false)
        }
    }

    #[test]
    fn test_commit_reveal_deadline_keeps_non_expired() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let e1 = world.spawn_entity();

        let commitment = BytesN::from_array(&env, &[0xABu8; 32]);
        let cr_data = encode_commit_reveal(&env, &commitment, 1000, false);
        let cr_sym = Symbol::new(&env, COMMIT_REVEAL_TYPE);
        world.add_component(e1, cr_sym.clone(), cr_data);

        // now = 0, deadline = 1000, not expired
        commit_reveal_deadline_system(&mut world, &env);
        assert!(world.has_component(e1, &cr_sym));
    }

    #[test]
    fn test_commit_reveal_keeps_revealed() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let e1 = world.spawn_entity();

        let commitment = BytesN::from_array(&env, &[0xABu8; 32]);
        let cr_data = encode_commit_reveal(&env, &commitment, 0, true);
        let cr_sym = Symbol::new(&env, COMMIT_REVEAL_TYPE);
        world.add_component(e1, cr_sym.clone(), cr_data);

        commit_reveal_deadline_system(&mut world, &env);
        // Revealed commitments are not removed even if past deadline
        assert!(world.has_component(e1, &cr_sym));
    }

    #[test]
    fn test_cleanup_verified_no_markers() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        // Should not panic with empty world
        cleanup_verified_system(&mut world, &env, 100);
    }

    #[test]
    fn test_cleanup_verified_keeps_recent() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let e1 = world.spawn_entity();

        let marker_data = encode_verified_marker(&env, 0);
        let verified_sym = Symbol::new(&env, VERIFIED_MARKER_TYPE);
        world.add_component(e1, verified_sym.clone(), marker_data);

        // max_age is 1000, marker is at time 0, now is 0, age = 0 <= 1000
        cleanup_verified_system(&mut world, &env, 1000);
        assert!(world.has_component(e1, &verified_sym));
    }

    #[test]
    fn test_verify_proofs_with_invalid_result_does_not_mark_entity() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let entity_id = world.spawn_entity();
        let verifier = RejectingVerifier;
        let vk = VerificationKey {
            alpha: super::super::types::G1Point {
                bytes: BytesN::from_array(&env, &[0u8; 64]),
            },
            beta: super::super::types::G2Point {
                bytes: BytesN::from_array(&env, &[0u8; 128]),
            },
            gamma: super::super::types::G2Point {
                bytes: BytesN::from_array(&env, &[0u8; 128]),
            },
            delta: super::super::types::G2Point {
                bytes: BytesN::from_array(&env, &[0u8; 128]),
            },
            ic: soroban_sdk::Vec::new(&env),
        };
        let proof = Groth16Proof {
            a: super::super::types::G1Point {
                bytes: BytesN::from_array(&env, &[0u8; 64]),
            },
            b: super::super::types::G2Point {
                bytes: BytesN::from_array(&env, &[0u8; 128]),
            },
            c: super::super::types::G1Point {
                bytes: BytesN::from_array(&env, &[0u8; 64]),
            },
        };

        let result =
            verify_proofs_with(&mut world, &env, entity_id, &verifier, &vk, &proof, &[]).unwrap();
        assert!(!result);
        assert!(!world.has_component(entity_id, &Symbol::new(&env, VERIFIED_MARKER_TYPE)));
    }
}
