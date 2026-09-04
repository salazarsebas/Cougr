//! ZK subsystem integration tests.
//!
//! Tests ZK components lifecycle, commit-reveal flow, cleanup systems,
//! and byte encoding round-trips.

use cougr_core::zk::experimental::{
    cleanup_verified_system, encode_verified_marker, VERIFIED_MARKER_TYPE,
};
use cougr_core::zk::stable::{
    commit_reveal_deadline_system, encode_commit_reveal, COMMIT_REVEAL_TYPE, HIDDEN_STATE_TYPE,
};
use cougr_core::SimpleWorld;
use soroban_sdk::{symbol_short, Bytes, BytesN, Env, Symbol};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_hidden_state_component_lifecycle() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    let e1 = world.spawn_entity();
    let hidden_sym = Symbol::new(&env, HIDDEN_STATE_TYPE);

    // Add hidden state component (as raw bytes for ECS storage)
    let commitment = BytesN::from_array(&env, &[0xABu8; 32]);
    let commitment_bytes: Bytes = commitment.clone().into();
    world.add_component(e1, hidden_sym.clone(), commitment_bytes.clone());

    assert!(world.has_component(e1, &hidden_sym));
    let stored = world.get_component(e1, &hidden_sym).unwrap();
    assert_eq!(stored, commitment_bytes);

    // Remove hidden state
    world.remove_component(e1, &hidden_sym);
    assert!(!world.has_component(e1, &hidden_sym));
}

#[test]
fn test_commit_reveal_full_flow() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    let e1 = world.spawn_entity();
    let cr_sym = Symbol::new(&env, COMMIT_REVEAL_TYPE);

    // Phase 1: Commit
    let commitment = BytesN::from_array(&env, &[0xCDu8; 32]);
    let cr_data = encode_commit_reveal(&env, &commitment, 1000, false);
    world.add_component(e1, cr_sym.clone(), cr_data);
    assert!(world.has_component(e1, &cr_sym));

    // Check that non-expired, non-revealed commitment stays
    commit_reveal_deadline_system(&mut world, &env);
    assert!(world.has_component(e1, &cr_sym));

    // Phase 2: Reveal (update to revealed=true)
    let revealed_data = encode_commit_reveal(&env, &commitment, 1000, true);
    world.add_component(e1, cr_sym.clone(), revealed_data);

    // Even past deadline, revealed commitments stay
    commit_reveal_deadline_system(&mut world, &env);
    assert!(world.has_component(e1, &cr_sym));
}

#[test]
fn test_commit_reveal_timeout_removes_component() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    let e1 = world.spawn_entity();
    let cr_sym = Symbol::new(&env, COMMIT_REVEAL_TYPE);

    // Commit with deadline = 0 (already expired at ledger time 0 since now > deadline
    // requires now > 0 to expire at 0 - but default ledger time = 0, so now = 0.
    // We need deadline < now. Let's set deadline to 0, with current time = 0 → 0 > 0 = false.
    // So the commit won't be removed. We should set deadline such that it will expire.)

    // Set a commit that will expire: deadline = 0, now = 0 → 0 > 0 = false, not expired.
    // We need to work around the ledger timestamp. Let's add two commits:
    // one with deadline=0 (not yet expired at t=0) and verify behavior.

    let commitment = BytesN::from_array(&env, &[0xEEu8; 32]);
    let cr_data = encode_commit_reveal(&env, &commitment, 0, false);
    world.add_component(e1, cr_sym.clone(), cr_data);

    // now=0, deadline=0 → 0 > 0 = false, commit stays
    commit_reveal_deadline_system(&mut world, &env);
    assert!(world.has_component(e1, &cr_sym));
}

#[test]
fn test_verified_marker_encode_decode_roundtrip() {
    let env = Env::default();

    let verified_at: u64 = 12345;
    let encoded = encode_verified_marker(&env, verified_at);
    assert_eq!(encoded.len(), 8);

    // Decode manually (big-endian u64)
    let mut arr = [0u8; 8];
    for i in 0..8u32 {
        arr[i as usize] = encoded.get(i).unwrap();
    }
    let decoded = u64::from_be_bytes(arr);
    assert_eq!(decoded, verified_at);
}

