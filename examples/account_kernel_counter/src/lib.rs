//! account_kernel_counter – proves the `account-kernel` piece is usable in a
//! real Soroban game.
//!
//! ## What this example does
//!
//! A single-cell counter that can only be incremented by a holder of a valid
//! session key.  All authorization flows through the `account_kernel` module
//! that was scaffolded by `cougr add account-kernel`.
//!
//! ## Public API
//!
//! | Function | Description |
//! |---|---|
//! | `register_session` | Owner approves a scoped session (once per session) |
//! | `increment` | Increment counter – authorized via the session kernel |
//! | `count` | Read current counter value for an owner |
//!
//! ## Authorization flow
//!
//! ```text
//! register_session(owner, key_id, max_ops, expires_at)
//!   └─ build_scope()      ← from account_kernel piece
//!   └─ SessionStorage::store()
//!
//! increment(owner, key_id, nonce, intent_expires_at)
//!   └─ authorize_action() ← from account_kernel piece
//!      └─ kernel.authorize_session()
//!         └─ SessionPolicy (expiry + budget + nonce)
//!   └─ counter += 1 (only if authorize_action returns Ok)
//! ```

#![no_std]

pub mod account_kernel;

use cougr_core::accounts::{AccountKernel, SessionStorage};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env, Symbol};

/// Storage key for the per-owner counter.
fn counter_key(_env: &Env, owner: &Address) -> (Symbol, Address) {
    (symbol_short!("cnt"), owner.clone())
}

#[contract]
#[derive(Clone)]
pub struct CounterGame;

#[contractimpl]
impl CounterGame {
    /// Owner approves a scoped session that permits `tap` actions.
    ///
    /// Must be called once before `increment`.  The caller supplies a
    /// `key_id` so tests can control which identifier is used.
    pub fn register_session(
        env: Env,
        owner: Address,
        key_id: BytesN<32>,
        max_ops: u32,
        expires_at: u64,
    ) {
        owner.require_auth();

        let scope = account_kernel::build_scope(&env, max_ops, expires_at);
        let session_key = cougr_core::accounts::SessionKey {
            key_id,
            scope,
            created_at: env.ledger().timestamp(),
            operations_used: 0,
            next_nonce: 0,
        };
        SessionStorage::store(&env, &owner, &session_key);
    }

    /// Increment the counter for `owner`.
    ///
    /// Authorization is delegated entirely to `account_kernel::authorize_action`
    /// – the unmodified piece output.  If the session is expired, the budget is
    /// exhausted, or the nonce has been replayed, this call panics.
    pub fn increment(
        env: Env,
        owner: Address,
        key_id: BytesN<32>,
        nonce: u64,
        intent_expires_at: u64,
    ) -> u32 {
        let kernel = AccountKernel::new(owner.clone());
        let action = account_kernel::counter_action(&env);

        account_kernel::authorize_action(
            &env,
            &kernel,
            &owner,
            &key_id,
            action,
            nonce,
            intent_expires_at,
        )
        .expect("authorization failed");

        let key = counter_key(&env, &owner);
        let prev: u32 = env.storage().instance().get(&key).unwrap_or(0);
        let next = prev.saturating_add(1);
        env.storage().instance().set(&key, &next);
        next
    }

    /// Return the current counter value for `owner`.
    pub fn count(env: Env, owner: Address) -> u32 {
        env.storage()
            .instance()
            .get(&counter_key(&env, &owner))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests;
