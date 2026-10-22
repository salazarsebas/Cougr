use soroban_sdk::{contracttype, Address, BytesN, Env, Symbol, Vec};

use super::device_storage::DeviceStorage;
use super::error::AccountError;

/// A registered device key with metadata.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceKey {
    /// Unique identifier for this device key.
    pub key_id: BytesN<32>,
    /// Human-readable device name (e.g., "phone", "laptop").
    pub device_name: Symbol,
    /// Ledger timestamp when the device was registered.
    pub registered_at: u64,
    /// Ledger timestamp of the last use.
    pub last_used: u64,
    /// Whether this device key is currently active.
    pub is_active: bool,
}

/// Policy for multi-device management.
#[contracttype]
#[derive(Clone, Debug)]
pub struct DevicePolicy {
    /// Maximum number of devices that can be registered.
    pub max_devices: u32,
    /// Number of seconds of inactivity before auto-revoke.
    /// Set to 0 to disable auto-revoke.
    pub auto_revoke_after: u64,
}

/// Trait for account types that support multi-device key management.
pub trait MultiDeviceProvider {
    /// Register a new device key.
    fn register_device(
        &mut self,
        env: &Env,
        key_id: BytesN<32>,
        device_name: Symbol,
    ) -> Result<DeviceKey, AccountError>;

    /// Revoke a device key by its ID.
    fn revoke_device(&mut self, env: &Env, key_id: &BytesN<32>) -> Result<(), AccountError>;

    /// List all registered device keys (active and inactive).
    fn list_devices(&self, env: &Env) -> Vec<DeviceKey>;

    /// Returns the number of active devices.
    fn active_device_count(&self, env: &Env) -> usize;

    /// Update the last_used timestamp for a device.
    fn update_last_used(&mut self, env: &Env, key_id: &BytesN<32>) -> Result<(), AccountError>;

    /// Set the device management policy.
    fn set_policy(&mut self, env: &Env, policy: DevicePolicy);

    /// Get the current device policy.
    fn policy(&self, env: &Env) -> DevicePolicy;

    /// Revoke devices that have been inactive beyond the policy's auto_revoke_after.
    /// Returns the number of devices revoked.
    fn cleanup_inactive(&mut self, env: &Env) -> u32;
}

/// Persistent implementation of multi-device management.
///
/// Device keys and policy are stored in Soroban persistent storage
/// via [`DeviceStorage`], surviving across contract invocations.
pub struct DeviceManager {
    address: Address,
}

impl DeviceManager {
    /// Create a new device manager with the given policy, persisting it.
    pub fn new(address: Address, policy: DevicePolicy, env: &Env) -> Self {
        DeviceStorage::store_policy(env, &address, &policy);
        DeviceStorage::store_devices(env, &address, &Vec::new(env));
        Self { address }
    }

    /// Load an existing device manager (policy must already be stored).
    pub fn load(address: Address) -> Self {
        Self { address }
    }

    /// Create a device manager with a default policy.
    pub fn with_defaults(address: Address, env: &Env) -> Self {
        let policy = DevicePolicy {
            max_devices: 5,
            auto_revoke_after: 0,
        };
        Self::new(address, policy, env)
    }
}

impl MultiDeviceProvider for DeviceManager {
    fn register_device(
        &mut self,
        env: &Env,
        key_id: BytesN<32>,
        device_name: Symbol,
    ) -> Result<DeviceKey, AccountError> {
        let policy =
            DeviceStorage::load_policy(env, &self.address).ok_or(AccountError::StorageError)?;
        let devices = DeviceStorage::load_devices(env, &self.address);

        // Check for duplicate active key_id and count active devices
        let mut active_count: u32 = 0;
        let mut revoked_index: Option<usize> = None;

        for i in 0..devices.len() {
            if let Some(d) = devices.get(i) {
                if d.key_id == key_id {
                    if d.is_active {
                        // Key is already registered and active
                        return Err(AccountError::DuplicateDeviceKey);
                    } else {
                        // Found a revoked entry with this key_id - we'll reuse it
                        revoked_index = Some(i as usize);
                    }
                }
                if d.is_active {
                    active_count += 1;
                }
            }
        }

        if active_count >= policy.max_devices {
            return Err(AccountError::DeviceLimitReached);
        }

        let now = env.ledger().timestamp();
        let device = DeviceKey {
            key_id,
            device_name,
            registered_at: now,
            last_used: now,
            is_active: true,
        };

        let mut new_devices: Vec<DeviceKey> = Vec::new(env);

        if let Some(revoked_idx) = revoked_index {
            // Reuse the revoked entry by replacing it
            for i in 0..devices.len() {
                if i == revoked_idx as u32 {
                    new_devices.push_back(device.clone());
                } else if let Some(d) = devices.get(i) {
                    new_devices.push_back(d);
                }
            }
        } else {
            // No revoked entry to reuse, add as new
            new_devices = devices;
            new_devices.push_back(device.clone());
        }

        DeviceStorage::store_devices(env, &self.address, &new_devices);
        Ok(device)
    }

