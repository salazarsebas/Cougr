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
pub struct SessionAuthSigner;

impl AccountSigner for SessionAuthSigner {
    fn verify(
        &self,
        _env: &Env,
        _account: &Address,
        intent: &SignedIntent,
    ) -> Result<(), AccountError> {
        if intent.signer.kind != IntentSigner::Session {
            return Err(AccountError::SignerMismatch);
        }
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
