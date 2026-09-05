//! Account subsystem integration tests.
//!
//! Tests session key lifecycle, BatchBuilder with MockAccount,
//! SessionStorage persistence, recovery flow, and DeviceManager.

use cougr_core::accounts::{
    AccountError, DeviceManager, DevicePolicy, MultiDeviceProvider, RecoverableAccount,
    RecoveryConfig, RecoveryProvider, SessionKey, SessionScope, SessionStorage,
};
use soroban_sdk::{
    contract, contractimpl, symbol_short, testutils::Address as _, vec, Address, BytesN, Env,
};

// Dummy contract for storage context
#[contract]
pub struct TestContract;

#[contractimpl]
impl TestContract {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_session_key_lifecycle() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let addr = Address::generate(&env);

    env.as_contract(&contract_id, || {
        // 1. Create session key
        let key = SessionKey {
            key_id: BytesN::from_array(&env, &[1u8; 32]),
            scope: SessionScope {
                allowed_actions: vec![&env, symbol_short!("move"), symbol_short!("attack")],
                max_operations: 100,
                expires_at: 99999,
            },
            created_at: 0,
            operations_used: 0,
            next_nonce: 0,
        };

        // 2. Store
        SessionStorage::store(&env, &addr, &key);
        let loaded = SessionStorage::load(&env, &addr, &key.key_id).unwrap();
        assert_eq!(loaded.operations_used, 0);
        assert_eq!(loaded.scope.max_operations, 100);

        // 3. Use (increment)
        SessionStorage::increment_usage(&env, &addr, &key.key_id).unwrap();
        SessionStorage::increment_usage(&env, &addr, &key.key_id).unwrap();
        let used = SessionStorage::load(&env, &addr, &key.key_id).unwrap();
        assert_eq!(used.operations_used, 2);

        // 4. Store a second key
        let key2 = SessionKey {
            key_id: BytesN::from_array(&env, &[2u8; 32]),
            scope: SessionScope {
                allowed_actions: vec![&env, symbol_short!("move")],
                max_operations: 10,
                expires_at: 0, // already expired
            },
            created_at: 0,
            operations_used: 0,
            next_nonce: 0,
        };
        SessionStorage::store(&env, &addr, &key2);

        let all = SessionStorage::load_all(&env, &addr);
        assert_eq!(all.len(), 2);

        // 5. Cleanup expired
        let removed = SessionStorage::cleanup_expired(&env, &addr);
        assert_eq!(removed, 1);

        let remaining = SessionStorage::load_all(&env, &addr);
        assert_eq!(remaining.len(), 1);

        // 6. Remove remaining key
        assert!(SessionStorage::remove(&env, &addr, &key.key_id));
        let empty = SessionStorage::load_all(&env, &addr);
        assert_eq!(empty.len(), 0);
    });
}

#[test]
fn test_session_storage_multiple_accounts() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());

    let addr1 = Address::generate(&env);
    let addr2 = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let key1 = SessionKey {
            key_id: BytesN::from_array(&env, &[1u8; 32]),
            scope: SessionScope {
                allowed_actions: vec![&env, symbol_short!("move")],
                max_operations: 50,
                expires_at: 99999,
            },
            created_at: 0,
            operations_used: 0,
            next_nonce: 0,
        };
        let key2 = SessionKey {
            key_id: BytesN::from_array(&env, &[2u8; 32]),
            scope: SessionScope {
                allowed_actions: vec![&env, symbol_short!("attack")],
                max_operations: 25,
                expires_at: 99999,
            },
            created_at: 0,
            operations_used: 0,
            next_nonce: 0,
        };

        // Store keys for different accounts
        SessionStorage::store(&env, &addr1, &key1);
        SessionStorage::store(&env, &addr2, &key2);

        // Each account has their own keys
        let a1_keys = SessionStorage::load_all(&env, &addr1);
        assert_eq!(a1_keys.len(), 1);
        let a2_keys = SessionStorage::load_all(&env, &addr2);
        assert_eq!(a2_keys.len(), 1);

        // Cross-account lookup fails
        assert!(SessionStorage::load(&env, &addr1, &key2.key_id).is_none());
        assert!(SessionStorage::load(&env, &addr2, &key1.key_id).is_none());
    });
}

