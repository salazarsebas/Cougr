#![no_std]

mod components;
mod systems;
#[cfg(test)]
mod test;

use components::{ECSWorldState, GameState, RevealInfo, WORLD_KEY};
use systems::{reset_system, reveal_card_system, to_game_state, validate_player, validate_reveal};

use soroban_sdk::{contract, contractimpl, Address, Env};

// Main contract
#[contract]
pub struct MemoryMatchContract;

#[contractimpl]
impl MemoryMatchContract {
    pub fn init_game(env: Env, player: Address) -> GameState {
        let world_state = ECSWorldState::new(&env, player);
        env.storage().instance().set(&WORLD_KEY, &world_state);
        to_game_state(&env, &world_state)
    }

    pub fn reveal_card(env: Env, player: Address, position: u32) -> RevealInfo {
        let mut world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        validate_reveal(&world_state, &player, position);

        let result = reveal_card_system(&env, &mut world_state, position);

        // Save world state
        env.storage().instance().set(&WORLD_KEY, &world_state);

        result
    }

    pub fn get_game_state(env: Env) -> GameState {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        to_game_state(&env, &world_state)
    }

    pub fn reset_game(env: Env, player: Address) -> GameState {
        let mut world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        validate_player(&world_state, &player);

        reset_system(&env, &mut world_state);

        // Save world state
        env.storage().instance().set(&WORLD_KEY, &world_state);

        to_game_state(&env, &world_state)
    }
}
