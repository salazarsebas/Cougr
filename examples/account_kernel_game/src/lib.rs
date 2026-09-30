//! account_kernel_game - Cougr game example demonstrating account-kernel
//! session authorization scaffolded via `cougr add account-kernel`.
//!
//! A player initializes a session key via `init_session`, then executes
//! `increment` on the counter contract, authorized through `AccountKernel`
//! and `authorize_action`.

#![no_std]

pub mod account_kernel;

use account_kernel::{authorize_action, counter_action};
use cougr_core::accounts::{AccountKernel, SessionKey, SessionStorage};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env, Symbol};

#[contract]
#[derive(Clone)]
pub struct AccountKernelGame;

#[contractimpl]
impl AccountKernelGame {
    /// Initialize a session key with a scoped operation budget and expiry.
    pub fn init_session(
        env: Env,
        account: Address,
        key_id: BytesN<32>,
        max_ops: u32,
        expires_at: u64,
    ) {
        account.require_auth();
        let scope = account_kernel::build_scope(&env, max_ops, expires_at);
        let session = SessionKey {
            key_id,
            scope,
            created_at: env.ledger().timestamp(),
            operations_used: 0,
            next_nonce: 0,
        };
        SessionStorage::store(&env, &account, &session);
    }

    /// Perform a counter increment authorized through the scaffolded account kernel.
    pub fn increment(
        env: Env,
        account: Address,
        key_id: BytesN<32>,
        nonce: u64,
        expires_at: u64,
    ) -> u32 {
        let kernel = AccountKernel::new(account.clone());
        let action = counter_action(&env);

        authorize_action(
            &env,
            &kernel,
            &account,
            &key_id,
            action,
            nonce,
            expires_at,
        )
        .unwrap_or_else(|_| panic!("session authorization rejected"));

        let key = Symbol::new(&env, "counter");
        let current: u32 = env.storage().instance().get(&key).unwrap_or(0);
        let updated = current.saturating_add(1);
        env.storage().instance().set(&key, &updated);
        updated
    }

    /// Read the current counter value.
    pub fn get_counter(env: Env) -> u32 {
        let key = Symbol::new(&env, "counter");
        env.storage().instance().get(&key).unwrap_or(0)
    }
}

#[cfg(test)]
mod test;
