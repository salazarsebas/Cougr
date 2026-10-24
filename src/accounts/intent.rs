use soroban_sdk::{contracttype, xdr::ToXdr, Address, Bytes, BytesN, Env, Symbol};

use super::types::GameAction;

/// Domain separation tag prefixed to every hashed account intent payload.
pub const INTENT_DOMAIN_TAG: &[u8] = b"COUGR_ACCOUNT_INTENT_V1";

/// Supported intent signer kinds for the account kernel.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum IntentSigner {
    Direct = 0,
    Session = 1,
    Passkey = 2,
}

/// Stable identifier for the signer used by an intent.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignerRef {
    pub kind: IntentSigner,
    pub session_key_id: BytesN<32>,
    pub label: Symbol,
}

/// Signature container for intent verification.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum IntentProofKind {
    None = 0,
    Secp256r1 = 1,
}

/// Signature bytes for a signed intent.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntentProof {
    pub kind: IntentProofKind,
    pub signature: BytesN<64>,
}

/// Canonical signed intent schema for account authorization.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedIntent {
    pub account: Address,
    pub signer: SignerRef,
    pub action: GameAction,
    pub nonce: u64,
    pub expires_at: u64,
    pub action_hash: BytesN<32>,
    pub proof: IntentProof,
}

/// Result of a successful authorization.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AuthMethod {
    Direct = 0,
    Session = 1,
    Passkey = 2,
}

/// Structured authorization result returned by the kernel.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthResult {
    pub method: AuthMethod,
    pub nonce_consumed: u64,
    pub session_key_id: BytesN<32>,
    pub remaining_operations: u32,
}

impl SignerRef {
    pub fn direct(env: &Env) -> Self {
        Self {
            kind: IntentSigner::Direct,
            session_key_id: BytesN::from_array(env, &[0u8; 32]),
            label: Symbol::new(env, ""),
        }
    }

    pub fn session(env: &Env, key_id: &BytesN<32>) -> Self {
        Self {
            kind: IntentSigner::Session,
            session_key_id: key_id.clone(),
            label: Symbol::new(env, ""),
        }
    }

    pub fn passkey(env: &Env, label: Symbol) -> Self {
        Self {
            kind: IntentSigner::Passkey,
            session_key_id: BytesN::from_array(env, &[0u8; 32]),
            label,
        }
    }
}

impl IntentProof {
    pub fn none(env: &Env) -> Self {
        Self {
            kind: IntentProofKind::None,
            signature: BytesN::from_array(env, &[0u8; 64]),
        }
    }

    pub fn secp256r1(signature: BytesN<64>) -> Self {
        Self {
            kind: IntentProofKind::Secp256r1,
            signature,
        }
    }
}

impl SignedIntent {
    pub fn direct(
        env: &Env,
        account: Address,
        action: GameAction,
        nonce: u64,
        expires_at: u64,
    ) -> Self {
        let signer = SignerRef::direct(env);
        let contract = env.current_contract_address();
        let action_hash = hash_intent(
            env, &account, &contract, &signer, &action, nonce, expires_at,
        );
        Self {
            account,
            signer,
            action,
            nonce,
            expires_at,
            action_hash,
            proof: IntentProof::none(env),
        }
    }

    pub fn session(
        env: &Env,
        account: Address,
        key_id: &BytesN<32>,
        action: GameAction,
        nonce: u64,
        expires_at: u64,
    ) -> Self {
        let signer = SignerRef::session(env, key_id);
        let contract = env.current_contract_address();
        let action_hash = hash_intent(
            env, &account, &contract, &signer, &action, nonce, expires_at,
        );
        Self {
            account,
            signer,
            action,
            nonce,
            expires_at,
            action_hash,
            proof: IntentProof::none(env),
        }
    }

    pub fn passkey(
        env: &Env,
        account: Address,
        label: Symbol,
        action: GameAction,
        nonce: u64,
        expires_at: u64,
        signature: BytesN<64>,
    ) -> Self {
        let signer = SignerRef::passkey(env, label);
        let contract = env.current_contract_address();
        let action_hash = hash_intent(
            env, &account, &contract, &signer, &action, nonce, expires_at,
        );
        Self {
            account,
            signer,
            action,
            nonce,
            expires_at,
            action_hash,
            proof: IntentProof::secp256r1(signature),
        }
    }

    pub fn recompute_hash(&self, env: &Env) -> BytesN<32> {
        let contract = env.current_contract_address();
        hash_intent(
            env,
            &self.account,
            &contract,
            &self.signer,
            &self.action,
            self.nonce,
            self.expires_at,
        )
    }
}

