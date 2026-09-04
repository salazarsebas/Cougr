//! hidden_hand - canonical Cougr ZK circuits example.
//!
//! Demonstrates `cougr_core::circuits::hidden_cards` for private card deals.

#![no_std]

use cougr_core::circuits::hidden_cards;
use cougr_core::zk::Groth16Proof;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol,
};

#[contracttype]
#[derive(Clone, Debug)]
pub struct TableConfig {
    pub deck_size: u32,
    pub hand_size: u32,
}

#[contract]
#[derive(Clone)]
pub struct HiddenHand;

#[contractimpl]
impl HiddenHand {
    /// Configure the table with a standard hidden-card circuit spec.
    pub fn init_table(env: Env, deck_size: u32, hand_size: u32) -> TableConfig {
        let _spec = hidden_cards(&env, deck_size, hand_size).expect("valid table config");
        let config = TableConfig {
            deck_size,
            hand_size,
        };
        env.storage().instance().set(&table_key(&env), &config);
        config
    }

    /// Verify a deal proof against the frozen hidden-cards layout.
    pub fn verify_deal(
        env: Env,
        player: Address,
        deck_root: BytesN<32>,
        hand_commitment: BytesN<32>,
        proof: Groth16Proof,
    ) -> bool {
        let config: TableConfig = env
            .storage()
            .instance()
            .get(&table_key(&env))
            .expect("table not initialized");
        let spec = hidden_cards(&env, config.deck_size, config.hand_size).expect("circuit spec");
        spec.verify_hidden_hand(
            &env,
            &proof,
            &deck_root,
            &hand_commitment,
            player_id(&player),
        )
        .unwrap_or(false)
    }
}

fn table_key(_env: &Env) -> Symbol {
    symbol_short!("table")
}

fn player_id(player: &Address) -> u32 {
    let bytes = player.to_string().to_bytes();
    let mut id = 0u32;
    for i in 0..bytes.len().min(4) {
        id = id.wrapping_add(u32::from(bytes.get(i).unwrap_or(0)));
    }
    id
}

#[cfg(test)]
mod tests;
