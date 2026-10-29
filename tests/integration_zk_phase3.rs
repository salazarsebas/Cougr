//! Phase-3 ZK integration tests.
//!
//! These cover orchestration contracts for the advanced experimental surface:
//! fog-of-war transitions, multiplayer state channels, and recursive layouts.

use cougr_core::zk::experimental::{
    apply_fog_of_war_transition, apply_state_channel_transition, close_state_channel,
    compose_statement_roots, open_state_channel, FogOfWarSnapshot, FogOfWarTransition,
    RecursiveProofLayout, StateChannelTransition,
};
use cougr_core::zk::ZKError;
use soroban_sdk::{BytesN, Env};

#[test]
fn fog_of_war_transition_updates_explored_root() {
    let env = Env::default();
    let snapshot = FogOfWarSnapshot {
        map_root: BytesN::from_array(&env, &[1u8; 32]),
        explored_root: BytesN::from_array(&env, &[2u8; 32]),
        origin_x: 5,
        origin_y: 5,
        visibility_radius: 2,
    };
    let transition = FogOfWarTransition {
        prior_explored_root: snapshot.explored_root.clone(),
        next_explored_root: BytesN::from_array(&env, &[3u8; 32]),
        tile_x: 6,
        tile_y: 6,
    };

    let updated = apply_fog_of_war_transition(&snapshot, &transition).unwrap();
    assert_eq!(updated.explored_root, transition.next_explored_root);
}

#[test]
fn state_channel_transition_and_close_flow() {
    let env = Env::default();
    let channel = open_state_channel(
        BytesN::from_array(&env, &[1u8; 32]),
        BytesN::from_array(&env, &[2u8; 32]),
        BytesN::from_array(&env, &[3u8; 32]),
        100,
    )
    .unwrap();
    let transition = StateChannelTransition {
        prior_state_root: channel.state_root.clone(),
        next_state_root: BytesN::from_array(&env, &[4u8; 32]),
        round: 1,
        submitted_at: 42,
    };

    let updated = apply_state_channel_transition(&channel, &transition).unwrap();
    assert_eq!(updated.state_root, transition.next_state_root);
    assert_eq!(updated.round, 1);

    let closed = close_state_channel(&updated, &updated.state_root, updated.round, 50).unwrap();
    assert!(closed.closed);
    assert_eq!(closed.dispute_deadline, 50);
}

#[test]
fn state_channel_transition_rejects_late_submission() {
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
        round: 1,
        submitted_at: 11,
    };

    let result = apply_state_channel_transition(&channel, &transition);
    assert!(matches!(result, Err(ZKError::DeadlineExpired)));
}

#[test]
fn recursive_layout_matches_composed_statement_root() {
    let env = Env::default();
    let step_roots = [
        BytesN::from_array(&env, &[7u8; 32]),
        BytesN::from_array(&env, &[8u8; 32]),
    ];

    let accumulator = compose_statement_roots(&env, &step_roots).unwrap();
    let layout = RecursiveProofLayout::from_step_roots(
        &env,
        BytesN::from_array(&env, &[1u8; 32]),
        BytesN::from_array(&env, &[2u8; 32]),
        &step_roots,
    )
    .unwrap();

    assert_eq!(layout.accumulator_root, accumulator);
    assert_eq!(layout.proof_count, 2);
}
