use soroban_sdk::{Address, BytesN, Env, Symbol, Vec};

use super::error::AccountError;
use super::ttl;
use super::types::SessionKey;

/// Symbol used as the storage key prefix for session keys.
const SESSION_KEYS_PREFIX: &str = "sess_keys";

/// Persistent session key storage using Soroban contract storage.
///
/// Session keys are stored in the contract's persistent storage, keyed
/// by account address. This allows session keys to survive across
/// contract invocations (unlike `ContractAccount`'s in-memory storage).
///
/// # Example
/// ```no_run
/// use cougr_core::accounts::{SessionKey, SessionScope, SessionStorage};
/// use soroban_sdk::{symbol_short, testutils::Address as _, Address, BytesN, Env, Vec};
///
/// let env = Env::default();
/// let player_address = Address::generate(&env);
/// let key_id = BytesN::from_array(&env, &[7u8; 32]);
/// let session_key = SessionKey {
///     key_id: key_id.clone(),
///     scope: SessionScope {
///         allowed_actions: Vec::from_array(&env, [symbol_short!("move")]),
///         max_operations: 10,
///         expires_at: 1_000,
///     },
///     created_at: 0,
///     operations_used: 0,
///     next_nonce: 0,
/// };
///
/// SessionStorage::store(&env, &player_address, &session_key);
/// let loaded = SessionStorage::load(&env, &player_address, &key_id);
/// assert!(loaded.is_some());
/// ```
pub struct SessionStorage;

impl SessionStorage {
    /// Store a session key for an account.
    ///
    /// If a key with the same `key_id` already exists, it is overwritten.
    pub fn store(env: &Env, account: &Address, key: &SessionKey) {
        let keys = Self::load_all(env, account);
        // Remove existing key with same ID if present
        let mut new_keys: Vec<SessionKey> = Vec::new(env);
        for i in 0..keys.len() {
            if let Some(k) = keys.get(i) {
                if k.key_id != key.key_id {
                    new_keys.push_back(k);
                }
            }
        }
        new_keys.push_back(key.clone());
        let storage_key = Self::storage_key(env, account);
        env.storage().persistent().set(&storage_key, &new_keys);
        ttl::extend_persistent(env, &storage_key);
    }

    /// Load a specific session key by ID.
    pub fn load(env: &Env, account: &Address, key_id: &BytesN<32>) -> Option<SessionKey> {
        let keys = Self::load_all(env, account);
        for i in 0..keys.len() {
            if let Some(k) = keys.get(i) {
                if &k.key_id == key_id {
                    return Some(k);
                }
            }
        }
        None
    }

    /// Load all session keys for an account.
    pub fn load_all(env: &Env, account: &Address) -> Vec<SessionKey> {
        let storage_key = Self::storage_key(env, account);
        let existing = env.storage().persistent().get(&storage_key);
        if existing.is_some() {
            ttl::extend_persistent(env, &storage_key);
        }
        existing.unwrap_or_else(|| Vec::new(env))
    }

    /// Remove a session key by ID. Returns true if found and removed.
    pub fn remove(env: &Env, account: &Address, key_id: &BytesN<32>) -> bool {
        let keys = Self::load_all(env, account);
        let mut new_keys: Vec<SessionKey> = Vec::new(env);
        let mut found = false;

        for i in 0..keys.len() {
            if let Some(k) = keys.get(i) {
                if &k.key_id == key_id {
                    found = true;
                } else {
                    new_keys.push_back(k);
                }
            }
        }

        if found {
            let storage_key = Self::storage_key(env, account);
            if new_keys.is_empty() {
                env.storage().persistent().remove(&storage_key);
            } else {
                env.storage().persistent().set(&storage_key, &new_keys);
                ttl::extend_persistent(env, &storage_key);
            }
        }
        found
    }

    /// Increment the usage count of a session key.
    pub fn increment_usage(
        env: &Env,
        account: &Address,
        key_id: &BytesN<32>,
    ) -> Result<(), AccountError> {
        let keys = Self::load_all(env, account);
        let mut new_keys: Vec<SessionKey> = Vec::new(env);
        let mut found = false;

        for i in 0..keys.len() {
            if let Some(mut k) = keys.get(i) {
                if &k.key_id == key_id {
                    found = true;
                    k.operations_used += 1;
                    new_keys.push_back(k);
                } else {
                    new_keys.push_back(k);
                }
            }
        }

        if !found {
            return Err(AccountError::InvalidScope);
        }

        let storage_key = Self::storage_key(env, account);
        env.storage().persistent().set(&storage_key, &new_keys);
        ttl::extend_persistent(env, &storage_key);
        Ok(())
    }