#[test]
fn test_recovery_full_flow() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let config = RecoveryConfig {
        threshold: 2,
        timelock_period: 0, // no timelock for test
        max_guardians: 5,
    };
    let owner = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let mut account = RecoverableAccount::new(owner, config, &env);

        // Add guardians
        let g1 = Address::generate(&env);
        let g2 = Address::generate(&env);
        let g3 = Address::generate(&env);

        account.add_guardian(&env, g1.clone()).unwrap();
        account.add_guardian(&env, g2.clone()).unwrap();
        account.add_guardian(&env, g3.clone()).unwrap();
        assert_eq!(account.guardian_count(&env), 3);

        // Initiate recovery
        let new_owner = Address::generate(&env);
        account.initiate_recovery(&env, new_owner.clone()).unwrap();

        // First approval
        account.approve_recovery(&env, &g1).unwrap();

        // Not enough approvals yet
        let result = account.execute_recovery(&env);
        assert!(result.is_err());

        // Second approval (meets threshold)
        account.approve_recovery(&env, &g2).unwrap();

        // Execute recovery
        let recovered_owner = account.execute_recovery(&env).unwrap();
        assert_eq!(recovered_owner, new_owner);
    });
}

#[test]
fn test_recovery_cancel() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let config = RecoveryConfig {
        threshold: 1,
        timelock_period: 0,
        max_guardians: 3,
    };
    let owner = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let mut account = RecoverableAccount::new(owner, config, &env);

        let g1 = Address::generate(&env);
        account.add_guardian(&env, g1.clone()).unwrap();

        let new_owner = Address::generate(&env);
        account.initiate_recovery(&env, new_owner).unwrap();

        // Cancel recovery
        account.cancel_recovery(&env).unwrap();

        // Can't execute cancelled recovery
        let result = account.execute_recovery(&env);
        assert!(result.is_err());
    });
}

#[test]
fn test_device_manager_full_lifecycle() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let addr = Address::generate(&env);
    let policy = DevicePolicy {
        max_devices: 3,
        auto_revoke_after: 0,
    };

    env.as_contract(&contract_id, || {
        let mut manager = DeviceManager::new(addr, policy, &env);

        // Register devices
        let k1 = BytesN::from_array(&env, &[1u8; 32]);
        let k2 = BytesN::from_array(&env, &[2u8; 32]);
        let k3 = BytesN::from_array(&env, &[3u8; 32]);

        manager
            .register_device(&env, k1.clone(), symbol_short!("phone"))
            .unwrap();
        manager
            .register_device(&env, k2.clone(), symbol_short!("laptop"))
            .unwrap();
        manager
            .register_device(&env, k3.clone(), symbol_short!("tablet"))
            .unwrap();

        assert_eq!(manager.active_device_count(&env), 3);

        // At limit - should fail
        let k4 = BytesN::from_array(&env, &[4u8; 32]);
        let result = manager.register_device(&env, k4, symbol_short!("extra"));
        assert_eq!(result, Err(AccountError::DeviceLimitReached));

        // Revoke one
        manager.revoke_device(&env, &k2).unwrap();
        assert_eq!(manager.active_device_count(&env), 2);

        // Now can register a new one
        let k5 = BytesN::from_array(&env, &[5u8; 32]);
        manager
            .register_device(&env, k5, symbol_short!("new"))
            .unwrap();
        assert_eq!(manager.active_device_count(&env), 3);

        // List shows all (including revoked)
        assert_eq!(manager.list_devices(&env).len(), 4);
    });
}

#[test]
fn test_device_manager_update_last_used() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let addr = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let mut manager = DeviceManager::with_defaults(addr, &env);

        let k1 = BytesN::from_array(&env, &[1u8; 32]);
        manager
            .register_device(&env, k1.clone(), symbol_short!("phone"))
            .unwrap();

        // Update last used
        manager.update_last_used(&env, &k1).unwrap();

        // Update non-existent
        let fake = BytesN::from_array(&env, &[99u8; 32]);
        let result = manager.update_last_used(&env, &fake);
        assert_eq!(result, Err(AccountError::DeviceNotFound));
    });
}

#[test]
fn test_device_manager_policy_change() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let addr = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let mut manager = DeviceManager::with_defaults(addr, &env);
        assert_eq!(manager.policy(&env).max_devices, 5);

        let new_policy = DevicePolicy {
            max_devices: 2,
            auto_revoke_after: 1000,
        };
        manager.set_policy(&env, new_policy);
        assert_eq!(manager.policy(&env).max_devices, 2);
        assert_eq!(manager.policy(&env).auto_revoke_after, 1000);
    });
}
