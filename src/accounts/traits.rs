use soroban_sdk::{Address, BytesN, Env};

use super::error::AccountError;
use super::intent::{AuthResult, SignedIntent};
use super::types::{AccountCapabilities, GameAction, SessionKey, SessionScope};

/// Core account trait for Cougr game accounts.
///
/// All account types (Classic G-address and Contract C-address) implement
/// this trait to provide unified authorization for game actions.
pub trait CougrAccount {
    /// Returns the Stellar address of this account.
    fn address(&self) -> &Address;

    /// Returns the capabilities supported by this account type.
    fn capabilities(&self) -> AccountCapabilities;

    /// Authorize a game action.
    ///
    /// For Classic accounts, this calls `address.require_auth()`.
    /// For Contract accounts, this may use session keys or custom logic.
    fn authorize(&self, env: &Env, action: &GameAction) -> Result<(), AccountError>;
}

/// Explicit intent-based authorization interface used by the account kernel.
pub trait IntentAccount: CougrAccount {
    fn authorize_intent(
        &mut self,
        env: &Env,
        intent: &SignedIntent,
    ) -> Result<AuthResult, AccountError>;
}

/// Session key management for contract accounts.
///
/// This trait provides session key functionality for gasless gameplay.
/// Only contract accounts (C-addresses) support this capability.
pub trait SessionKeyProvider: CougrAccount {
    /// Create a new session key with the given scope.
    fn create_session(
        &mut self,
        env: &Env,
        scope: SessionScope,
    ) -> Result<SessionKey, AccountError>;

    /// Validate that a session key is still active and within scope.
    fn validate_session(&self, env: &Env, key: &SessionKey) -> Result<bool, AccountError>;

    /// Revoke an existing session key.
    fn revoke_session(&mut self, env: &Env, key_id: &BytesN<32>) -> Result<(), AccountError>;

    /// Extend the expiry of an existing session key.
    fn renew_session(
        &mut self,
        env: &Env,
        key_id: &BytesN<32>,
        new_expires_at: u64,
    ) -> Result<SessionKey, AccountError>;
}
