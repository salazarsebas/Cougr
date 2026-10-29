#![no_std]
#![allow(deprecated)]

use soroban_sdk::{contract, contractimpl, panic_with_error, symbol_short, Address, BytesN, Env};

pub mod components;
pub mod helpers;
pub mod systems;
pub mod types;

#[cfg(test)]
mod test;

use crate::helpers::assert_initialized;
use crate::systems::{PaymentSystem, RaceSystem};
use crate::types::{DataKey, GameState, PlayerStanding, ProofInput, Race, RacingError};

#[contract]
pub struct CrossAssetRacingLeague;

#[contractimpl]
impl CrossAssetRacingLeague {
    pub fn init_league(env: Env, owner: Address) {
        if env.storage().instance().has(&DataKey::Owner) {
            panic_with_error!(&env, RacingError::AlreadyInitialized);
        }

        owner.require_auth();

        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage().instance().set(&DataKey::CurrentSeason, &1u32);
        env.storage().instance().set(&DataKey::CurrentRaceId, &1u32);
        env.storage().instance().set(&DataKey::LeagueActive, &true);

        env.events().publish(
            (symbol_short!("init"),),
            ("League initialized", owner.clone()),
        );
    }

    pub fn create_race(env: Env, owner: Address, duration: u32) -> u32 {
        RaceSystem::create_race(&env, owner, duration)
    }

    pub fn enter_race(env: Env, player: Address, race_id: u32) {
        RaceSystem::enter_race(&env, player, race_id)
    }

    pub fn start_race(env: Env, owner: Address, race_id: u32) {
        RaceSystem::start_race(&env, owner, race_id)
    }

    pub fn activate_boost(env: Env, player: Address, race_id: u32, boost_type: u32) {
        RaceSystem::activate_boost(&env, player, race_id, boost_type)
    }

    pub fn credit_payment(
        env: Env,
        owner: Address,
        player: Address,
        amount: u32,
        receipt_hash: BytesN<32>,
    ) {
        PaymentSystem::credit_payment(&env, owner, player, amount, receipt_hash)
    }

    pub fn get_player_credits(env: Env, player: Address) -> u32 {
        assert_initialized(&env);
        let credit_key = DataKey::PlayerPaymentCredits(player);
        env.storage().persistent().get(&credit_key).unwrap_or(0)
    }

    pub fn submit_race_proof(env: Env, player: Address, proof: ProofInput) -> bool {
        RaceSystem::submit_race_proof(&env, player, proof)
    }

    pub fn complete_race(env: Env, owner: Address, race_id: u32) {
        RaceSystem::complete_race(&env, owner, race_id)
    }

    pub fn get_player_standing(env: Env, season_id: u32, player: Address) -> PlayerStanding {
        assert_initialized(&env);
        let standing_key = DataKey::PlayerStanding(season_id, player);
        env.storage()
            .instance()
            .get(&standing_key)
            .unwrap_or(PlayerStanding {
                points: 0,
                races_completed: 0,
                best_finish: u32::MAX,
                boost_count: 0,
            })
    }

    pub fn get_game_state(env: Env) -> GameState {
        assert_initialized(&env);
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("Not initialized");
        let current_season: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CurrentSeason)
            .unwrap_or(1);
        let current_race_id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CurrentRaceId)
            .unwrap_or(1);
        let league_active: bool = env
            .storage()
            .instance()
            .get(&DataKey::LeagueActive)
            .unwrap_or(false);

        GameState {
            owner,
            current_season,
            current_race_id,
            league_active,
        }
    }

    pub fn get_race(env: Env, race_id: u32) -> Race {
        env.storage()
            .instance()
            .get(&DataKey::Race(race_id))
            .expect("Race not found")
    }
}
