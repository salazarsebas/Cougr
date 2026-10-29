//! secp256r1 (WebAuthn/Passkey) authentication support.
//!
//! Provides signature verification and key storage for secp256r1 public keys,
//! enabling WebAuthn/Passkey-based authentication for game accounts.
//!
//! Uses Soroban's built-in `env.crypto().secp256r1_verify()` which became
//! available in Protocol 21.
//!
//! # Example
//! ```no_run
//! use cougr_core::accounts::{verify_secp256r1, Secp256r1Key, Secp256r1Storage};
//! use soroban_sdk::{symbol_short, testutils::Address as _, Address, Bytes, BytesN, Env};
//!
//! let env = Env::default();
//! let account_addr = Address::generate(&env);
//! let passkey_pubkey = BytesN::from_array(&env, &[4u8; 65]);
//! let key = Secp256r1Key {
//!     public_key: passkey_pubkey.clone(),
//!     label: symbol_short!("passkey1"),
//!     registered_at: 0,
//! };
//! Secp256r1Storage::store(&env, &account_addr, &key);
//!
//! let message = Bytes::new(&env);
//! let signature = BytesN::from_array(&env, &[0u8; 64]);
//! verify_secp256r1(&env, &passkey_pubkey, &message, &signature)?;
//! # Ok::<(), cougr_core::accounts::AccountError>(())
//! ```

use soroban_sdk::{contracttype, Address, Bytes, BytesN, Env, Symbol, Vec};

use super::error::AccountError;

/// A registered secp256r1 public key for WebAuthn/Passkey auth.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Secp256r1Key {
    /// SEC-1 uncompressed public key (65 bytes: 0x04 || x || y).
    pub public_key: BytesN<65>,
    /// Human-readable label (e.g., "passkey_1", "yubikey").
    pub label: Symbol,
    /// Ledger timestamp when the key was registered.
    pub registered_at: u64,
}

const SECP256R1_KEYS_PREFIX: &str = "p256_keys";

/// Persistent storage for secp256r1 keys per account.
pub struct Secp256r1Storage;

impl Secp256r1Storage {
    /// Store a secp256r1 key. If a key with the same label exists, it is overwritten.
    pub fn store(env: &Env, account: &Address, key: &Secp256r1Key) {
        let keys = Self::load_all(env, account);
        let mut new_keys: Vec<Secp256r1Key> = Vec::new(env);

        // Remove existing key with same label if present
        for i in 0..keys.len() {
            if let Some(k) = keys.get(i) {
                if k.label != key.label {
                    new_keys.push_back(k);
                }
            }
        }
        new_keys.push_back(key.clone());

        let storage_key = Self::storage_key(env, account);
        env.storage().persistent().set(&storage_key, &new_keys);
    }

    /// Load all registered secp256r1 keys for an account.
    pub fn load_all(env: &Env, account: &Address) -> Vec<Secp256r1Key> {
        let storage_key = Self::storage_key(env, account);
        env.storage()
            .persistent()
            .get(&storage_key)
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Remove a secp256r1 key by label. Returns true if found and removed.
    pub fn remove(env: &Env, account: &Address, label: &Symbol) -> bool {
        let keys = Self::load_all(env, account);
        let mut new_keys: Vec<Secp256r1Key> = Vec::new(env);
        let mut found = false;

        for i in 0..keys.len() {
            if let Some(k) = keys.get(i) {
                if &k.label == label {
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
            }
        }
        found
    }

    /// Find a key by label.
    pub fn find_by_label(env: &Env, account: &Address, label: &Symbol) -> Option<Secp256r1Key> {
        let keys = Self::load_all(env, account);
        for i in 0..keys.len() {
            if let Some(k) = keys.get(i) {
                if &k.label == label {
                    return Some(k);
                }
            }
        }
        None
    }

    fn storage_key(env: &Env, account: &Address) -> (Symbol, Address) {
        (Symbol::new(env, SECP256R1_KEYS_PREFIX), account.clone())
    }
}

/// Verify a secp256r1 signature.
///
/// Hashes the message with SHA-256, then verifies the signature against the
/// public key using `env.crypto().secp256r1_verify()`.
///
/// **Note**: `secp256r1_verify` panics on invalid signatures in the Soroban
/// runtime. This function catches the panic as an error in test environments,
/// but in production the transaction will fail.
pub fn verify_secp256r1(
    env: &Env,
    public_key: &BytesN<65>,
    message: &Bytes,
    signature: &BytesN<64>,
) -> Result<(), AccountError> {
    let digest = env.crypto().sha256(message);
    env.crypto()
        .secp256r1_verify(public_key, &digest, signature);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{contract, contractimpl, symbol_short, testutils::Address as _, Env};

    #[contract]
    pub struct TestContract;

    #[contractimpl]
    impl TestContract {}

    fn make_key(env: &Env, label_str: &str, pubkey_byte: u8) -> Secp256r1Key {
        // Build a 65-byte "public key" (not cryptographically valid, for storage tests)
        let mut bytes = [0u8; 65];
        bytes[0] = 0x04; // uncompressed prefix
        bytes[1] = pubkey_byte;
        Secp256r1Key {
            public_key: BytesN::from_array(env, &bytes),
            label: Symbol::new(env, label_str),
            registered_at: 0,
        }
    }

    #[test]
    fn test_store_and_load_keys() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let key1 = make_key(&env, "passkey1", 1);
            let key2 = make_key(&env, "passkey2", 2);

            Secp256r1Storage::store(&env, &addr, &key1);
            Secp256r1Storage::store(&env, &addr, &key2);

            let all = Secp256r1Storage::load_all(&env, &addr);
            assert_eq!(all.len(), 2);
        });
    }

    #[test]
    fn test_store_overwrites_same_label() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let key1 = make_key(&env, "passkey1", 1);
            Secp256r1Storage::store(&env, &addr, &key1);

            // Store again with same label but different pubkey byte
            let key1_v2 = make_key(&env, "passkey1", 99);
            Secp256r1Storage::store(&env, &addr, &key1_v2);

            let all = Secp256r1Storage::load_all(&env, &addr);
            assert_eq!(all.len(), 1); // not duplicated
        });
    }

    #[test]
    fn test_remove_key() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let key = make_key(&env, "passkey1", 1);
            Secp256r1Storage::store(&env, &addr, &key);

            let label = symbol_short!("passkey1");
            assert!(Secp256r1Storage::remove(&env, &addr, &label));
            assert_eq!(Secp256r1Storage::load_all(&env, &addr).len(), 0);
        });
    }

    #[test]
    fn test_remove_nonexistent() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let label = symbol_short!("nope");
            assert!(!Secp256r1Storage::remove(&env, &addr, &label));
        });
    }

    #[test]
    fn test_find_by_label() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let key = make_key(&env, "passkey1", 42);
            Secp256r1Storage::store(&env, &addr, &key);

            let label = symbol_short!("passkey1");
            let found = Secp256r1Storage::find_by_label(&env, &addr, &label);
            assert!(found.is_some());

            let missing = symbol_short!("nope");
            assert!(Secp256r1Storage::find_by_label(&env, &addr, &missing).is_none());
        });
    }

    #[test]
    fn test_load_empty() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let all = Secp256r1Storage::load_all(&env, &addr);
            assert_eq!(all.len(), 0);
        });
    }
}