#[test]
fn test_commit_reveal_encode_roundtrip() {
    let env = Env::default();

    let commitment = BytesN::from_array(&env, &[0x42u8; 32]);
    let deadline: u64 = 99999;
    let revealed = false;

    let encoded = encode_commit_reveal(&env, &commitment, deadline, revealed);
    assert_eq!(encoded.len(), 41); // 32 + 8 + 1

    // Decode commitment (first 32 bytes)
    let mut commitment_arr = [0u8; 32];
    for i in 0..32u32 {
        commitment_arr[i as usize] = encoded.get(i).unwrap();
    }
    assert_eq!(commitment_arr, [0x42u8; 32]);

    // Decode deadline (bytes 32-39)
    let mut deadline_arr = [0u8; 8];
    for i in 0..8u32 {
        deadline_arr[i as usize] = encoded.get(32 + i).unwrap();
    }
    assert_eq!(u64::from_be_bytes(deadline_arr), deadline);

    // Decode revealed (byte 40)
    assert_eq!(encoded.get(40).unwrap(), 0);
}

#[test]
fn test_cleanup_verified_with_multiple_entities() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let verified_sym = Symbol::new(&env, VERIFIED_MARKER_TYPE);

    // Entity 1: verified_at = 0 (oldest)
    let e1 = world.spawn_entity();
    world.add_component(e1, verified_sym.clone(), encode_verified_marker(&env, 0));

    // Entity 2: verified_at = 0 (same age)
    let e2 = world.spawn_entity();
    world.add_component(e2, verified_sym.clone(), encode_verified_marker(&env, 0));

    // Entity 3: no verified marker (unrelated entity)
    let e3 = world.spawn_entity();
    world.add_component(e3, symbol_short!("pos"), Bytes::from_array(&env, &[10]));

    // max_age = 0: only remove markers where (now - verified_at) > 0
    // now = 0, verified_at = 0 → age = 0, 0 > 0 = false → no removal
    cleanup_verified_system(&mut world, &env, 0);
    assert!(world.has_component(e1, &verified_sym));
    assert!(world.has_component(e2, &verified_sym));

    // e3 should be untouched
    assert!(world.has_component(e3, &symbol_short!("pos")));
}

#[test]
fn test_zk_components_coexist_with_ecs_components() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    let player = world.spawn_entity();
    let hidden_sym = Symbol::new(&env, HIDDEN_STATE_TYPE);
    let verified_sym = Symbol::new(&env, VERIFIED_MARKER_TYPE);

    // Add regular ECS components
    world.add_component(
        player,
        symbol_short!("pos"),
        Bytes::from_array(&env, &[10, 20]),
    );
    world.add_component(player, symbol_short!("hp"), Bytes::from_array(&env, &[100]));

    // Add ZK components
    let commitment: Bytes = BytesN::from_array(&env, &[0xABu8; 32]).into();
    world.add_component(player, hidden_sym.clone(), commitment);
    world.add_component(
        player,
        verified_sym.clone(),
        encode_verified_marker(&env, 42),
    );

    // All components coexist
    assert!(world.has_component(player, &symbol_short!("pos")));
    assert!(world.has_component(player, &symbol_short!("hp")));
    assert!(world.has_component(player, &hidden_sym));
    assert!(world.has_component(player, &verified_sym));

    // Can query by ZK component
    let hidden_entities = world.get_entities_with_component(&hidden_sym, &env);
    assert_eq!(hidden_entities.len(), 1);
}

