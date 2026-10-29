//! Integration tests for `cougr_core::circuits` with real pipeline VKs and proofs.

use cougr_core::circuits::{
    fair_dice, fog_of_war, hidden_cards, sealed_bid, test_fixtures, CircuitId, CircuitParams,
};
use cougr_core::zk::experimental::{FogOfWarSnapshot, FogOfWarTransition};
use cougr_core::zk::ZKError;
use soroban_sdk::{BytesN, Env};

#[test]
fn hidden_cards_builder_uses_pipeline_vk() {
    let env = Env::default();
    let spec = hidden_cards(&env, 52, 5).unwrap();
    assert_eq!(spec.circuit_id, CircuitId::HiddenCards);
    assert_eq!(spec.layout.public_input_count(), 5);
    assert_eq!(spec.verification_key.ic.len(), 6);
}

#[test]
fn fair_dice_pipeline_proof_verifies_on_chain() {
    let env = Env::default();
    let seed = test_fixtures::pipeline_public_bytes32(&env, CircuitId::FairDice);
    let spec = fair_dice(&env, 6, &seed).unwrap();
    let proof = test_fixtures::pipeline_proof(&env, CircuitId::FairDice);

    let result = spec.verify_dice_roll(&env, &proof, 6, 5);
    assert_eq!(result, Ok(true));
}

#[test]
fn hidden_cards_pipeline_proof_verifies_on_chain() {
    let env = Env::default();
    let spec = hidden_cards(&env, 52, 5).unwrap();
    let proof = test_fixtures::pipeline_proof(&env, CircuitId::HiddenCards);
    let public = test_fixtures::pipeline_public_inputs(&env, CircuitId::HiddenCards);

    let deck = BytesN::from_array(&env, &public[0].bytes.to_array());
    let hand = BytesN::from_array(&env, &public[1].bytes.to_array());
    let player_id = 2;

    let result = spec.verify_hidden_hand(&env, &proof, &deck, &hand, player_id);
    assert_eq!(result, Ok(true));
}

#[test]
fn fog_of_war_pipeline_proof_verifies_on_chain() {
    let env = Env::default();
    let spec = fog_of_war(&env, 32, 32, 3).unwrap();
    let proof = test_fixtures::pipeline_proof(&env, CircuitId::FogOfWar);
    let public = test_fixtures::pipeline_public_inputs(&env, CircuitId::FogOfWar);

    let snapshot = FogOfWarSnapshot {
        map_root: BytesN::from_array(&env, &public[0].bytes.to_array()),
        explored_root: BytesN::from_array(&env, &public[1].bytes.to_array()),
        origin_x: 0,
        origin_y: 0,
        visibility_radius: 3,
    };
    let transition = FogOfWarTransition {
        prior_explored_root: BytesN::from_array(&env, &public[1].bytes.to_array()),
        next_explored_root: BytesN::from_array(&env, &public[2].bytes.to_array()),
        tile_x: 1,
        tile_y: 2,
    };

    let result = spec.verify_fog_exploration(&env, &proof, &snapshot, &transition);
    assert_eq!(result, Ok(true));
}

#[test]
fn sealed_bid_pipeline_proof_verifies_on_chain() {
    let env = Env::default();
    let spec = sealed_bid(&env, 1000).unwrap();
    let proof = test_fixtures::pipeline_proof(&env, CircuitId::SealedBid);
    let public = test_fixtures::pipeline_public_inputs(&env, CircuitId::SealedBid);

    let auction = BytesN::from_array(&env, &public[0].bytes.to_array());
    let commit = BytesN::from_array(&env, &public[1].bytes.to_array());
    let revealed = 50u32;

    let result = spec.verify_bid_reveal(&env, &proof, &auction, &commit, revealed);
    assert_eq!(result, Ok(true));
}

#[test]
fn fair_dice_rejects_out_of_range_roll_before_verification() {
    let env = Env::default();
    let seed = test_fixtures::pipeline_public_bytes32(&env, CircuitId::FairDice);
    let spec = fair_dice(&env, 6, &seed).unwrap();
    let proof = test_fixtures::pipeline_proof(&env, CircuitId::FairDice);

    assert_eq!(
        spec.verify_dice_roll(&env, &proof, 7, 5),
        Err(ZKError::InvalidInput)
    );
}

#[test]
fn fair_dice_builder_binds_seed_commitment() {
    let env = Env::default();
    let seed = test_fixtures::pipeline_public_bytes32(&env, CircuitId::FairDice);
    let spec = fair_dice(&env, 6, &seed).unwrap();
    assert_eq!(spec.params, CircuitParams::FairDice(6, seed));
}
