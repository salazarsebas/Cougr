//! Fluent builder pattern for creating session keys with scoped permissions.
//!
//! # Example
//! ```no_run
//! use cougr_core::accounts::SessionBuilder;
//! use soroban_sdk::{symbol_short, Env};
//!
//! let env = Env::default();
//! let scope = SessionBuilder::new(&env)
//!     .allow_action(symbol_short!("move"))
//!     .allow_action(symbol_short!("attack"))
//!     .max_operations(100_u32)
//!     .expires_at(3_600_u64)
//!     .build_scope();
//! assert_eq!(scope.allowed_actions.len(), 2);
//! ```

use soroban_sdk::{Env, Symbol, Vec};

use super::error::AccountError;
use super::traits::SessionKeyProvider;
use super::types::{SessionKey, SessionScope};

/// Builder for creating sessions with scoped permissions.
pub struct SessionBuilder<'a> {
    env: &'a Env,
    allowed_actions: Vec<Symbol>,
    max_operations: u32,
    expires_at: u64,
}

impl<'a> SessionBuilder<'a> {
    /// Create a new session builder.
    pub fn new(env: &'a Env) -> Self {
        Self {
            env,
            allowed_actions: Vec::new(env),
            max_operations: 0,
            expires_at: 0,
        }
    }

    /// Allow a specific game action (e.g., `symbol_short!("move")`).
    pub fn allow_action(mut self, action: Symbol) -> Self {
        self.allowed_actions.push_back(action);
        self
    }

    /// Set maximum number of operations for this session.
    pub fn max_operations(mut self, count: u32) -> Self {
        self.max_operations = count;
        self
    }

    /// Set expiration timestamp (ledger timestamp).
    pub fn expires_at(mut self, timestamp: u64) -> Self {
        self.expires_at = timestamp;
        self
    }

    /// Set expiration relative to the current ledger timestamp.
    pub fn expires_in(mut self, seconds: u64) -> Self {
        self.expires_at = self.env.ledger().timestamp().saturating_add(seconds);
        self
    }

    /// Build the SessionScope.
    pub fn build_scope(self) -> SessionScope {
        SessionScope {
            allowed_actions: self.allowed_actions,
            max_operations: self.max_operations,
            expires_at: self.expires_at,
        }
    }

    /// Build and create the session key on a provider.
    pub fn create<P: SessionKeyProvider>(
        self,
        provider: &mut P,
    ) -> Result<SessionKey, AccountError> {
        let env = self.env;
        let scope = self.build_scope();
        provider.create_session(env, scope)
    }
}

/// Deterministic session key id for a scope.
///
/// The layout is big-endian and exactly 32 bytes:
/// `timestamp (u64) | sequence (u32) | existing_sessions (u32)` followed by
/// `allowed_actions.len() (u32) | max_operations (u32) | expires_at (u64)`.
///
/// This is the single source of truth for the id a contract account creates and
/// for the committed cross-language vectors in
/// `packages/sdk-session/vectors/session-vectors.json`.
pub fn derive_session_key_id(
    timestamp: u64,
    sequence: u32,
    existing_sessions: u32,
    scope: &SessionScope,
) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[0..8].copy_from_slice(&timestamp.to_be_bytes());
    bytes[8..12].copy_from_slice(&sequence.to_be_bytes());
    bytes[12..16].copy_from_slice(&existing_sessions.to_be_bytes());
    bytes[16..20].copy_from_slice(&(scope.allowed_actions.len()).to_be_bytes());
    bytes[20..24].copy_from_slice(&scope.max_operations.to_be_bytes());
    bytes[24..32].copy_from_slice(&scope.expires_at.to_be_bytes());
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, testutils::Ledger as _, Env};

    #[test]
    fn test_session_builder_new() {
        let env = Env::default();
        let builder = SessionBuilder::new(&env);
        let scope = builder.build_scope();
        assert_eq!(scope.allowed_actions.len(), 0);
        assert_eq!(scope.max_operations, 0);
        assert_eq!(scope.expires_at, 0);
    }

    #[test]
    fn test_session_builder_allow_action() {
        let env = Env::default();
        let scope = SessionBuilder::new(&env)
            .allow_action(symbol_short!("move"))
            .allow_action(symbol_short!("attack"))
            .build_scope();
        assert_eq!(scope.allowed_actions.len(), 2);
    }

    #[test]
    fn test_session_builder_max_operations() {
        let env = Env::default();
        let scope = SessionBuilder::new(&env).max_operations(100).build_scope();
        assert_eq!(scope.max_operations, 100);
    }

    #[test]
    fn test_session_builder_expires_at() {
        let env = Env::default();
        let scope = SessionBuilder::new(&env).expires_at(5000).build_scope();
        assert_eq!(scope.expires_at, 5000);
    }

    #[test]
    fn test_session_builder_expires_in() {
        let env = Env::default();
        env.ledger().with_mut(|li| {
            li.timestamp = 2_000;
        });
        let scope = SessionBuilder::new(&env).expires_in(800).build_scope();
        assert_eq!(scope.expires_at, 2_800);
    }

    #[test]
    fn test_session_builder_full_chain() {
        let env = Env::default();
        let scope = SessionBuilder::new(&env)
            .allow_action(symbol_short!("move"))
            .allow_action(symbol_short!("attack"))
            .allow_action(symbol_short!("trade"))
            .max_operations(200)
            .expires_at(10000)
            .build_scope();

        assert_eq!(scope.allowed_actions.len(), 3);
        assert_eq!(scope.max_operations, 200);
        assert_eq!(scope.expires_at, 10000);
        assert_eq!(scope.allowed_actions.get(0).unwrap(), symbol_short!("move"));
    }
}
