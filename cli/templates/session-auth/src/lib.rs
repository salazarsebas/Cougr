//! {{crate_name}} - a Cougr game contract generated from the `{{template_id}}` template.
//!
//! Approve once, then play. The owner signs a single wallet prompt to create a
//! scoped session key; every move after that is authorized by the session
//! instead of the wallet. When the session runs out the contract falls back to
//! direct owner auth rather than dropping the move.
//!
//! Flow: `approve_session` → `tap` many times → `renew_session` before expiry →
//! `fallback_tap` once the session is stale.
//!
//! Demonstrates:
//!   - `session::SessionManager` - session lifecycle (approve, renew, execute)
//!   - `accounts::SessionBuilder` - scoping a key to one action and a budget
//!   - `SorobanGame` - standard world load/save for the gameplay state

#![no_std]

pub mod components;
pub mod systems;
#[cfg(test)]
mod test;

use components::{player_key, Score};
use systems::{intent_deadline, tap_action, tap_scope, validate_request, SessionPolicyError};

use cougr_core::accounts::{ReplayProtection, SessionStorage, SignedIntent};
use cougr_core::game::SorobanGame;
use cougr_core::impl_soroban_game;
use cougr_core::session::{ActiveSession, SessionManager};
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

#[contract]
#[derive(Clone)]
pub struct {{ContractName}};

// Generates `load_world` / `save_world` against the "world" instance-storage key.
impl_soroban_game!({{ContractName}}, "world");

#[contractimpl]
impl {{ContractName}} {
    /// One-time owner approval that mints a scoped session key.
    ///
    /// This is the only call that prompts the wallet. The returned
    /// [`ActiveSession`] carries the key ID every later call needs, plus the
    /// expiry and remaining budget a client can render.
    pub fn approve_session(
        env: Env,
        owner: Address,
        max_taps: u32,
        expires_in: u64,
    ) -> ActiveSession {
        owner.require_auth();

        match validate_request(max_taps, expires_in) {
            Ok(()) => {}
            Err(SessionPolicyError::BudgetOutOfRange) => panic!("tap budget out of range"),
            Err(SessionPolicyError::LifetimeOutOfRange) => panic!("session lifetime out of range"),
        }

        let scope = tap_scope(&env, max_taps, expires_in);
        let key = SessionManager::approve(&env, &owner, scope).expect("session approved");
        let status =
            SessionManager::status(&env, &owner, &key.key_id).expect("session status readable");

        ActiveSession::from_status(&status, key.scope.expires_at)
    }

    /// Play a turn, authorized by the session key - no wallet prompt.
    ///
    /// Panics if the session is missing, expired, revoked, or out of budget.
    /// Use [`fallback_tap`](Self::fallback_tap) when a client should keep
    /// playing through an expiry instead.
    pub fn tap(env: Env, owner: Address, key_id: BytesN<32>) -> u32 {
        let session = SessionStorage::load(&env, &owner, &key_id).expect("session missing");

        SessionManager::execute_action(
            &env,
            &owner,
            &session,
            tap_action(&env),
            intent_deadline(&env),
        )
        .expect("session authorizes the tap");

        Self::award_tap(&env, &owner)
    }

    /// Play a turn, trying the session first and falling back to direct owner
    /// auth when it is expired, revoked, or out of budget.
    pub fn fallback_tap(env: Env, owner: Address, key_id: BytesN<32>) -> u32 {
        let session = SessionStorage::load(&env, &owner, &key_id).expect("session missing");
        let action = tap_action(&env);
        let deadline = intent_deadline(&env);

        let session_intent = SignedIntent::session(
            &env,
            owner.clone(),
            &key_id,
            action.clone(),
            session.next_nonce,
            deadline,
        );
        let direct_intent = SignedIntent::direct(
            &env,
            owner.clone(),
            action,
            ReplayProtection::next_account_nonce(&env, &owner),
            deadline,
        );

        SessionManager::fallback_execute(&env, &session_intent, &direct_intent)
            .expect("session or owner authorizes the tap");

        Self::award_tap(&env, &owner)
    }

    /// Extend a session's lifetime. The owner re-signs; the key ID is unchanged.
    pub fn renew_session(
        env: Env,
        owner: Address,
        key_id: BytesN<32>,
        expires_in: u64,
    ) -> ActiveSession {
        owner.require_auth();

        if expires_in == 0 || expires_in > systems::MAX_SESSION_LIFETIME {
            panic!("session lifetime out of range");
        }

        let expires_at = env.ledger().timestamp().saturating_add(expires_in);
        let key =
            SessionManager::renew(&env, &owner, &key_id, expires_at).expect("session renewed");
        let status =
            SessionManager::status(&env, &owner, &key.key_id).expect("session status readable");

        ActiveSession::from_status(&status, key.scope.expires_at)
    }

    /// Current health of a session: expiry, remaining budget, renewal hint.
    pub fn session_state(env: Env, owner: Address, key_id: BytesN<32>) -> Option<ActiveSession> {
        let status = SessionManager::status(&env, &owner, &key_id).ok()?;
        let session = SessionStorage::load(&env, &owner, &key_id)?;
        Some(ActiveSession::from_status(
            &status,
            session.scope.expires_at,
        ))
    }

    /// How many taps `owner` has played.
    pub fn score(env: Env, owner: Address) -> u32 {
        let Some(entity) = Self::entity_of(&env, &owner) else {
            return 0;
        };
        let world = {{ContractName}}::load_world(&env);
        world
            .get_typed::<Score>(&env, entity)
            .map(|score| score.taps)
            .unwrap_or(0)
    }

    // ─── Internal helpers ─────────────────────────────────────────────────────

    fn entity_of(env: &Env, owner: &Address) -> Option<u32> {
        env.storage().instance().get(&player_key(env, owner))
    }

    /// Credit one tap to `owner`, creating their entity on first play.
    fn award_tap(env: &Env, owner: &Address) -> u32 {
        let mut world = {{ContractName}}::load_world(env);

        let entity = match Self::entity_of(env, owner) {
            Some(entity) => entity,
            None => {
                let entity = world.spawn_entity();
                env.storage()
                    .instance()
                    .set(&player_key(env, owner), &entity);
                entity
            }
        };

        let taps = world
            .get_typed::<Score>(env, entity)
            .map(|score| score.taps)
            .unwrap_or(0)
            .saturating_add(1);

        world.set_typed(env, entity, &Score { taps });
        {{ContractName}}::save_world(env, &world);
        taps
    }
}
