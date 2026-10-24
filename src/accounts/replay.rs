use soroban_sdk::{Address, Env, Symbol};

use super::error::AccountError;
use super::ttl::extend_persistent_ttl;

const ACCOUNT_NONCE_PREFIX: &str = "acct_nonce";

/// Persistent nonce tracking for replay protection.
pub struct ReplayProtection;

impl ReplayProtection {
    pub fn next_account_nonce(env: &Env, account: &Address) -> u64 {
        let key = Self::storage_key(env, account);
        if let Some(nonce) = env.storage().persistent().get(&key) {
            extend_persistent_ttl(env, &key);
            nonce
        } else {
            0
        }
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
        let key = Self::storage_key(env, account);
        env.storage().persistent().set(&key, &(expected + 1));
        extend_persistent_ttl(env, &key);
        Ok(expected)
    }

    fn storage_key(env: &Env, account: &Address) -> (Symbol, Address) {
        (Symbol::new(env, ACCOUNT_NONCE_PREFIX), account.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::ttl::{ACCOUNT_TTL_EXTEND_TO, ACCOUNT_TTL_THRESHOLD};
    use soroban_sdk::{
        contract, contractimpl,
        testutils::{Address as _, Ledger as _},
        Env,
    };

    #[contract]
    pub struct TestContract;

    #[contractimpl]
    impl TestContract {}

    #[test]
    fn test_verify_and_consume_extends_nonce_ttl_on_write() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let account = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let key = ReplayProtection::storage_key(&env, &account);
            assert_eq!(ReplayProtection::next_account_nonce(&env, &account), 0);

            let consumed =
                ReplayProtection::verify_and_consume_account_nonce(&env, &account, 0).unwrap();
            assert_eq!(consumed, 0);

            let initial_ttl = env.storage().persistent().get_ttl(&key);
            assert!(initial_ttl >= ACCOUNT_TTL_EXTEND_TO);
        });

        env.ledger().with_mut(|li| {
            li.sequence_number += ACCOUNT_TTL_EXTEND_TO - ACCOUNT_TTL_THRESHOLD + 100;
        });

        env.as_contract(&contract_id, || {
            let key = ReplayProtection::storage_key(&env, &account);
            let ttl_before = env.storage().persistent().get_ttl(&key);
            assert!(ttl_before < ACCOUNT_TTL_THRESHOLD);

            let consumed =
                ReplayProtection::verify_and_consume_account_nonce(&env, &account, 1).unwrap();
            assert_eq!(consumed, 1);

            let ttl_after = env.storage().persistent().get_ttl(&key);
            assert!(ttl_after >= ACCOUNT_TTL_EXTEND_TO);
        });
    }

    #[test]
    fn test_next_account_nonce_extends_ttl_on_read() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let account = Address::generate(&env);

        env.as_contract(&contract_id, || {
            ReplayProtection::verify_and_consume_account_nonce(&env, &account, 0).unwrap();
        });

        env.ledger().with_mut(|li| {
            li.sequence_number += ACCOUNT_TTL_EXTEND_TO - ACCOUNT_TTL_THRESHOLD + 100;
        });

        env.as_contract(&contract_id, || {
            let key = ReplayProtection::storage_key(&env, &account);
            let ttl_before = env.storage().persistent().get_ttl(&key);
            assert!(ttl_before < ACCOUNT_TTL_THRESHOLD);

            let nonce = ReplayProtection::next_account_nonce(&env, &account);
            assert_eq!(nonce, 1);

            let ttl_after = env.storage().persistent().get_ttl(&key);
            assert!(ttl_after >= ACCOUNT_TTL_EXTEND_TO);
        });
    }
}