    /// Consume one authorized session action and advance its nonce.
    pub fn consume_authorized_session(
        env: &Env,
        account: &Address,
        key_id: &BytesN<32>,
    ) -> Result<SessionKey, AccountError> {
        let keys = Self::load_all(env, account);
        let mut new_keys: Vec<SessionKey> = Vec::new(env);
        let mut updated: Option<SessionKey> = None;

        for i in 0..keys.len() {
            if let Some(mut k) = keys.get(i) {
                if &k.key_id == key_id {
                    if env.ledger().timestamp() >= k.scope.expires_at {
                        return Err(AccountError::SessionExpired);
                    }
                    if k.operations_used >= k.scope.max_operations {
                        return Err(AccountError::SessionBudgetExceeded);
                    }
                    k.operations_used += 1;
                    k.next_nonce += 1;
                    updated = Some(k.clone());
                }
                new_keys.push_back(k);
            }
        }

        let updated = updated.ok_or(AccountError::SessionRevoked)?;
        let storage_key = Self::storage_key(env, account);
        env.storage().persistent().set(&storage_key, &new_keys);
        ttl::extend_persistent(env, &storage_key);
        Ok(updated)
    }

    /// Extend the expiry timestamp of an existing session key.
    pub fn renew_session(
        env: &Env,
        account: &Address,
        key_id: &BytesN<32>,
        new_expires_at: u64,
    ) -> Result<SessionKey, AccountError> {
        let keys = Self::load_all(env, account);
        let mut new_keys: Vec<SessionKey> = Vec::new(env);
        let mut updated: Option<SessionKey> = None;

        for i in 0..keys.len() {
            if let Some(mut k) = keys.get(i) {
                if &k.key_id == key_id {
                    if new_expires_at <= env.ledger().timestamp() {
                        return Err(AccountError::InvalidScope);
                    }
                    k.scope.expires_at = new_expires_at;
                    updated = Some(k.clone());
                    new_keys.push_back(k);
                } else {
                    new_keys.push_back(k);
                }
            }
        }

        let updated = updated.ok_or(AccountError::SessionRevoked)?;
        let storage_key = Self::storage_key(env, account);
        env.storage().persistent().set(&storage_key, &new_keys);
        ttl::extend_persistent(env, &storage_key);
        Ok(updated)
    }

    /// Remove all expired session keys for an account.
    /// Returns the number of keys removed.
    pub fn cleanup_expired(env: &Env, account: &Address) -> u32 {
        let keys = Self::load_all(env, account);
        let now = env.ledger().timestamp();
        let mut new_keys: Vec<SessionKey> = Vec::new(env);
        let mut removed: u32 = 0;

        for i in 0..keys.len() {
            if let Some(k) = keys.get(i) {
                if now >= k.scope.expires_at {
                    removed += 1;
                } else {
                    new_keys.push_back(k);
                }
            }
        }

        if removed > 0 {
            let storage_key = Self::storage_key(env, account);
            if new_keys.is_empty() {
                env.storage().persistent().remove(&storage_key);
            } else {
                env.storage().persistent().set(&storage_key, &new_keys);
                ttl::extend_persistent(env, &storage_key);
            }
        }

        removed
    }

    /// Build the storage key for an account's session keys.
    fn storage_key(env: &Env, account: &Address) -> (Symbol, Address) {
        (Symbol::new(env, SESSION_KEYS_PREFIX), account.clone())
    }

