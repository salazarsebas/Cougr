//! Persistent storage for multi-device key management.
//!
//! Follows the same pattern as [`SessionStorage`](super::storage::SessionStorage),
//! using Soroban's persistent contract storage keyed by account address.

use soroban_sdk::{Address, BytesN, Env, Symbol, Vec};

use super::error::AccountError;
use super::multi_device::{DeviceKey, DevicePolicy};

const DEVICES_PREFIX: &str = "dev_keys";
const POLICY_PREFIX: &str = "dev_policy";

/// Persistent storage for device keys and device policy.
pub struct DeviceStorage;

impl DeviceStorage {
    // --- Device Keys ---

    /// Store the full device key list for an account (overwrites).
    pub fn store_devices(env: &Env, account: &Address, devices: &Vec<DeviceKey>) {
        let key = Self::devices_key(env, account);
        env.storage().persistent().set(&key, devices);
    }

    /// Load all device keys. Returns empty vec if none stored.
    pub fn load_devices(env: &Env, account: &Address) -> Vec<DeviceKey> {
        let key = Self::devices_key(env, account);
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    // --- Device Policy ---

    /// Store device management policy for an account.
    pub fn store_policy(env: &Env, account: &Address, policy: &DevicePolicy) {
        let key = Self::policy_key(env, account);
        env.storage().persistent().set(&key, policy);
    }

    /// Load device policy. Returns None if not set.
    pub fn load_policy(env: &Env, account: &Address) -> Option<DevicePolicy> {
        let key = Self::policy_key(env, account);
        env.storage().persistent().get(&key)
    }

    // --- Helpers ---

    /// Update a specific device key by ID. Loads all devices, applies the
    /// updater to the first ACTIVE matching device, and writes back.
    pub fn update_device(
        env: &Env,
        account: &Address,
        key_id: &BytesN<32>,
        updater: impl FnOnce(&mut DeviceKey),
    ) -> Result<(), AccountError> {
        let devices = Self::load_devices(env, account);
        let mut new_devices: Vec<DeviceKey> = Vec::new(env);
        let mut found = false;
        let mut updater = Some(updater);

        for i in 0..devices.len() {
            if let Some(mut d) = devices.get(i) {
                if &d.key_id == key_id && d.is_active {
                    found = true;
                    if let Some(f) = updater.take() {
                        f(&mut d);
                    }
                }
                new_devices.push_back(d);
            }
        }

        if !found {
            return Err(AccountError::DeviceNotFound);
        }

        Self::store_devices(env, account, &new_devices);
        Ok(())
    }

    // --- Storage keys ---

    fn devices_key(env: &Env, account: &Address) -> (Symbol, Address) {
        (Symbol::new(env, DEVICES_PREFIX), account.clone())
    }

    fn policy_key(env: &Env, account: &Address) -> (Symbol, Address) {
        (Symbol::new(env, POLICY_PREFIX), account.clone())
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

    fn make_device(env: &Env, id_byte: u8, name: &str) -> DeviceKey {
        DeviceKey {
            key_id: BytesN::from_array(env, &[id_byte; 32]),
            device_name: Symbol::new(env, name),
            registered_at: 0,
            last_used: 0,
            is_active: true,
        }
    }

    #[test]
    fn test_store_and_load_devices() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut devices = Vec::new(&env);
            devices.push_back(make_device(&env, 1, "phone"));
            devices.push_back(make_device(&env, 2, "laptop"));

            DeviceStorage::store_devices(&env, &addr, &devices);
            let loaded = DeviceStorage::load_devices(&env, &addr);
            assert_eq!(loaded.len(), 2);
        });
    }

    #[test]
    fn test_load_devices_empty() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let loaded = DeviceStorage::load_devices(&env, &addr);
            assert_eq!(loaded.len(), 0);
        });
    }

    #[test]
    fn test_store_and_load_policy() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let policy = DevicePolicy {
                max_devices: 5,
                auto_revoke_after: 1000,
            };
            DeviceStorage::store_policy(&env, &addr, &policy);

            let loaded = DeviceStorage::load_policy(&env, &addr).unwrap();
            assert_eq!(loaded.max_devices, 5);
            assert_eq!(loaded.auto_revoke_after, 1000);
        });
    }

    #[test]
    fn test_load_policy_none() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            assert!(DeviceStorage::load_policy(&env, &addr).is_none());
        });
    }

    #[test]
    fn test_update_device() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut devices = Vec::new(&env);
            devices.push_back(make_device(&env, 1, "phone"));

            DeviceStorage::store_devices(&env, &addr, &devices);

            let key_id = BytesN::from_array(&env, &[1u8; 32]);
            DeviceStorage::update_device(&env, &addr, &key_id, |d| {
                d.last_used = 999;
            })
            .unwrap();

            let loaded = DeviceStorage::load_devices(&env, &addr);
            assert_eq!(loaded.get(0).unwrap().last_used, 999);
        });
    }
}