    fn revoke_device(&mut self, env: &Env, key_id: &BytesN<32>) -> Result<(), AccountError> {
        let devices = DeviceStorage::load_devices(env, &self.address);
        let mut found_active = false;

        // Search for an active entry with this key_id
        for i in 0..devices.len() {
            if let Some(d) = devices.get(i) {
                if &d.key_id == key_id && d.is_active {
                    found_active = true;
                    break;
                }
            }
        }

        if !found_active {
            return Err(AccountError::DeviceNotFound);
        }

        DeviceStorage::update_device(env, &self.address, key_id, |d| {
            d.is_active = false;
        })
    }

    fn list_devices(&self, env: &Env) -> Vec<DeviceKey> {
        DeviceStorage::load_devices(env, &self.address)
    }

    fn active_device_count(&self, env: &Env) -> usize {
        let devices = DeviceStorage::load_devices(env, &self.address);
        let mut count: usize = 0;
        for i in 0..devices.len() {
            if let Some(d) = devices.get(i) {
                if d.is_active {
                    count += 1;
                }
            }
        }
        count
    }

    fn update_last_used(&mut self, env: &Env, key_id: &BytesN<32>) -> Result<(), AccountError> {
        let now = env.ledger().timestamp();
        DeviceStorage::update_device(env, &self.address, key_id, |d| {
            d.last_used = now;
        })
    }

    fn set_policy(&mut self, env: &Env, policy: DevicePolicy) {
        DeviceStorage::store_policy(env, &self.address, &policy);
    }

    fn policy(&self, env: &Env) -> DevicePolicy {
        DeviceStorage::load_policy(env, &self.address).unwrap_or(DevicePolicy {
            max_devices: 5,
            auto_revoke_after: 0,
        })
    }

