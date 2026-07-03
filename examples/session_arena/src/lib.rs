//! session_arena — canonical Cougr session UX example.
//!
//! Flow: `approve_session` once → `tap` many times without wallet prompts →
//! `renew_session` before expiry → `fallback_tap` when session is stale.

#![no_std]

use cougr_core::accounts::{
    GameAction, ReplayProtection, SessionBuilder, SessionStorage, SignedIntent,
};
use cougr_core::impl_component;
use cougr_core::session::{ActiveSession, SessionManager};
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Score {
    pub taps: u32,
}

impl_component!(Score, "score", Table, { taps: u32 });

#[contract]
#[derive(Clone)]
pub struct SessionArena;

#[contractimpl]
impl SessionArena {
    /// One-time owner approval that creates a scoped session key.
    pub fn approve_session(
        env: Env,
        owner: Address,
        max_taps: u32,
        expires_in: u64,
    ) -> ActiveSession {
        owner.require_auth();
        let scope = SessionBuilder::new(&env)
            .allow_action(symbol_short!("tap"))
            .max_operations(max_taps)
            .expires_in(expires_in)
            .build_scope();

        let key = SessionManager::approve(&env, &owner, scope).expect("session approved");
        let status = SessionManager::status(&env, &owner, &key.key_id).expect("session status");
        ActiveSession::from_status(&status, key.scope.expires_at)
    }

    /// Gameplay action authorized via the active session (no wallet prompt).
    pub fn tap(env: Env, owner: Address, key_id: soroban_sdk::BytesN<32>) -> u32 {
        let session = SessionStorage::load(&env, &owner, &key_id).expect("session missing");
        let action = GameAction {
            system_name: symbol_short!("tap"),
            data: soroban_sdk::Bytes::new(&env),
        };
        SessionManager::execute_action(
            &env,
            &owner,
            &session,
            action,
            env.ledger().timestamp().saturating_add(60),
        )
        .expect("session tap");

        // Simple score counter stored in instance storage for the demo.
        let key = (Symbol::new(&env, "score"), owner.clone());
        let mut score: Score = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or(Score { taps: 0 });
        score.taps = score.taps.saturating_add(1);
        env.storage().instance().set(&key, &score);
        score.taps
    }

    /// Extend session lifetime (owner must re-approve via wallet).
    pub fn renew_session(
        env: Env,
        owner: Address,
        key_id: soroban_sdk::BytesN<32>,
        expires_in: u64,
    ) -> ActiveSession {
        owner.require_auth();
        let new_expires = env.ledger().timestamp().saturating_add(expires_in);
        let key = SessionManager::renew(&env, &owner, &key_id, new_expires).expect("renewed");
        let status = SessionManager::status(&env, &owner, &key.key_id).expect("session status");
        ActiveSession::from_status(&status, key.scope.expires_at)
    }

    /// Tap using session first, falling back to direct owner auth when expired.
    pub fn fallback_tap(env: Env, owner: Address, key_id: soroban_sdk::BytesN<32>) -> u32 {
        let session = SessionStorage::load(&env, &owner, &key_id).expect("session missing");
        let action = GameAction {
            system_name: symbol_short!("tap"),
            data: soroban_sdk::Bytes::new(&env),
        };
        let session_intent = SignedIntent::session(
            &env,
            owner.clone(),
            &key_id,
            action.clone(),
            session.next_nonce,
            env.ledger().timestamp().saturating_add(60),
        );
        let direct_intent = SignedIntent::direct(
            &env,
            owner.clone(),
            action,
            ReplayProtection::next_account_nonce(&env, &owner),
            env.ledger().timestamp().saturating_add(60),
        );
        SessionManager::fallback_execute(&env, &session_intent, &direct_intent)
            .expect("fallback tap");

        let key = (Symbol::new(&env, "score"), owner.clone());
        let mut score: Score = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or(Score { taps: 0 });
        score.taps = score.taps.saturating_add(1);
        env.storage().instance().set(&key, &score);
        score.taps
    }

    pub fn score(env: Env, owner: Address) -> u32 {
        let key = (Symbol::new(&env, "score"), owner);
        env.storage()
            .instance()
            .get(&key)
            .map(|s: Score| s.taps)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests;