    pub(crate) fn is_action_allowed(
        scope: &crate::accounts::types::SessionScope,
        action: &Symbol,
    ) -> bool {
        if scope.allowed_actions.is_empty() {
            return true;
        }
        for i in 0..scope.allowed_actions.len() {
            if scope.allowed_actions.get(i).unwrap() == *action {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{contract, contractimpl, symbol_short, testutils::Address as _, vec, Env};

    use crate::accounts::types::SessionScope;

    // Dummy contract to provide a contract context for storage tests
    #[contract]
    pub struct TestContract;

    #[contractimpl]
    impl TestContract {}

    fn make_session_key(env: &Env, id_byte: u8, expires_at: u64) -> SessionKey {
        SessionKey {
            key_id: BytesN::from_array(env, &[id_byte; 32]),
            scope: SessionScope {
                allowed_actions: vec![env, symbol_short!("move")],
                max_operations: 100,
                expires_at,
            },
            created_at: 0,
            operations_used: 0,
            next_nonce: 0,
        }
    }

    #[test]
    fn test_store_and_load() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let key = make_session_key(&env, 1, 99999);

        env.as_contract(&contract_id, || {
            SessionStorage::store(&env, &addr, &key);
            let loaded = SessionStorage::load(&env, &addr, &key.key_id);
            assert!(loaded.is_some());
            assert_eq!(loaded.unwrap().operations_used, 0);
        });
    }

    #[test]
    fn test_load_nonexistent() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let fake_id = BytesN::from_array(&env, &[99u8; 32]);

        env.as_contract(&contract_id, || {
            assert!(SessionStorage::load(&env, &addr, &fake_id).is_none());
        });
    }

    #[test]
    fn test_load_all() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            SessionStorage::store(&env, &addr, &make_session_key(&env, 1, 99999));
            SessionStorage::store(&env, &addr, &make_session_key(&env, 2, 99999));

            let all = SessionStorage::load_all(&env, &addr);
            assert_eq!(all.len(), 2);
        });
    }

    #[test]
    fn test_remove() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let key = make_session_key(&env, 1, 99999);

        env.as_contract(&contract_id, || {
            SessionStorage::store(&env, &addr, &key);
            assert!(SessionStorage::remove(&env, &addr, &key.key_id));
            assert!(SessionStorage::load(&env, &addr, &key.key_id).is_none());
        });
    }

    #[test]
    fn test_remove_nonexistent() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let fake_id = BytesN::from_array(&env, &[99u8; 32]);

        env.as_contract(&contract_id, || {
            assert!(!SessionStorage::remove(&env, &addr, &fake_id));
        });
    }

    #[test]
    fn test_increment_usage() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let key = make_session_key(&env, 1, 99999);

        env.as_contract(&contract_id, || {
            SessionStorage::store(&env, &addr, &key);
            SessionStorage::increment_usage(&env, &addr, &key.key_id).unwrap();

            let loaded = SessionStorage::load(&env, &addr, &key.key_id).unwrap();
            assert_eq!(loaded.operations_used, 1);
        });
    }

    #[test]
    fn test_increment_usage_nonexistent() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let fake_id = BytesN::from_array(&env, &[99u8; 32]);

        env.as_contract(&contract_id, || {
            let result = SessionStorage::increment_usage(&env, &addr, &fake_id);
            assert_eq!(result, Err(AccountError::InvalidScope));
        });
    }

    #[test]
    fn test_cleanup_expired() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            // Key with expires_at = 0 (already expired at ledger timestamp 0)
            SessionStorage::store(&env, &addr, &make_session_key(&env, 1, 0));
            // Key with expires_at = 99999 (not expired)
            SessionStorage::store(&env, &addr, &make_session_key(&env, 2, 99999));

            let removed = SessionStorage::cleanup_expired(&env, &addr);
            assert_eq!(removed, 1);

            let remaining = SessionStorage::load_all(&env, &addr);
            assert_eq!(remaining.len(), 1);
        });
    }

    #[test]
    fn test_renew_session_updates_expiry() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let key = make_session_key(&env, 3, 1_000);
            SessionStorage::store(&env, &addr, &key);
            let renewed = SessionStorage::renew_session(&env, &addr, &key.key_id, 9_000).unwrap();
            assert_eq!(renewed.scope.expires_at, 9_000);
        });
    }

    #[test]
    fn test_store_overwrites_existing() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut key = make_session_key(&env, 1, 99999);
            SessionStorage::store(&env, &addr, &key);

            // Store again with same key_id but different operations
            key.operations_used = 42;
            SessionStorage::store(&env, &addr, &key);

            let all = SessionStorage::load_all(&env, &addr);
            assert_eq!(all.len(), 1); // should not duplicate

            let loaded = SessionStorage::load(&env, &addr, &key.key_id).unwrap();
            assert_eq!(loaded.operations_used, 42);
        });
    }
}