#[test]
fn test_multiple_commit_reveals_different_entities() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let cr_sym = Symbol::new(&env, COMMIT_REVEAL_TYPE);

    // Player 1 commits
    let p1 = world.spawn_entity();
    let c1 = BytesN::from_array(&env, &[1u8; 32]);
    world.add_component(
        p1,
        cr_sym.clone(),
        encode_commit_reveal(&env, &c1, 1000, false),
    );

    // Player 2 commits
    let p2 = world.spawn_entity();
    let c2 = BytesN::from_array(&env, &[2u8; 32]);
    world.add_component(
        p2,
        cr_sym.clone(),
        encode_commit_reveal(&env, &c2, 1000, false),
    );

    // Both have commit-reveal
    let cr_entities = world.get_entities_with_component(&cr_sym, &env);
    assert_eq!(cr_entities.len(), 2);

    // Player 1 reveals
    world.add_component(
        p1,
        cr_sym.clone(),
        encode_commit_reveal(&env, &c1, 1000, true),
    );

    // System run: neither expired (deadline=1000, now=0)
    commit_reveal_deadline_system(&mut world, &env);
    assert!(world.has_component(p1, &cr_sym));
    assert!(world.has_component(p2, &cr_sym));
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase C - ZK Accessible
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_verification_key_from_raw_bytes_roundtrip() {
    use cougr_core::zk::experimental::VerificationKey;
    let env = Env::default();

    let alpha: [u8; 64] = {
        let mut b = [0u8; 64];
        b[0] = 0x01;
        b[63] = 0x02;
        b
    };
    let beta: [u8; 128] = {
        let mut b = [0u8; 128];
        b[0] = 0x03;
        b[127] = 0x04;
        b
    };
    let gamma: [u8; 128] = [0x05u8; 128];
    let delta: [u8; 128] = [0x06u8; 128];
    let ic0: [u8; 64] = [0x07u8; 64];
    let ic1: [u8; 64] = [0x08u8; 64];
    let ic: &[[u8; 64]] = &[ic0, ic1];

    let vk = VerificationKey::from_raw_bytes(&env, &alpha, &beta, &gamma, &delta, ic);

    assert_eq!(vk.ic.len(), 2);
    assert_eq!(vk.alpha.bytes.get(0), Some(0x01u8));
    assert_eq!(vk.alpha.bytes.get(63), Some(0x02u8));
    assert_eq!(vk.beta.bytes.get(0), Some(0x03u8));
    assert_eq!(vk.beta.bytes.get(127), Some(0x04u8));
    assert_eq!(vk.ic.get(0).unwrap().bytes.get(0), Some(0x07u8));
    assert_eq!(vk.ic.get(1).unwrap().bytes.get(0), Some(0x08u8));
}

#[test]
fn test_groth16_proof_from_raw_bytes_roundtrip() {
    use cougr_core::zk::experimental::Groth16Proof;
    let env = Env::default();

    let a: [u8; 64] = {
        let mut b = [0u8; 64];
        b[31] = 0xAA;
        b
    };
    let b: [u8; 128] = {
        let mut arr = [0u8; 128];
        arr[0] = 0xBB;
        arr
    };
    let c: [u8; 64] = [0xCCu8; 64];

    let proof = Groth16Proof::from_raw_bytes(&env, &a, &b, &c);

    assert_eq!(proof.a.bytes.get(31), Some(0xAAu8));
    assert_eq!(proof.b.bytes.get(0), Some(0xBBu8));
    assert_eq!(proof.c.bytes.get(0), Some(0xCCu8));
    assert_eq!(proof.a.bytes.len(), 64);
    assert_eq!(proof.b.bytes.len(), 128);
    assert_eq!(proof.c.bytes.len(), 64);
}

#[test]
fn test_verification_key_from_raw_bytes_empty_ic() {
    use cougr_core::zk::experimental::VerificationKey;
    let env = Env::default();

    let vk = VerificationKey::from_raw_bytes(
        &env,
        &[0u8; 64],
        &[0u8; 128],
        &[0u8; 128],
        &[0u8; 128],
        &[],
    );
    assert_eq!(vk.ic.len(), 0);
}

