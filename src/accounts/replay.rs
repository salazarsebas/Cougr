use soroban_sdk::{Address, Env, Symbol};

use super::error::AccountError;
use super::ttl::extend_persistent_ttl;

const ACCOUNT_NONCE_PREFIX: &str = "acct_nonce";

/// Persistent nonce tracking for replay protection.
pub struct ReplayProtection;

impl ReplayProtection {
    pub fn next_account_nonce(env: &Env, account: &Address) -> u64 {
        let key = Self::storage_key(env, account);
        match env.storage().persistent().get(&key) {
            Some(nonce) => {
                extend_persistent_ttl(env, &key);
                nonce
            }
            None => 0,
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
    use crate::accounts::ttl::persistent_ttl_bounds;
    use soroban_sdk::{
        contract, contractimpl,
        testutils::{storage::Persistent as _, Address as _, Ledger as _},
        Env,
    };

    #[contract]
    pub struct TestContract;

    #[contractimpl]
    impl TestContract {}

    #[test]
    fn test_nonce_write_and_read_extend_ttl() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let account = Address::generate(&env);

        let (threshold, extend_to) = env.as_contract(&contract_id, || persistent_ttl_bounds(&env));

        env.as_contract(&contract_id, || {
            let consumed =
                ReplayProtection::verify_and_consume_account_nonce(&env, &account, 0).unwrap();
            assert_eq!(consumed, 0);

            let key = ReplayProtection::storage_key(&env, &account);
            let ttl_after_write = env.storage().persistent().get_ttl(&key);
            assert!(
                ttl_after_write >= extend_to,
                "expected TTL >= {extend_to} after write, got {ttl_after_write}"
            );
        });

        // Advance past the default persistent TTL (4,096 ledgers in testutils)
        // and below the extension threshold so the next read triggers a bump.
        let advance = extend_to - threshold + 100;
        env.ledger().with_mut(|li| {
            li.sequence_number += advance;
        });

        env.as_contract(&contract_id, || {
            let key = ReplayProtection::storage_key(&env, &account);
            let ttl_before_read = env.storage().persistent().get_ttl(&key);
            assert!(
                ttl_before_read < threshold,
                "expected TTL < {threshold} before read, got {ttl_before_read}"
            );

            let next = ReplayProtection::next_account_nonce(&env, &account);
            assert_eq!(next, 1);

            let ttl_after_read = env.storage().persistent().get_ttl(&key);
            assert!(
                ttl_after_read >= extend_to,
                "expected TTL >= {extend_to} after read, got {ttl_after_read}"
            );
        });
    }
}
