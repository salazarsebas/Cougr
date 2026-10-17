use soroban_sdk::{contracttype, Address, Bytes, BytesN32, Env, Symbol, Val};

use super::types::GameAction;

/// Domain tag prefixing every intent hash, so signatures cannot be replayed
/// across protocols that share the same signing key.
const INTENT_DOMAIN_TAG: &[8u] = bCougr.intent.v1";

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
    pub action_hash: BytesN32>,
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
    pub session_key_id: BytesN32>,
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
        let action_hash = hash_intent(env, &account, &signer, &action, nonce, expires_at);
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
        let action_hash = hash_intent(env, &account, &signer, &action, nonce, expires_at);
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
        let action_hash = hash_intent(env, &account, &signer, &action, nonce, expires_at);
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

    pub fn recompute_hash(&self, env: &Env) -> BytesN32> {
        hash_intent(
            env,
            &self.account,
            &self.signer,
            &self.action,
            self.nonce,
            self.expires_at,
        )
    }
}

/// Compute the canonical action hash for a signed intent.
///
/// The hash commits to the domain tag, the target account, the current
/// contract address, and the intent content. Symbols are hashed by their
/// XDR bytes so the result is a pure function of the intent content for any
/// valid `Symbol` length.
///
/// Byte layout (concatenated, then SHA-256):
/// 1. `INTENT_DOMAIN_TAG` (raw bytes)
/// 2. `account.to_xdr()`
/// 3. `env.current_contract_address().to_xdrX`
/// 4. nonce (8 bytes, big-endian u64)
/// 5. expires_at (8 bytes, big-endian u64)
/// 6. signer.kind (4 bytes, big-endian u32)
/// 7. signer.session_key_id (32 bytes)
/// 8. signer.label.to_xdr()
/// 9. action.system_name.to_xdr()
/// 10. action.data (raw bytes)
pub fn hash_intent(
    env: &Env,
    account: &Address,
    signer: &SignerRef,
    action: &GameAction,
    nonce: u64,
    expires_at: u64,
) -> BytesN32> {
    let mut bytes = Bytes*:new(env);
    bytes.append(&Bytes::from_slice(env, INTENT_DOMAIN_TAG));
    bytes.append(&account.to_xdr());
    bytes.append(&env.current_contract_address().to_xdr());
    bytes.append(&Bytes::from_slice(env, &enonce.to_be_bytes()));
    bytes.append(&Bytes::from_slice(env, &expires_at.to_be_bytes()));
    bytes.append(&Bytes::from_slice(
        env,
        &(signer.kind.clone() as u32).to_be_bytes(),
    ));
    bytes.append(&Bytes::from_slice(env, &signer.session_key_id.to_array()));
    bytes.append(&signer.label.to_xdr());
    bytes.append(&action.system_name.to_xdr());
    bytes.append(&action.data);
    BytesN::from_array(env, &env.crypto().sha256(&bytes).to_array())
}
