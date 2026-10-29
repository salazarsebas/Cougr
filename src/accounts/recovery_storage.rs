//! Persistent storage for social recovery data.
//!
//! Follows the same pattern as [`SessionStorage`](super::storage::SessionStorage),
//! using Soroban's persistent contract storage keyed by account address.

use soroban_sdk::{Address, Env, Symbol, Vec};

use super::recovery::{Guardian, RecoveryConfig, RecoveryRequest};

const GUARDIANS_PREFIX: &str = "rcv_guard";
const CONFIG_PREFIX: &str = "rcv_conf";
const REQUEST_PREFIX: &str = "rcv_req";

/// Persistent storage for recovery guardians, config, and active requests.
pub struct RecoveryStorage;

impl RecoveryStorage {
    // --- Guardians ---

    /// Store the full guardians list for an account (overwrites).
    pub fn store_guardians(env: &Env, account: &Address, guardians: &Vec<Guardian>) {
        let key = Self::guardians_key(env, account);
        env.storage().persistent().set(&key, guardians);
    }

    /// Load the guardians list. Returns empty vec if none stored.
    pub fn load_guardians(env: &Env, account: &Address) -> Vec<Guardian> {
        let key = Self::guardians_key(env, account);
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    // --- Recovery Config ---

    /// Store recovery configuration for an account.
    pub fn store_config(env: &Env, account: &Address, config: &RecoveryConfig) {
        let key = Self::config_key(env, account);
        env.storage().persistent().set(&key, config);
    }

    /// Load recovery configuration. Returns None if not set.
    pub fn load_config(env: &Env, account: &Address) -> Option<RecoveryConfig> {
        let key = Self::config_key(env, account);
        env.storage().persistent().get(&key)
    }

    // --- Active Recovery Request ---

    /// Store an active recovery request.
    pub fn store_request(env: &Env, account: &Address, request: &RecoveryRequest) {
        let key = Self::request_key(env, account);
        env.storage().persistent().set(&key, request);
    }

    /// Load the active recovery request. Returns None if none active.
    pub fn load_request(env: &Env, account: &Address) -> Option<RecoveryRequest> {
        let key = Self::request_key(env, account);
        env.storage().persistent().get(&key)
    }

    /// Remove the active recovery request.
    pub fn remove_request(env: &Env, account: &Address) {
        let key = Self::request_key(env, account);
        env.storage().persistent().remove(&key);
    }

    // --- Storage keys ---

    fn guardians_key(env: &Env, account: &Address) -> (Symbol, Address) {
        (Symbol::new(env, GUARDIANS_PREFIX), account.clone())
    }

    fn config_key(env: &Env, account: &Address) -> (Symbol, Address) {
        (Symbol::new(env, CONFIG_PREFIX), account.clone())
    }

    fn request_key(env: &Env, account: &Address) -> (Symbol, Address) {
        (Symbol::new(env, REQUEST_PREFIX), account.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{contract, contractimpl, testutils::Address as _, Env};

    #[contract]
    pub struct TestContract;

    #[contractimpl]
    impl TestContract {}

    fn make_config() -> RecoveryConfig {
        RecoveryConfig {
            threshold: 2,
            timelock_period: 100,
            max_guardians: 5,
        }
    }

    #[test]
    fn test_store_and_load_guardians() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut guardians = Vec::new(&env);
            guardians.push_back(Guardian {
                address: Address::generate(&env),
                added_at: 0,
            });
            guardians.push_back(Guardian {
                address: Address::generate(&env),
                added_at: 10,
            });

            RecoveryStorage::store_guardians(&env, &addr, &guardians);
            let loaded = RecoveryStorage::load_guardians(&env, &addr);
            assert_eq!(loaded.len(), 2);
        });
    }

    #[test]
    fn test_load_guardians_empty() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let loaded = RecoveryStorage::load_guardians(&env, &addr);
            assert_eq!(loaded.len(), 0);
        });
    }

    #[test]
    fn test_store_and_load_config() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let config = make_config();
            RecoveryStorage::store_config(&env, &addr, &config);

            let loaded = RecoveryStorage::load_config(&env, &addr).unwrap();
            assert_eq!(loaded.threshold, 2);
            assert_eq!(loaded.timelock_period, 100);
            assert_eq!(loaded.max_guardians, 5);
        });
    }

    #[test]
    fn test_load_config_none() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            assert!(RecoveryStorage::load_config(&env, &addr).is_none());
        });
    }

    #[test]
    fn test_store_and_load_request() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let request = RecoveryRequest {
                new_owner: Address::generate(&env),
                approvals: Vec::new(&env),
                initiated_at: 50,
                timelock_until: 150,
                cancelled: false,
            };

            RecoveryStorage::store_request(&env, &addr, &request);
            let loaded = RecoveryStorage::load_request(&env, &addr).unwrap();
            assert_eq!(loaded.initiated_at, 50);
            assert_eq!(loaded.timelock_until, 150);
            assert!(!loaded.cancelled);
        });
    }

    #[test]
    fn test_remove_request() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let request = RecoveryRequest {
                new_owner: Address::generate(&env),
                approvals: Vec::new(&env),
                initiated_at: 50,
                timelock_until: 150,
                cancelled: false,
            };

            RecoveryStorage::store_request(&env, &addr, &request);
            assert!(RecoveryStorage::load_request(&env, &addr).is_some());

            RecoveryStorage::remove_request(&env, &addr);
            assert!(RecoveryStorage::load_request(&env, &addr).is_none());
        });
    }
}
