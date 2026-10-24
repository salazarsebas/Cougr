//! Test helpers for session flows.

use soroban_sdk::{Address, BytesN, Env, Symbol, Vec};

use crate::accounts::{SessionKey, SessionScope};
use crate::session::{SessionManager, SessionStatus};

/// Pre-built session fixture for sandbox and integration tests.
pub struct MockSession {
    pub owner: Address,
    pub delegate: Address,
    pub key: SessionKey,
}

impl MockSession {
    /// Approve a scoped session for `owner` bound to `delegate` (mocks owner auth on the env).
    pub fn approve(
        env: &Env,
        owner: &Address,
        delegate: &Address,
        actions: &[&str],
        max_operations: u32,
        expires_in: u64,
    ) -> Self {
        env.mock_all_auths();
        let mut builder = crate::accounts::SessionBuilder::new(env)
            .max_operations(max_operations)
            .delegate(delegate.clone());
        for action in actions {
            builder = builder.allow_action(Symbol::new(env, action));
        }
        let key = SessionManager::approve_with_builder(env, owner, builder.expires_in(expires_in))
            .expect("mock session approval");
        Self {
            owner: owner.clone(),
            delegate: delegate.clone(),
            key,
        }
    }

    pub fn status(&self, env: &Env) -> SessionStatus {
        SessionManager::status(env, &self.owner, &self.key.key_id).expect("session status")
    }

    pub fn key_id(&self) -> &BytesN<32> {
        &self.key.key_id
    }

    pub fn scope(
        env: &Env,
        actions: &[&str],
        max_operations: u32,
        expires_in: u64,
    ) -> SessionScope {
        let mut allowed = Vec::new(env);
        for action in actions {
            allowed.push_back(Symbol::new(env, action));
        }
        SessionScope {
            allowed_actions: allowed,
            max_operations,
            expires_at: env.ledger().timestamp().saturating_add(expires_in),
        }
    }
}
