//! Table state and storage keys for {{crate_name}}.
//!
//! A hidden-information game keeps the interesting state *off* chain: the deck
//! order and every player's hand stay with their owners. What the contract
//! stores is the public shape of the table, who is sitting at it, and how many
//! deals each seat has proven - everything a proof is checked against, and
//! nothing that would leak a hand.

use cougr_core::impl_component;
use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

/// Public parameters of the table, frozen at `init_table` so every later proof
/// is verified against the same circuit layout.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct TableConfig {
    pub deck_size: u32,
    pub hand_size: u32,
}

/// How many deals a seat has proven. One component per seated player.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct DealsVerified {
    pub count: u32,
}

impl_component!(DealsVerified, "deals", Sparse, { count: u32 });

/// Instance-storage key holding the [`TableConfig`].
pub fn table_key() -> Symbol {
    symbol_short!("table")
}

/// Instance-storage key mapping a player to their seat number.
pub fn seat_key(env: &Env, player: &Address) -> (Symbol, Address) {
    (Symbol::new(env, "seat"), player.clone())
}

/// ECS entity that holds a seat's components.
///
/// Seats are zero-based for the circuit; `SimpleWorld` entity IDs start at one.
pub fn seat_entity(seat: u32) -> u32 {
    seat + 1
}
