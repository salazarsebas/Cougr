//! {{crate_name}} - a Cougr game contract generated from the `{{template_id}}` template.
//!
//! Players take a seat at a table, then prove - without revealing a single card
//! - that the hand they hold was really dealt from the committed deck. The
//! contract never learns the hand: it checks a Groth16 proof against the public
//! commitments and counts the deals that verified.
//!
//! Demonstrates:
//!   - `circuits::hidden_cards` - pre-built hidden-card circuit spec
//!   - `zk::Groth16Proof` - proof submitted as a contract argument
//!   - `SorobanGame` - standard world load/save
//!   - `impl_soroban_game!` - wires the trait to a `#[contract]` struct

#![no_std]

pub mod components;
pub mod systems;
#[cfg(test)]
mod test;

use components::{seat_entity, seat_key, table_key, DealsVerified, TableConfig};
use systems::{validate_config, verify_deal, TableError};

use cougr_core::game::SorobanGame;
use cougr_core::impl_soroban_game;
use cougr_core::zk::Groth16Proof;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

#[contract]
#[derive(Clone)]
pub struct {{ContractName}};

// Generates `load_world` / `save_world` against the "world" instance-storage key.
impl_soroban_game!({{ContractName}}, "world");

#[contractimpl]
impl {{ContractName}} {
    /// Open a table with a fixed deck and hand size.
    ///
    /// The configuration is frozen here: every later proof is checked against
    /// the circuit layout these two numbers select. Panics if the table shape
    /// is outside what the circuit supports.
    pub fn init_table(env: Env, deck_size: u32, hand_size: u32) -> TableConfig {
        let config = TableConfig {
            deck_size,
            hand_size,
        };

        match validate_config(&config) {
            Ok(()) => {}
            Err(TableError::DeckSizeOutOfRange) => panic!("deck size out of range"),
            Err(TableError::HandSizeOutOfRange) => panic!("hand size out of range"),
            Err(TableError::UnsupportedLayout) => panic!("unsupported circuit layout"),
        }

        env.storage().instance().set(&table_key(), &config);
        config
    }

    /// Take a seat at the table, returning the zero-based seat number.
    ///
    /// The seat number is what the circuit binds a hand to, so a player proves
    /// against the seat they joined with. Joining twice returns the same seat.
    pub fn join_table(env: Env, player: Address) -> u32 {
        player.require_auth();
        Self::require_table(&env);

        if let Some(seat) = Self::seat_of(env.clone(), player.clone()) {
            return seat;
        }

        let mut world = {{ContractName}}::load_world(&env);
        let entity = world.spawn_entity();
        world.set_typed(&env, entity, &DealsVerified { count: 0 });
        {{ContractName}}::save_world(&env, &world);

        let seat = entity - 1;
        env.storage()
            .instance()
            .set(&seat_key(&env, &player), &seat);
        seat
    }

    /// The seat `player` joined with, or `None` if they never sat down.
    pub fn seat_of(env: Env, player: Address) -> Option<u32> {
        env.storage().instance().get(&seat_key(&env, &player))
    }

    /// The frozen table configuration, or `None` before `init_table`.
    pub fn table(env: Env) -> Option<TableConfig> {
        env.storage().instance().get(&table_key())
    }

    /// Verify that `player`'s hand was dealt from the committed deck.
    ///
    /// Returns `false` for a proof that does not check out - only a missing
    /// table or an unseated player is treated as a caller error.
    pub fn verify_deal(
        env: Env,
        player: Address,
        deck_root: BytesN<32>,
        hand_commitment: BytesN<32>,
        proof: Groth16Proof,
    ) -> bool {
        let config = Self::require_table(&env);
        let seat = Self::seat_of(env.clone(), player)
            .unwrap_or_else(|| panic!("player has not joined the table"));

        let verified = verify_deal(&env, &config, seat, &deck_root, &hand_commitment, &proof);
        if verified {
            Self::record_deal(&env, seat);
        }
        verified
    }

    /// How many deals `player` has proven so far.
    pub fn deals_verified(env: Env, player: Address) -> u32 {
        let Some(seat) = Self::seat_of(env.clone(), player) else {
            return 0;
        };
        let world = {{ContractName}}::load_world(&env);
        world
            .get_typed::<DealsVerified>(&env, seat_entity(seat))
            .map(|deals| deals.count)
            .unwrap_or(0)
    }

    // ─── Internal helpers ─────────────────────────────────────────────────────

    fn require_table(env: &Env) -> TableConfig {
        env.storage()
            .instance()
            .get(&table_key())
            .unwrap_or_else(|| panic!("table not initialised"))
    }

    fn record_deal(env: &Env, seat: u32) {
        let mut world = {{ContractName}}::load_world(env);
        let entity = seat_entity(seat);
        let count = world
            .get_typed::<DealsVerified>(env, entity)
            .map(|deals| deals.count)
            .unwrap_or(0);

        world.set_typed(
            env,
            entity,
            &DealsVerified {
                count: count.saturating_add(1),
            },
        );
        {{ContractName}}::save_world(env, &world);
    }
}