    fn cleanup_inactive(&mut self, env: &Env) -> u32 {
        let policy = DeviceStorage::load_policy(env, &self.address).unwrap_or(DevicePolicy {
            max_devices: 5,
            auto_revoke_after: 0,
        });

        if policy.auto_revoke_after == 0 {
            return 0;
        }

        let now = env.ledger().timestamp();
        let threshold = policy.auto_revoke_after;
        let devices = DeviceStorage::load_devices(env, &self.address);
        let mut new_devices: Vec<DeviceKey> = Vec::new(env);
        let mut revoked: u32 = 0;

        for i in 0..devices.len() {
            if let Some(mut d) = devices.get(i) {
                if d.is_active && now.saturating_sub(d.last_used) > threshold {
                    d.is_active = false;
                    revoked += 1;
                }
                new_devices.push_back(d);
            }
        }

        if revoked > 0 {
            DeviceStorage::store_devices(env, &self.address, &new_devices);
        }

        revoked
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::policy::{ActiveDevicePolicy, DeviceContext, Policy};
    use soroban_sdk::testutils::Ledger;
    use soroban_sdk::{contract, contractimpl, symbol_short, testutils::Address as _, Env};

    #[contract]
    pub struct TestContract;

    #[contractimpl]
    impl TestContract {}

    fn default_policy() -> DevicePolicy {
        DevicePolicy {
            max_devices: 3,
            auto_revoke_after: 0,
        }
    }

    #[test]
    fn test_register_device() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut manager = DeviceManager::new(addr, default_policy(), &env);
            let key_id = BytesN::from_array(&env, &[1u8; 32]);
            let device = manager
                .register_device(&env, key_id, symbol_short!("phone"))
                .unwrap();

            assert!(device.is_active);
            assert_eq!(manager.active_device_count(&env), 1);
        });
    }

    #[test]
    fn test_device_limit() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let policy = DevicePolicy {
                max_devices: 2,
                auto_revoke_after: 0,
            };
            let mut manager = DeviceManager::new(addr, policy, &env);

            manager
                .register_device(
                    &env,
                    BytesN::from_array(&env, &[1u8; 32]),
                    symbol_short!("dev1"),
                )
                .unwrap();
            manager
                .register_device(
                    &env,
                    BytesN::from_array(&env, &[2u8; 32]),
                    symbol_short!("dev2"),
                )
                .unwrap();

            let result = manager.register_device(
                &env,
                BytesN::from_array(&env, &[3u8; 32]),
                symbol_short!("dev3"),
            );
            assert_eq!(result, Err(AccountError::DeviceLimitReached));
        });
    }

    #[test]
    fn test_revoke_device() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut manager = DeviceManager::new(addr, default_policy(), &env);
            let key_id = BytesN::from_array(&env, &[1u8; 32]);

            manager
                .register_device(&env, key_id.clone(), symbol_short!("phone"))
                .unwrap();
            manager.revoke_device(&env, &key_id).unwrap();

            assert_eq!(manager.active_device_count(&env), 0);
            assert_eq!(manager.list_devices(&env).len(), 1); // still in list, just inactive
        });
    }

    #[test]
    fn test_revoke_nonexistent() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut manager = DeviceManager::new(addr, default_policy(), &env);
            let key_id = BytesN::from_array(&env, &[99u8; 32]);

            let result = manager.revoke_device(&env, &key_id);
            assert_eq!(result, Err(AccountError::DeviceNotFound));
        });
    }

    #[test]
    fn test_update_last_used() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut manager = DeviceManager::new(addr, default_policy(), &env);
            let key_id = BytesN::from_array(&env, &[1u8; 32]);

            manager
                .register_device(&env, key_id.clone(), symbol_short!("phone"))
                .unwrap();
            manager.update_last_used(&env, &key_id).unwrap();

            let devices = manager.list_devices(&env);
            assert_eq!(devices.len(), 1);
        });
    }

    #[test]
    fn test_update_last_used_nonexistent() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut manager = DeviceManager::new(addr, default_policy(), &env);
            let key_id = BytesN::from_array(&env, &[99u8; 32]);

            let result = manager.update_last_used(&env, &key_id);
            assert_eq!(result, Err(AccountError::DeviceNotFound));
        });
    }

    #[test]
    fn test_cleanup_inactive_disabled() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut manager = DeviceManager::new(addr, default_policy(), &env);

            manager
                .register_device(
                    &env,
                    BytesN::from_array(&env, &[1u8; 32]),
                    symbol_short!("phone"),
                )
                .unwrap();

            let revoked = manager.cleanup_inactive(&env);
            assert_eq!(revoked, 0);
            assert_eq!(manager.active_device_count(&env), 1);
        });
    }

    #[test]
    fn test_set_policy() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut manager = DeviceManager::new(addr, default_policy(), &env);
            let new_policy = DevicePolicy {
                max_devices: 10,
                auto_revoke_after: 500,
            };
            manager.set_policy(&env, new_policy);
            assert_eq!(manager.policy(&env).max_devices, 10);
            assert_eq!(manager.policy(&env).auto_revoke_after, 500);
        });
    }

    #[test]
    fn test_with_defaults() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let manager = DeviceManager::with_defaults(addr, &env);
            assert_eq!(manager.policy(&env).max_devices, 5);
            assert_eq!(manager.active_device_count(&env), 0);
        });
    }

    #[test]
    fn test_revoked_device_allows_new_registration() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let policy = DevicePolicy {
                max_devices: 1,
                auto_revoke_after: 0,
            };
            let mut manager = DeviceManager::new(addr, policy, &env);

            let key1 = BytesN::from_array(&env, &[1u8; 32]);
            manager
                .register_device(&env, key1.clone(), symbol_short!("old"))
                .unwrap();
            manager.revoke_device(&env, &key1).unwrap();

            // Should be able to register again since active count is 0
            let key2 = BytesN::from_array(&env, &[2u8; 32]);
            manager
                .register_device(&env, key2, symbol_short!("new"))
                .unwrap();
            assert_eq!(manager.active_device_count(&env), 1);
        });
    }

    #[test]
    fn test_reregister_same_key_after_revoke() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut manager = DeviceManager::new(addr.clone(), default_policy(), &env);
            let key_id = BytesN::from_array(&env, &[1u8; 32]);

            // Register, revoke, then register the SAME key again
            manager
                .register_device(&env, key_id.clone(), symbol_short!("phone"))
                .unwrap();
            manager.revoke_device(&env, &key_id).unwrap();
            assert_eq!(manager.active_device_count(&env), 0);

            // Re-register with same key_id
            manager
                .register_device(&env, key_id.clone(), symbol_short!("phone2"))
                .unwrap();
            assert_eq!(manager.active_device_count(&env), 1);

            // After re-register, ActiveDevicePolicy must accept the key
            let device_policy = ActiveDevicePolicy;
            device_policy
                .evaluate(
                    &env,
                    &DeviceContext {
                        account: &addr,
                        key_id: &key_id,
                    },
                )
                .unwrap();

            // Now revoke again - should work
            manager.revoke_device(&env, &key_id).unwrap();
            assert_eq!(manager.active_device_count(&env), 0);

            // After second revoke, ActiveDevicePolicy must reject the key
            assert_eq!(
                device_policy.evaluate(
                    &env,
                    &DeviceContext {
                        account: &addr,
                        key_id: &key_id,
                    },
                ),
                Err(AccountError::DeviceNotFound)
            );
        });
    }

    #[test]
    fn test_reregister_same_key_updates_last_used() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let policy = DevicePolicy {
                max_devices: 3,
                auto_revoke_after: 1000,
            };
            let mut manager = DeviceManager::new(addr, policy, &env);
            let key_id = BytesN::from_array(&env, &[1u8; 32]);

            // Register, revoke, re-register
            manager
                .register_device(&env, key_id.clone(), symbol_short!("phone"))
                .unwrap();
            manager.revoke_device(&env, &key_id).unwrap();
            manager
                .register_device(&env, key_id.clone(), symbol_short!("phone2"))
                .unwrap();

            // Advance time past auto-revoke threshold
            env.ledger().with_mut(|li| {
                li.timestamp = 2000;
            });

            // Update last_used on the re-registered key (this should set last_used to 2000)
            manager.update_last_used(&env, &key_id).unwrap();

            // Cleanup should NOT revoke it because we just updated last_used to 2000
            let revoked = manager.cleanup_inactive(&env);
            assert_eq!(revoked, 0);
            assert_eq!(manager.active_device_count(&env), 1);
        });
    }

    #[test]
    fn test_duplicate_active_key_error() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut manager = DeviceManager::new(addr, default_policy(), &env);
            let key_id = BytesN::from_array(&env, &[1u8; 32]);

            // Register once
            manager
                .register_device(&env, key_id.clone(), symbol_short!("phone"))
                .unwrap();

            // Try to register the same active key again
            let result = manager.register_device(&env, key_id, symbol_short!("phone2"));
            assert_eq!(result, Err(AccountError::DuplicateDeviceKey));
        });
    }

    #[test]
    fn test_revoke_already_revoked_device_fails() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut manager = DeviceManager::new(addr, default_policy(), &env);
            let key_id = BytesN::from_array(&env, &[1u8; 32]);

            manager
                .register_device(&env, key_id.clone(), symbol_short!("phone"))
                .unwrap();
            manager.revoke_device(&env, &key_id).unwrap();

            // Try to revoke again - should fail
            let result = manager.revoke_device(&env, &key_id);
            assert_eq!(result, Err(AccountError::DeviceNotFound));
        });
    }

    #[test]
    fn test_cleanup_inactive_with_reregistered_key() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let policy = DevicePolicy {
                max_devices: 3,
                auto_revoke_after: 500,
            };
            let mut manager = DeviceManager::new(addr, policy, &env);
            let key_id = BytesN::from_array(&env, &[1u8; 32]);

            // Register and use at time 0
            manager
                .register_device(&env, key_id.clone(), symbol_short!("phone"))
                .unwrap();

            // Advance time and revoke
            env.ledger().with_mut(|li| {
                li.timestamp = 100;
            });
            manager.revoke_device(&env, &key_id).unwrap();

            // Re-register at time 100
            manager
                .register_device(&env, key_id.clone(), symbol_short!("phone2"))
                .unwrap();

            // Advance time to 700 (600 seconds after re-registration)
            env.ledger().with_mut(|li| {
                li.timestamp = 700;
            });

            // Should auto-revoke because last_used was at registration (time 100)
            // and now it's been > 500 seconds
            let revoked = manager.cleanup_inactive(&env);
            assert_eq!(revoked, 1);
            assert_eq!(manager.active_device_count(&env), 0);
        });
    }

    #[test]
    fn test_revoke_with_stale_duplicate_in_storage() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            // Simulate legacy state: stale revoked entry followed by live active entry
            let key_id = BytesN::from_array(&env, &[1u8; 32]);
            let mut devices = Vec::new(&env);

            // Stale entry (revoked)
            devices.push_back(DeviceKey {
                key_id: key_id.clone(),
                device_name: symbol_short!("old"),
                registered_at: 0,
                last_used: 0,
                is_active: false,
            });

            // Live entry (active)
            devices.push_back(DeviceKey {
                key_id: key_id.clone(),
                device_name: symbol_short!("new"),
                registered_at: 0,
                last_used: 0,
                is_active: true,
            });

            DeviceStorage::store_devices(&env, &addr, &devices);

            let mut manager = DeviceManager::load(addr.clone());

            // revoke_device should succeed and target the active entry
            manager.revoke_device(&env, &key_id).unwrap();

            // Active count should now be 0
            assert_eq!(manager.active_device_count(&env), 0);

            // ActiveDevicePolicy should reject the key
            let device_policy = ActiveDevicePolicy;
            assert_eq!(
                device_policy.evaluate(
                    &env,
                    &DeviceContext {
                        account: &addr,
                        key_id: &key_id,
                    },
                ),
                Err(AccountError::DeviceNotFound)
            );
        });
    }
}
