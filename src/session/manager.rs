use soroban_sdk::{Address, BytesN, Env};

use crate::accounts::{
    AccountError, AccountKernel, ContractAccount, GameAction, SessionBuilder, SessionKey,
    SessionKeyProvider, SessionScope, SessionStorage, SignedIntent,
};

use super::SessionStatus;

/// End-to-end session lifecycle facade over the account kernel.
pub struct SessionManager;

impl SessionManager {
    /// Create a scoped session key for `owner`, bound to `delegate`.
    ///
    /// Contract entrypoints must call `owner.require_auth()` before invoking this.
    /// The delegate is the only address that may authorize intents for this session.
    pub fn approve(
        env: &Env,
        owner: &Address,
        delegate: &Address,
        scope: SessionScope,
    ) -> Result<SessionKey, AccountError> {
        let mut account = ContractAccount::new(owner.clone());
        account.create_session(env, delegate.clone(), scope)
    }

    /// Create a session from a [`SessionBuilder`] chain.
    ///
    /// Contract entrypoints must call `owner.require_auth()` before invoking this.
    /// The builder must have a delegate set, otherwise creation fails.
    pub fn approve_with_builder(
        _env: &Env,
        owner: &Address,
        builder: SessionBuilder<'_>,
    ) -> Result<SessionKey, AccountError> {
        let mut account = ContractAccount::new(owner.clone());
        builder.create(&mut account)
    }

    /// Authorize a fully-formed signed intent through the kernel.
    pub fn execute_intent(
        env: &Env,
        intent: &SignedIntent,
    ) -> Result<crate::accounts::AuthResult, AccountError> {
        AccountKernel::new(intent.account.clone()).authorize(env, intent)
    }

    /// Build and authorize a session-scoped game action.
    ///
    /// The session's delegate must authorize the intent; the owner's auth alone is insufficient.
    pub fn execute_action(
        env: &Env,
        owner: &Address,
        session: &SessionKey,
        action: GameAction,
        intent_expires_at: u64,
    ) -> Result<crate::accounts::AuthResult, AccountError> {
        let intent = SignedIntent::session(
            env,
            owner.clone(),
            &session.key_id,
            action,
            session.next_nonce,
            intent_expires_at,
        );
        AccountKernel::new(owner.clone()).authorize_session(env, &intent)
    }

    /// Read session health for UI renewal prompts.
    pub fn status(
        env: &Env,
        owner: &Address,
        key_id: &BytesN<32>,
    ) -> Result<SessionStatus, AccountError> {
        SessionStatus::read(env, owner, key_id)
    }

    /// Extend a session expiry.
    ///
    /// Contract entrypoints must call `owner.require_auth()` before invoking this.
    pub fn renew(
        env: &Env,
        owner: &Address,
        key_id: &BytesN<32>,
        new_expires_at: u64,
    ) -> Result<SessionKey, AccountError> {
        let mut account = ContractAccount::new(owner.clone());
        account.renew_session(env, key_id, new_expires_at)
    }

    /// Revoke a session key.
    ///
    /// Contract entrypoints must call `owner.require_auth()` before invoking this.
    pub fn revoke(env: &Env, owner: &Address, key_id: &BytesN<32>) -> Result<(), AccountError> {
        let mut account = ContractAccount::new(owner.clone());
        account.revoke_session(env, key_id)
    }

    /// Try session authorization first, then fall back to a direct owner intent.
    ///
    /// Session authorization requires the bound delegate's signature. An auth failure
    /// on the session path is not a documented fallback condition and is returned as-is.
    pub fn fallback_execute(
        env: &Env,
        session_intent: &SignedIntent,
        direct_intent: &SignedIntent,
    ) -> Result<crate::accounts::AuthResult, AccountError> {
        let kernel = AccountKernel::new(session_intent.account.clone());
        match kernel.authorize_session(env, session_intent) {
            Ok(result) => Ok(result),
            Err(
                AccountError::SessionExpired
                | AccountError::SessionBudgetExceeded
                | AccountError::SessionRevoked
                | AccountError::NonceMismatch
                | AccountError::ActionNotAllowed,
            ) => kernel.authorize_direct(env, direct_intent),
            Err(err) => Err(err),
        }
    }

    /// Remove expired keys for `owner` and return how many were deleted.
    pub fn cleanup_expired(env: &Env, owner: &Address) -> u32 {
        SessionStorage::cleanup_expired(env, owner)
    }
}
