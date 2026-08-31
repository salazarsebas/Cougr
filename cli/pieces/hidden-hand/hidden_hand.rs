//! Hidden-information helpers copied from the `hidden_hand` example.

use cougr_core::circuits::{hidden_cards, GameCircuitSpec};
use cougr_core::zk::Groth16Proof;
use soroban_sdk::{BytesN, Env};

/// Largest table accepted by this helper, keeping proof costs predictable.
pub const MAX_DECK_SIZE: u32 = 64;

/// Public table shape used to select a frozen circuit layout.
#[soroban_sdk::contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct TableConfig {
    pub deck_size: u32,
    pub hand_size: u32,
}

/// Validate a table shape before storing it in a contract.
pub fn validate_config(config: &TableConfig) -> bool {
    config.deck_size > 0
        && config.deck_size <= MAX_DECK_SIZE
        && config.hand_size > 0
        && config.hand_size <= config.deck_size
}

/// Build the circuit selected by a table shape.
pub fn circuit_for(env: &Env, config: &TableConfig) -> Option<GameCircuitSpec> {
    if !validate_config(config) {
        return None;
    }
    hidden_cards(env, config.deck_size, config.hand_size).ok()
}

/// Verify a deal without exposing the player's hand to the contract.
pub fn verify_deal(
    env: &Env,
    config: &TableConfig,
    seat: u32,
    deck_root: &BytesN<32>,
    hand_commitment: &BytesN<32>,
    proof: &Groth16Proof,
) -> bool {
    circuit_for(env, config)
        .map(|spec| spec.verify_hidden_hand(env, proof, deck_root, hand_commitment, seat).unwrap_or(false))
        .unwrap_or(false)
}
