#![no_std]

use cougr_core::accounts::multi_device::{DeviceManager, DevicePolicy, MultiDeviceProvider};
use cougr_core::accounts::recovery::{RecoverableAccount, RecoveryConfig, RecoveryProvider};
use cougr_core::accounts::{GameAction, SessionBuilder, SessionStorage};
use cougr_core::session::{ActiveSession, SessionManager};
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol,
};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct MatchState {
    pub player1: Address,
    pub p1_moves: u32,
    pub player2: Address,
    pub p2_moves: u32,
}

#[contract]
#[derive(Clone)]
pub struct GuardianRecoveryMatch;

#[contractimpl]
impl GuardianRecoveryMatch {
    pub fn init_match(env: Env, player1: Address, player2: Address) {
        let state = MatchState {
            player1: player1.clone(),
            p1_moves: 0,
            player2: player2.clone(),
            p2_moves: 0,
        };
        env.storage()
            .instance()
            .set(&symbol_short!("match"), &state);

        // Setup recovery for player1
        let config = RecoveryConfig {
            threshold: 2,
            timelock_period: 0, // No timelock for test simplicity
            max_guardians: 5,
        };
        RecoverableAccount::new(player1.clone(), config, &env);

        // Setup multi-device for player1
        DeviceManager::with_defaults(player1, &env);
    }

    pub fn add_guardian(env: Env, player: Address, guardian: Address) {
        player.require_auth();
        let mut account = RecoverableAccount::load(player);
        account.add_guardian(&env, guardian).unwrap();
    }

    pub fn register_device(env: Env, player: Address, key_id: BytesN<32>, name: Symbol) {
        player.require_auth();
        let mut manager = DeviceManager::load(player);
        manager.register_device(&env, key_id, name).unwrap();
    }

    pub fn revoke_device(env: Env, player: Address, key_id: BytesN<32>) {
        player.require_auth();
        let mut manager = DeviceManager::load(player);
        manager.revoke_device(&env, &key_id).unwrap();
    }

    pub fn approve_session(env: Env, player: Address, max_moves: u32) -> ActiveSession {
        player.require_auth();
        let scope = SessionBuilder::new(&env)
            .allow_action(symbol_short!("move"))
            .max_operations(max_moves)
            .expires_in(1000)
            .build_scope();

        let key = SessionManager::approve(&env, &player, scope).expect("session approved");
        let status = SessionManager::status(&env, &player, &key.key_id).expect("session status");
        ActiveSession::from_status(&status, key.scope.expires_at)
    }

    pub fn make_move(env: Env, player: Address, key_id: BytesN<32>) -> u32 {
        let session = SessionStorage::load(&env, &player, &key_id).expect("session missing");

        // Ensure device is active
        let manager = DeviceManager::load(player.clone());
        let mut device_active = false;
        for dev in manager.list_devices(&env) {
            if dev.key_id == key_id {
                device_active = dev.is_active;
                break;
            }
        }
        if !device_active {
            panic!("device revoked");
        }

        let action = GameAction {
            system_name: symbol_short!("move"),
            data: soroban_sdk::Bytes::new(&env),
        };
        SessionManager::execute_action(
            &env,
            &player,
            &session,
            action,
            env.ledger().timestamp().saturating_add(60),
        )
        .expect("session execute");

        let mut state: MatchState = env
            .storage()
            .instance()
            .get(&symbol_short!("match"))
            .unwrap();
        let mut current_moves = 0;
        if state.player1 == player {
            state.p1_moves += 1;
            current_moves = state.p1_moves;
        } else if state.player2 == player {
            state.p2_moves += 1;
            current_moves = state.p2_moves;
        } else {
            panic!("not a player");
        }
        env.storage()
            .instance()
            .set(&symbol_short!("match"), &state);
        current_moves
    }

    pub fn initiate_recovery(env: Env, player: Address, new_owner: Address) {
        let mut account = RecoverableAccount::load(player);
        account.initiate_recovery(&env, new_owner).unwrap();
    }

    pub fn approve_recovery(env: Env, player: Address, guardian: Address) {
        guardian.require_auth();
        let mut account = RecoverableAccount::load(player);
        account.approve_recovery(&env, &guardian).unwrap();
    }

    pub fn execute_recovery(env: Env, player: Address) -> Address {
        let mut account = RecoverableAccount::load(player.clone());
        let new_owner = account.execute_recovery(&env).unwrap();

        // Update match state
        let mut state: MatchState = env
            .storage()
            .instance()
            .get(&symbol_short!("match"))
            .unwrap();
        if state.player1 == player {
            state.player1 = new_owner.clone();
        } else if state.player2 == player {
            state.player2 = new_owner.clone();
        }
        env.storage()
            .instance()
            .set(&symbol_short!("match"), &state);

        new_owner
    }

    pub fn get_match(env: Env) -> MatchState {
        env.storage()
            .instance()
            .get(&symbol_short!("match"))
            .unwrap()
    }
}

#[cfg(test)]
mod tests;
