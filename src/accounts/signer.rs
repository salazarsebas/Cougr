use soroban_sdk::{Address, Env};

use super::error::AccountError;
use super::intent::{IntentProofKind, IntentSigner, SignedIntent};
use super::secp256r1_auth::{verify_secp256r1, Secp256r1Storage};

/// Base signer verification interface for the account kernel.
pub trait AccountSigner {
    fn verify(
        &self,
        env: &Env,
        account: &Address,
        intent: &SignedIntent,
    ) -> Result<(), AccountError>;
}

/// Direct owner signer backed by Soroban `require_auth`.
pub struct DirectAuthSigner;

impl AccountSigner for DirectAuthSigner {
    fn verify(
        &self,
        _env: &Env,
        account: &Address,
        intent: &SignedIntent,
    ) -> Result<(), AccountError> {
        if intent.signer.kind != IntentSigner::Direct {
            return Err(AccountError::SignerMismatch);
        }
        account.require_auth();
        Ok(())
    }
}

/// Session signer used for explicit session intents.
///
/// The session's delegate (stored on the `SessionKey`, set at approval time)
/// must authorize the exact intent fields before any session budget or nonce
/// is consumed. Owner authorization alone is not sufficient.
pub struct SessionAuthSigner {
    delegate: Address,
}

impl SessionAuthSigner {
    /// Build a session signer bound to the delegate loaded for this session.
    pub fn new(delegate: Address) -> Self {
        Self { delegate }
    }

    /// The delegate this signer will require authorization from.
    pub fn delegate(&self) -> &Address {
        &self.delegate
    }
}

impl AccountSigner for SessionAuthSigner {
    fn verify(
        &self,
        env: &Env,
        account: &Address,
        intent: &SignedIntent,
    ) -> Result<(), AccountError> {
        if intent.signer.kind != IntentSigner::Session {
            return Err(AccountError::SignerMismatch);
        }
        // Bind the delegate's signature to the exact intent: account, session
        // key id, nonce and action hash. The action hash carries the domain
        // separation provided by `hash_intent`.
        self.delegate.require_auth_for_args(
            (
                account.clone(),
                intent.signer.label.clone(),
                intent.nonce,
                intent.action_hash.clone(),
            )
                .into_val(env),
        );
        Ok(())
    }
}

/// Passkey signer backed by stored secp256r1 keys.
pub struct Secp256r1PasskeySigner;

impl AccountSigner for Secp256r1PasskeySigner {
    fn verify(
        &self,
        env: &Env,
        account: &Address,
        intent: &SignedIntent,
    ) -> Result<(), AccountError> {
        if intent.signer.kind != IntentSigner::Passkey {
            return Err(AccountError::SignerMismatch);
        }
        if intent.proof.kind != IntentProofKind::Secp256r1 {
            return Err(AccountError::InvalidSignature);
        }
        let key = Secp256r1Storage::find_by_label(env, account, &intent.signer.label)
            .ok_or(AccountError::SignerNotRegistered)?;
        let message = intent.action_hash.to_bytes();
        verify_secp256r1(env, &key.public_key, &message, &intent.proof.signature)
    }
}
