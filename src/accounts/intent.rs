use soroban_sdk::{contracttype, Address, Bytes, BytesN, Env, Symbol, Val};

use super::types::GameAction;

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
        let action_hash = hash_intent(env, &signer, &action, nonce, expires_at);
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
        let action_hash = hash_intent(env, &signer, &action, nonce, expires_at);
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
        let action_hash = hash_intent(env, &signer, &action, nonce, expires_at);
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
        hash_intent(env, &self.signer, &self.action, self.nonce, self.expires_at)
    }
}

pub fn hash_intent(
    env: &Env,
    signer: &SignerRef,
    action: &GameAction,
    nonce: u64,
    expires_at: u64,
) -> BytesN<32> {
    let mut bytes = Bytes::new(env);
    bytes.append(&Bytes::from_slice(env, &nonce.to_be_bytes()));
    bytes.append(&Bytes::from_slice(env, &expires_at.to_be_bytes()));
    bytes.append(&Bytes::from_slice(
        env,
        &(signer.kind.clone() as u32).to_be_bytes(),
    ));
    bytes.append(&Bytes::from_slice(env, &signer.session_key_id.to_array()));
    let label_bits: Val = signer.label.to_val();
    bytes.append(&Bytes::from_slice(
        env,
        &label_bits.get_payload().to_be_bytes(),
    ));
    let action_bits: Val = action.system_name.to_val();
    bytes.append(&Bytes::from_slice(
        env,
        &action_bits.get_payload().to_be_bytes(),
    ));
    bytes.append(&action.data);
    BytesN::from_array(env, &env.crypto().sha256(&bytes).to_array())
}
