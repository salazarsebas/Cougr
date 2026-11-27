use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{contracttype, Address, Bytes, BytesN, Env, Symbol};

use super::types::GameAction;

/// Domain separation tag prefixed to every intent hash preimage.
pub const INTENT_HASH_DOMAIN: &[u8] = b"cougr:account:intent:v1";

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
    /// Build a direct-auth intent bound to `account` and `env.current_contract_address()`.
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

    /// Build a session-auth intent bound to `account` and `env.current_contract_address()`.
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

    /// Build a passkey-auth intent bound to `account` and `env.current_contract_address()`.
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

    /// Recompute the deterministic intent hash for `self.account` and
    /// `env.current_contract_address()`.
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

/// Compute the domain-separated SHA-256 hash of an intent.
///
/// The preimage layout is documented in `docs/ACCOUNT_KERNEL.md`:
/// `INTENT_HASH_DOMAIN || account.to_xdr || contract.to_xdr || nonce (BE u64)
/// || expires_at (BE u64) || signer.kind (BE u32) || signer.session_key_id (32B)
/// || signer.label.to_xdr || action.system_name.to_xdr || action.data`.
pub fn hash_intent(
    env: &Env,
    account: &Address,
    contract: &Address,
    signer: &SignerRef,
    action: &GameAction,
    nonce: u64,
    expires_at: u64,
) -> BytesN<32> {
    let mut bytes = Bytes::from_slice(env, INTENT_HASH_DOMAIN);
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
    use soroban_sdk::{contract, contractimpl, symbol_short, testutils::Address as _, Val};

    #[contract]
    pub struct TestContractA;

    #[contractimpl]
    impl TestContractA {}

    #[contract]
    pub struct TestContractB;

    #[contractimpl]
    impl TestContractB {}

    fn address_to_strkey(addr: &Address) -> [u8; 56] {
        let s = addr.to_string();
        let mut buf = [0u8; 56];
        assert_eq!(s.len() as usize, buf.len());
        s.copy_into_slice(&mut buf);
        buf
    }

    fn make_action(env: &Env, name: &str, payload: &[u8]) -> GameAction {
        GameAction {
            system_name: Symbol::new(env, name),
            data: Bytes::from_slice(env, payload),
        }
    }

    #[test]
    fn test_same_intent_on_two_contracts_gives_two_hashes() {
        let env = Env::default();
        let contract_a = env.register(TestContractA, ());
        let contract_b = env.register(TestContractB, ());
        let account = Address::generate(&env);
        let key_id = BytesN::from_array(&env, &[0x11u8; 32]);
        let sig = BytesN::from_array(&env, &[0x22u8; 64]);

        let (direct_a, session_a, passkey_a) = env.as_contract(&contract_a, || {
            let action = make_action(&env, "move", b"xy");
            (
                SignedIntent::direct(&env, account.clone(), action.clone(), 7, 10_000),
                SignedIntent::session(&env, account.clone(), &key_id, action.clone(), 7, 10_000),
                SignedIntent::passkey(
                    &env,
                    account.clone(),
                    symbol_short!("phone"),
                    action,
                    7,
                    10_000,
                    sig.clone(),
                ),
            )
        });

        let (direct_b, session_b, passkey_b) = env.as_contract(&contract_b, || {
            let action = make_action(&env, "move", b"xy");
            (
                SignedIntent::direct(&env, account.clone(), action.clone(), 7, 10_000),
                SignedIntent::session(&env, account.clone(), &key_id, action.clone(), 7, 10_000),
                SignedIntent::passkey(
                    &env,
                    account.clone(),
                    symbol_short!("phone"),
                    action,
                    7,
                    10_000,
                    sig.clone(),
                ),
            )
        });

        assert_ne!(direct_a.action_hash, direct_b.action_hash);
        assert_ne!(session_a.action_hash, session_b.action_hash);
        assert_ne!(passkey_a.action_hash, passkey_b.action_hash);

        env.as_contract(&contract_a, || {
            assert_eq!(direct_a.recompute_hash(&env), direct_a.action_hash);
            assert_ne!(direct_b.recompute_hash(&env), direct_b.action_hash);
        });
        env.as_contract(&contract_b, || {
            assert_eq!(direct_b.recompute_hash(&env), direct_b.action_hash);
            assert_ne!(direct_a.recompute_hash(&env), direct_a.action_hash);
        });
    }

    #[test]
    fn test_same_intent_for_two_accounts_gives_two_hashes() {
        let env = Env::default();
        let contract = env.register(TestContractA, ());
        let account_a = Address::generate(&env);
        let account_b = Address::generate(&env);

        env.as_contract(&contract, || {
            let action = make_action(&env, "attack", b"\x01\x02");
            let intent_a = SignedIntent::direct(&env, account_a, action.clone(), 1, 5000);
            let intent_b = SignedIntent::direct(&env, account_b, action, 1, 5000);
            assert_ne!(intent_a.action_hash, intent_b.action_hash);
        });
    }

    #[test]
    fn test_long_symbol_action_and_label_are_stable_across_two_envs() {
        const LONG_ACTION: &str = "execute_battle_round";
        const OTHER_LONG_ACTION: &str = "execute_battle_turns";
        const LONG_LABEL: &str = "hardware_passkey_key";
        assert_eq!(LONG_ACTION.len(), 20);
        assert_eq!(OTHER_LONG_ACTION.len(), 20);
        assert_eq!(LONG_LABEL.len(), 20);

        let env1 = Env::default();
        let account1 = Address::generate(&env1);
        let contract1 = env1.register(TestContractA, ());
        let account_str = address_to_strkey(&account1);
        let contract_str = address_to_strkey(&contract1);

        let (hash1_session, hash1_passkey, action1_payload, label1_payload) =
            env1.as_contract(&contract1, || {
                let key_id = BytesN::from_array(&env1, &[0x5au8; 32]);
                let sig = BytesN::from_array(&env1, &[0x3cu8; 64]);
                let action = make_action(&env1, LONG_ACTION, b"payload");
                let label = Symbol::new(&env1, LONG_LABEL);
                let action_val: Val = action.system_name.to_val();
                let label_val: Val = label.to_val();

                let session_intent = SignedIntent::session(
                    &env1,
                    account1.clone(),
                    &key_id,
                    action.clone(),
                    42,
                    999,
                );
                let passkey_intent =
                    SignedIntent::passkey(&env1, account1.clone(), label, action, 42, 999, sig);
                (
                    session_intent.action_hash.to_array(),
                    passkey_intent.action_hash.to_array(),
                    action_val.get_payload(),
                    label_val.get_payload(),
                )
            });

        let env2 = Env::default();
        let account2 = Address::from_str(&env2, core::str::from_utf8(&account_str).unwrap());
        let contract2 = Address::from_str(&env2, core::str::from_utf8(&contract_str).unwrap());
        env2.register_at(&contract2, TestContractA, ());

        // Allocate unrelated long symbols in env2 so the host object handles for
        // LONG_ACTION and LONG_LABEL differ from env1.
        let _pad1 = Symbol::new(&env2, "padding_symbol_alpha_1");
        let _pad2 = Symbol::new(&env2, "padding_symbol_beta_2");
        let _pad3 = Symbol::new(&env2, "padding_symbol_gamma_3");

        let (hash2_session, hash2_passkey, hash2_other, action2_payload, label2_payload) = env2
            .as_contract(&contract2, || {
                let key_id = BytesN::from_array(&env2, &[0x5au8; 32]);
                let sig = BytesN::from_array(&env2, &[0x3cu8; 64]);
                let action = make_action(&env2, LONG_ACTION, b"payload");
                let other_action = make_action(&env2, OTHER_LONG_ACTION, b"payload");
                let label = Symbol::new(&env2, LONG_LABEL);
                let action_val: Val = action.system_name.to_val();
                let label_val: Val = label.to_val();

                let session_intent = SignedIntent::session(
                    &env2,
                    account2.clone(),
                    &key_id,
                    action.clone(),
                    42,
                    999,
                );
                let other_intent =
                    SignedIntent::session(&env2, account2.clone(), &key_id, other_action, 42, 999);
                let passkey_intent =
                    SignedIntent::passkey(&env2, account2.clone(), label, action, 42, 999, sig);
                (
                    session_intent.action_hash.to_array(),
                    passkey_intent.action_hash.to_array(),
                    other_intent.action_hash.to_array(),
                    action_val.get_payload(),
                    label_val.get_payload(),
                )
            });

        assert_ne!(
            action1_payload, action2_payload,
            "test must exercise distinct host object handles for long action symbols"
        );
        assert_ne!(
            label1_payload, label2_payload,
            "test must exercise distinct host object handles for long label symbols"
        );
        assert_eq!(hash1_session, hash2_session);
        assert_eq!(hash1_passkey, hash2_passkey);
        assert_ne!(hash2_session, hash2_other);
    }

    #[test]
    fn test_hash_layout_matches_documented_preimage_and_delimiters() {
        let env = Env::default();
        let contract = env.register(TestContractA, ());
        let account = Address::generate(&env);

        // Short symbol created via symbol_short! and Symbol::new must hash identically.
        let signer = SignerRef::direct(&env);
        let short_a = GameAction {
            system_name: symbol_short!("move"),
            data: Bytes::from_slice(&env, b"data"),
        };
        let short_b = make_action(&env, "move", b"data");
        assert_eq!(
            hash_intent(&env, &account, &contract, &signer, &short_a, 3, 100),
            hash_intent(&env, &account, &contract, &signer, &short_b, 3, 100),
        );

        // Self-delimiting XDR symbol encoding prevents label/action boundary ambiguity.
        let signer_ab = SignerRef::passkey(&env, Symbol::new(&env, "ab"));
        let signer_a = SignerRef::passkey(&env, Symbol::new(&env, "a"));
        assert_ne!(
            hash_intent(
                &env,
                &account,
                &contract,
                &signer_ab,
                &make_action(&env, "c", b""),
                0,
                10,
            ),
            hash_intent(
                &env,
                &account,
                &contract,
                &signer_a,
                &make_action(&env, "bc", b""),
                0,
                10,
            ),
        );

        // Verify the exact XDR header bytes and documented field concatenation order.
        let account_xdr = account.clone().to_xdr(&env);
        let contract_xdr = contract.clone().to_xdr(&env);
        assert_eq!(
            [
                account_xdr.get(0).unwrap(),
                account_xdr.get(1).unwrap(),
                account_xdr.get(2).unwrap(),
                account_xdr.get(3).unwrap(),
            ],
            [0x00, 0x00, 0x00, 0x12],
            "ScValType::Address discriminant"
        );
        assert_eq!(
            [
                contract_xdr.get(0).unwrap(),
                contract_xdr.get(1).unwrap(),
                contract_xdr.get(2).unwrap(),
                contract_xdr.get(3).unwrap(),
            ],
            [0x00, 0x00, 0x00, 0x12],
            "ScValType::Address discriminant"
        );

        let label = Symbol::new(&env, "key1");
        let action_name = Symbol::new(&env, "execute_battle_round");
        let label_xdr = label.clone().to_xdr(&env);
        let action_xdr = action_name.clone().to_xdr(&env);
        // ScValType::Symbol (0x0000000f) + BE u32 length + ASCII bytes + 0..3 zero pad bytes
        assert_eq!(
            label_xdr.to_alloc_vec(),
            [0x00, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00, 0x04, b'k', b'e', b'y', b'1']
        );
        let mut expected_action_xdr = alloc::vec![0x00, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00, 0x14];
        expected_action_xdr.extend_from_slice(b"execute_battle_round");
        assert_eq!(action_xdr.to_alloc_vec(), expected_action_xdr);

        let signer = SignerRef::passkey(&env, label);
        let action = GameAction {
            system_name: action_name,
            data: Bytes::from_slice(&env, &[0xde, 0xad, 0xbe, 0xef]),
        };

        let mut manual = Bytes::from_slice(&env, b"cougr:account:intent:v1");
        manual.append(&account_xdr);
        manual.append(&contract_xdr);
        manual.append(&Bytes::from_slice(&env, &5u64.to_be_bytes()));
        manual.append(&Bytes::from_slice(&env, &9000u64.to_be_bytes()));
        manual.append(&Bytes::from_slice(&env, &2u32.to_be_bytes()));
        manual.append(&Bytes::from_slice(&env, &[0u8; 32]));
        manual.append(&label_xdr);
        manual.append(&action_xdr);
        manual.append(&Bytes::from_slice(&env, &[0xde, 0xad, 0xbe, 0xef]));

        let expected = env.crypto().sha256(&manual).to_array();
        let actual = hash_intent(&env, &account, &contract, &signer, &action, 5, 9000).to_array();
        assert_eq!(actual, expected);
    }
}
