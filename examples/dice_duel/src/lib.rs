//! dice_duel — canonical Cougr ZK circuits example for verifiable dice rolls.

#![no_std]

use cougr_core::circuits::fair_dice;
use cougr_core::zk::Groth16Proof;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol,
};

#[contracttype]
#[derive(Clone, Debug)]
pub struct DuelConfig {
    pub sides: u32,
    pub seed_commitment: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct RollRecord {
    pub player: Address,
    pub roll: u32,
    pub nonce: u32,
}

#[contract]
#[derive(Clone)]
pub struct DiceDuel;

#[contractimpl]
impl DiceDuel {
    pub fn init_duel(env: Env, sides: u32, seed_commitment: BytesN<32>) -> DuelConfig {
        let _spec = fair_dice(&env, sides, &seed_commitment).expect("valid duel");
        let config = DuelConfig {
            sides,
            seed_commitment,
        };
        env.storage().instance().set(&duel_key(&env), &config);
        config
    }

    pub fn submit_roll(
        env: Env,
        player: Address,
        roll_result: u32,
        nonce: u32,
        proof: Groth16Proof,
    ) -> bool {
        let config: DuelConfig = env
            .storage()
            .instance()
            .get(&duel_key(&env))
            .expect("duel not initialized");
        let spec = fair_dice(&env, config.sides, &config.seed_commitment).expect("circuit spec");
        let ok = spec
            .verify_dice_roll(&env, &proof, roll_result, nonce)
            .unwrap_or(false);
        if ok {
            let record = RollRecord {
                player: player.clone(),
                roll: roll_result,
                nonce,
            };
            env.storage()
                .instance()
                .set(&roll_key(&env, &player), &record);
        }
        ok
    }

    pub fn roll_record(env: Env, player: Address) -> RollRecord {
        env.storage()
            .instance()
            .get(&roll_key(&env, &player))
            .expect("no roll")
    }
}

fn duel_key(_env: &Env) -> Symbol {
    symbol_short!("duel")
}

fn roll_key(_env: &Env, player: &Address) -> (Symbol, Address) {
    (symbol_short!("roll"), player.clone())
}

#[cfg(test)]
mod tests;
