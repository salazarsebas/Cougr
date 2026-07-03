//! fog_explorer — canonical Cougr ZK circuits example for fog-of-war exploration.

#![no_std]

use cougr_core::circuits::fog_of_war;
use cougr_core::zk::experimental::{FogOfWarSnapshot, FogOfWarTransition};
use cougr_core::zk::Groth16Proof;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol,
};

#[contracttype]
#[derive(Clone, Debug)]
pub struct MapConfig {
    pub width: u32,
    pub height: u32,
    pub visibility_radius: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ExplorerState {
    pub player: Address,
    pub explored_root: BytesN<32>,
    pub reveals: u32,
}

#[contract]
#[derive(Clone)]
pub struct FogExplorer;

#[contractimpl]
impl FogExplorer {
    pub fn init_map(env: Env, width: u32, height: u32, visibility_radius: u32) -> MapConfig {
        let _spec = fog_of_war(&env, width, height, visibility_radius).expect("valid map");
        let config = MapConfig {
            width,
            height,
            visibility_radius,
        };
        env.storage().instance().set(&map_key(&env), &config);
        config
    }

    pub fn register_explorer(
        env: Env,
        player: Address,
        explored_root: BytesN<32>,
    ) -> ExplorerState {
        let state = ExplorerState {
            player: player.clone(),
            explored_root,
            reveals: 0,
        };
        env.storage()
            .instance()
            .set(&explorer_key(&env, &player), &state);
        state
    }

    pub fn explore(
        env: Env,
        player: Address,
        map_root: BytesN<32>,
        prior_explored_root: BytesN<32>,
        next_explored_root: BytesN<32>,
        origin_x: i32,
        origin_y: i32,
        tile_x: i32,
        tile_y: i32,
        proof: Groth16Proof,
    ) -> bool {
        let config: MapConfig = env
            .storage()
            .instance()
            .get(&map_key(&env))
            .expect("map not initialized");
        let spec = fog_of_war(&env, config.width, config.height, config.visibility_radius)
            .expect("circuit spec");

        let snapshot = FogOfWarSnapshot {
            map_root,
            explored_root: prior_explored_root.clone(),
            origin_x,
            origin_y,
            visibility_radius: config.visibility_radius,
        };
        let transition = FogOfWarTransition {
            prior_explored_root,
            next_explored_root: next_explored_root.clone(),
            tile_x,
            tile_y,
        };

        let ok = spec
            .verify_fog_exploration(&env, &proof, &snapshot, &transition)
            .unwrap_or(false);
        if ok {
            let mut state: ExplorerState = env
                .storage()
                .instance()
                .get(&explorer_key(&env, &player))
                .expect("explorer missing");
            state.explored_root = next_explored_root;
            state.reveals = state.reveals.saturating_add(1);
            env.storage()
                .instance()
                .set(&explorer_key(&env, &player), &state);
        }
        ok
    }

    pub fn explorer_state(env: Env, player: Address) -> ExplorerState {
        env.storage()
            .instance()
            .get(&explorer_key(&env, &player))
            .expect("explorer missing")
    }
}

fn map_key(_env: &Env) -> Symbol {
    symbol_short!("map")
}

fn explorer_key(_env: &Env, player: &Address) -> (Symbol, Address) {
    (symbol_short!("expl"), player.clone())
}

#[cfg(test)]
mod tests;
