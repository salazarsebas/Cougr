use soroban_sdk::{Address, Env, Symbol};

use super::error::AccountError;

const ACCOUNT_NONCE_PREFIX: &str = "acct_nonce";

/// Persistent nonce tracking for replay protection.
pub struct ReplayProtection;

impl ReplayProtection {
    pub fn next_account_nonce(env: &Env, account: &Address) -> u64 {
        let key = Self::storage_key(env, account);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    pub fn verify_and_consume_account_nonce(
        env: &Env,
        account: &Address,
        nonce: u64,
    ) -> Result<u64, AccountError> {
        let expected = Self::next_account_nonce(env, account);
        if nonce != expected {
            return Err(AccountError::NonceMismatch);
        }
        env.storage()
            .persistent()
            .set(&Self::storage_key(env, account), &(expected + 1));
        Ok(expected)
    }

    fn storage_key(env: &Env, account: &Address) -> (Symbol, Address) {
        (Symbol::new(env, ACCOUNT_NONCE_PREFIX), account.clone())
    }
}