pub fn hash_intent(
    env: &Env,
    account: &Address,
    contract: &Address,
    signer: &SignerRef,
    action: &GameAction,
    nonce: u64,
    expires_at: u64,
) -> BytesN<32> {
    let mut bytes = Bytes::from_slice(env, INTENT_DOMAIN_TAG);
    bytes.append(&account.clone().to_xdr(env));
    bytes.append(&contract.clone().to_xdr(env));
    bytes.append(&Bytes::from_slice(env, &nonce.to_be_bytes()));
    bytes.append(&Bytes::from_slice(env, &expires_at.to_be_bytes()));
    bytes.append(&Bytes::from_slice(
        env,
        &(signer.kind.clone() as u32).to_be_bytes(),
    ));
    bytes.append(&Bytes::from_slice(env, &signer.session_key_id.to_array()));
    bytes.append(&signer.label.clone().to_xdr(env));
    bytes.append(&action.system_name.clone().to_xdr(env));
    bytes.append(&action.data);
    BytesN::from_array(env, &env.crypto().sha256(&bytes).to_array())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::{AccountError, AccountKernel};
    use soroban_sdk::{contract, contractimpl, testutils::Address as _};

    #[contract]
    struct ContractA;

    #[contractimpl]
    impl ContractA {}

    #[contract]
    struct ContractB;

    #[contractimpl]
    impl ContractB {}

    #[test]
    fn same_intent_on_two_contracts_produces_different_hashes() {
        let env = Env::default();
        let contract_a = env.register(ContractA, ());
        let contract_b = env.register(ContractB, ());
        let account = Address::generate(&env);
        let action = GameAction {
            system_name: Symbol::new(&env, "move"),
            data: Bytes::from_slice(&env, &[1, 2, 3]),
        };

        let intent_a = env.as_contract(&contract_a, || {
            SignedIntent::direct(&env, account.clone(), action.clone(), 1, 10_000)
        });
        let intent_b = env.as_contract(&contract_b, || {
            SignedIntent::direct(&env, account.clone(), action, 1, 10_000)
        });

        assert_ne!(intent_a.action_hash, intent_b.action_hash);

        env.as_contract(&contract_b, || {
            let kernel = AccountKernel::new(account);
            assert_eq!(
                kernel.authorize_direct(&env, &intent_a),
                Err(AccountError::InvalidIntent)
            );
        });
    }

    #[test]
    fn same_intent_for_two_accounts_produces_different_hashes() {
        let env = Env::default();
        let contract_id = env.register(ContractA, ());
        let account_a = Address::generate(&env);
        let account_b = Address::generate(&env);
        let action = GameAction {
            system_name: Symbol::new(&env, "move"),
            data: Bytes::from_slice(&env, &[7, 8]),
        };

        env.as_contract(&contract_id, || {
            let intent_a = SignedIntent::direct(&env, account_a, action.clone(), 0, 5_000);
            let intent_b = SignedIntent::direct(&env, account_b, action, 0, 5_000);
            assert_ne!(intent_a.action_hash, intent_b.action_hash);
        });
    }

    #[test]
    fn twenty_char_action_name_hashes_deterministically_across_envs() {
        const ACCOUNT_STRKEY: &str = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";
        const CONTRACT_STRKEY: &str = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4";
        const LONG_ACTION_20: &str = "execute_special_move"; // 20 chars (> 9-char small symbol limit)
        const LONG_LABEL: &str = "primary_device_key";

        let env_1 = Env::default();
        // Populate unrelated long symbols in env_1 so host object handles differ from env_2.
        let _noise_1 = Symbol::new(&env_1, "unrelated_long_symbol_one");
        let _noise_2 = Symbol::new(&env_1, "unrelated_long_symbol_two");
        let contract_1 = Address::from_str(&env_1, CONTRACT_STRKEY);
        env_1.register_at(&contract_1, ContractA, ());
        let account_1 = Address::from_str(&env_1, ACCOUNT_STRKEY);
        let hash_1 = env_1.as_contract(&contract_1, || {
            let action = GameAction {
                system_name: Symbol::new(&env_1, LONG_ACTION_20),
                data: Bytes::from_slice(&env_1, &[0xaa, 0xbb]),
            };
            let intent = SignedIntent::passkey(
                &env_1,
                account_1,
                Symbol::new(&env_1, LONG_LABEL),
                action,
                7,
                99_000,
                BytesN::from_array(&env_1, &[0u8; 64]),
            );
            intent.action_hash.to_array()
        });

        let env_2 = Env::default();
        let contract_2 = Address::from_str(&env_2, CONTRACT_STRKEY);
        env_2.register_at(&contract_2, ContractA, ());
        let account_2 = Address::from_str(&env_2, ACCOUNT_STRKEY);
        let hash_2 = env_2.as_contract(&contract_2, || {
            let action = GameAction {
                system_name: Symbol::new(&env_2, LONG_ACTION_20),
                data: Bytes::from_slice(&env_2, &[0xaa, 0xbb]),
            };
            let intent = SignedIntent::passkey(
                &env_2,
                account_2,
                Symbol::new(&env_2, LONG_LABEL),
                action,
                7,
                99_000,
                BytesN::from_array(&env_2, &[0u8; 64]),
            );
            intent.action_hash.to_array()
        });

        assert_eq!(hash_1, hash_2);
    }
}
