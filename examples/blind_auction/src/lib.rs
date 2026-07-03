//! blind_auction — canonical Cougr ZK circuits example for sealed-bid reveals.

#![no_std]

use cougr_core::circuits::sealed_bid;
use cougr_core::zk::Groth16Proof;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol,
};

#[contracttype]
#[derive(Clone, Debug)]
pub struct AuctionConfig {
    pub max_bid: u32,
    pub auction_id: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct BidReveal {
    pub bidder: Address,
    pub revealed_bid: u32,
}

#[contract]
#[derive(Clone)]
pub struct BlindAuction;

#[contractimpl]
impl BlindAuction {
    pub fn init_auction(env: Env, max_bid: u32, auction_id: BytesN<32>) -> AuctionConfig {
        let _spec = sealed_bid(&env, max_bid).expect("valid auction");
        let config = AuctionConfig {
            max_bid,
            auction_id,
        };
        env.storage().instance().set(&auction_key(&env), &config);
        config
    }

    pub fn reveal_bid(
        env: Env,
        bidder: Address,
        bid_commitment: BytesN<32>,
        revealed_bid: u32,
        proof: Groth16Proof,
    ) -> bool {
        let config: AuctionConfig = env
            .storage()
            .instance()
            .get(&auction_key(&env))
            .expect("auction not initialized");
        let spec = sealed_bid(&env, config.max_bid).expect("circuit spec");
        let ok = spec
            .verify_bid_reveal(
                &env,
                &proof,
                &config.auction_id,
                &bid_commitment,
                revealed_bid,
            )
            .unwrap_or(false);
        if ok {
            let reveal = BidReveal {
                bidder: bidder.clone(),
                revealed_bid,
            };
            env.storage()
                .instance()
                .set(&bid_key(&env, &bidder), &reveal);
        }
        ok
    }

    pub fn bid_reveal(env: Env, bidder: Address) -> BidReveal {
        env.storage()
            .instance()
            .get(&bid_key(&env, &bidder))
            .expect("no reveal")
    }
}

fn auction_key(_env: &Env) -> Symbol {
    symbol_short!("auction")
}

fn bid_key(_env: &Env, bidder: &Address) -> (Symbol, Address) {
    (symbol_short!("bid"), bidder.clone())
}

#[cfg(test)]
mod tests;
