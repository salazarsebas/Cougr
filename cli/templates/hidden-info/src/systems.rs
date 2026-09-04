//! Circuit wiring and table rules for {{crate_name}}.
//!
//! `cougr_core::circuits::hidden_cards` builds a `GameCircuitSpec` - the public
//! input layout and verification key for a hidden-card deal. Rebuilding it from
//! the stored [`TableConfig`] on every call, rather than caching a spec, keeps
//! one source of truth for what a valid proof must satisfy.

use cougr_core::circuits::{hidden_cards, GameCircuitSpec};
use cougr_core::zk::Groth16Proof;
use soroban_sdk::{BytesN, Env};

use crate::components::TableConfig;

/// Largest table this contract accepts. The circuit itself covers a wider
/// range; capping it here keeps proof verification inside a predictable
/// resource budget.
pub const MAX_DECK_SIZE: u32 = 64;

/// Why a table configuration is not usable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableError {
    /// `deck_size` is zero or above [`MAX_DECK_SIZE`].
    DeckSizeOutOfRange,
    /// `hand_size` is zero, or larger than the deck it is drawn from.
    HandSizeOutOfRange,
    /// The circuit builder rejected these parameters.
    UnsupportedLayout,
}

/// Validate table parameters before they are frozen into storage.
pub fn validate_config(config: &TableConfig) -> Result<(), TableError> {
    if config.deck_size == 0 || config.deck_size > MAX_DECK_SIZE {
        return Err(TableError::DeckSizeOutOfRange);
    }
    if config.hand_size == 0 || config.hand_size > config.deck_size {
        return Err(TableError::HandSizeOutOfRange);
    }
    Ok(())
}

/// Build the hidden-cards circuit spec for `config`.
pub fn circuit_for(env: &Env, config: &TableConfig) -> Result<GameCircuitSpec, TableError> {
    validate_config(config)?;
    hidden_cards(env, config.deck_size, config.hand_size).map_err(|_| TableError::UnsupportedLayout)
}

/// Check that `proof` shows `hand_commitment` was dealt to `seat` from the deck
/// committed to by `deck_root`.
///
/// A proof that does not verify is a `false`, not an error: an opponent
/// submitting a bad proof is ordinary gameplay, not a contract fault.
pub fn verify_deal(
    env: &Env,
    config: &TableConfig,
    seat: u32,
    deck_root: &BytesN<32>,
    hand_commitment: &BytesN<32>,
    proof: &Groth16Proof,
) -> bool {
    let Ok(spec) = circuit_for(env, config) else {
        return false;
    };
    spec.verify_hidden_hand(env, proof, deck_root, hand_commitment, seat)
        .unwrap_or(false)
}