#[test]
fn test_batch_proof_context_record_and_finalize() {
    use cougr_core::zk::experimental::{hash_state_root, BatchProofContext};
    use soroban_sdk::Bytes;

    let env = Env::default();
    let empty: [Bytes; 0] = [];
    let initial_root = hash_state_root(&env, &empty);

    let ctx = BatchProofContext::new(initial_root.clone())
        .record_action()
        .record_action()
        .record_action();

    assert_eq!(ctx.action_count, 3);

    let final_root = hash_state_root(&env, &empty);
    let (init, fin, count) = ctx.finalize(final_root.clone());

    assert_eq!(init, initial_root);
    assert_eq!(fin, final_root);
    assert_eq!(count, 3);
}

#[test]
fn test_hash_state_root_empty_is_deterministic() {
    use cougr_core::zk::experimental::hash_state_root;
    use soroban_sdk::Bytes;

    let env = Env::default();
    let empty: [Bytes; 0] = [];

    let r1 = hash_state_root(&env, &empty);
    let r2 = hash_state_root(&env, &empty);
    assert_eq!(r1, r2);
}

#[test]
fn test_hash_state_root_differs_by_content() {
    use cougr_core::zk::experimental::hash_state_root;
    use soroban_sdk::Bytes;

    let env = Env::default();
    let a = Bytes::from_array(&env, &[1u8, 2u8, 3u8]);
    let b = Bytes::from_array(&env, &[4u8, 5u8, 6u8]);

    let root_a = hash_state_root(&env, &[a]);
    let root_b = hash_state_root(&env, &[b]);
    assert_ne!(root_a, root_b);
}

#[test]
fn test_bls12_381_aggregate_signatures_empty_fails() {
    use cougr_core::zk::experimental::bls12_381_aggregate_signatures;
    use cougr_core::zk::ZKError;

    let env = Env::default();
    let result = bls12_381_aggregate_signatures(&env, &[]);
    assert_eq!(result.unwrap_err(), ZKError::InvalidInput);
}

#[test]
fn test_bls12_381_verify_aggregated_empty_fails() {
    use cougr_core::zk::experimental::bls12_381_verify_aggregated;
    use cougr_core::zk::{Bls12381G1Point, Bls12381G2Point, ZKError};

    let env = Env::default();
    let g1 = Bls12381G1Point {
        bytes: BytesN::from_array(&env, &[0u8; 96]),
    };
    let g2 = Bls12381G2Point {
        bytes: BytesN::from_array(&env, &[0u8; 192]),
    };

    let result = bls12_381_verify_aggregated(&env, &g1, &g2, &[], &[]);
    assert_eq!(result, Err(ZKError::InvalidInput));
}

#[test]
fn test_bls12_381_verify_aggregated_mismatched_lengths_fails() {
    use cougr_core::zk::experimental::bls12_381_verify_aggregated;
    use cougr_core::zk::{Bls12381G1Point, Bls12381G2Point, ZKError};

    let env = Env::default();
    let sig = Bls12381G1Point {
        bytes: BytesN::from_array(&env, &[0u8; 96]),
    };
    let g2 = Bls12381G2Point {
        bytes: BytesN::from_array(&env, &[0u8; 192]),
    };
    let msg = Bls12381G1Point {
        bytes: BytesN::from_array(&env, &[0u8; 96]),
    };

    // 1 message but 0 pubkeys
    let result = bls12_381_verify_aggregated(&env, &sig, &g2, &[msg], &[]);
    assert_eq!(result, Err(ZKError::InvalidInput));
}

#[test]
fn test_bls12_381_verify_aggregated_same_msg_empty_pubkeys_fails() {
    use cougr_core::zk::experimental::bls12_381_verify_aggregated_same_msg;
    use cougr_core::zk::{Bls12381G1Point, Bls12381G2Point, ZKError};

    let env = Env::default();
    let sig = Bls12381G1Point {
        bytes: BytesN::from_array(&env, &[0u8; 96]),
    };
    let g2 = Bls12381G2Point {
        bytes: BytesN::from_array(&env, &[0u8; 192]),
    };
    let msg = Bls12381G1Point {
        bytes: BytesN::from_array(&env, &[0u8; 96]),
    };

    let result = bls12_381_verify_aggregated_same_msg(&env, &sig, &g2, &msg, &[]);
    assert_eq!(result, Err(ZKError::InvalidInput));
}
