//! Graceful degradation utilities for account operations.
//!
//! These functions attempt advanced features (session keys, batching) and
//! automatically fall back to simpler alternatives when the account doesn't
//! support them.

use soroban_sdk::{Env, Symbol};

use super::batch::BatchBuilder;
use super::error::AccountError;
use super::traits::CougrAccount;
use super::types::{AccountCapabilities, GameAction, SessionKey, SessionScope};

/// Execute an action using a session key if available and valid,
/// falling back to direct authorization.
///
/// # Deprecated
///
/// This helper only checks session fields locally (expiry, operation count,
/// allowed actions). It never verifies that the session exists in storage and
/// never verifies the session's delegate, so it must not be used to authorize
/// session-key actions. Use `SessionManager::fallback_execute` instead, which
/// loads the stored session and requires the delegate's authorization before
/// consuming any budget.
///
/// Direct authorization via `account.authorize` is still performed when the
/// session path does not apply, so callers without sessions are unaffected.
#[deprecated(
    note = "session fields are not verified against storage or the delegate; use SessionManager::fallback_execute"
)]
pub fn authorize_with_fallback<A: CougrAccount>(
    env: &Env,
    account: &A,
    action: &GameAction,
    session: Option<&SessionKey>,
) -> Result<(), AccountError> {
    if let Some(key) = session {
        if account.capabilities().has_session_keys {
            let now = env.ledger().timestamp();
            if now < key.scope.expires_at
                && key.operations_used < key.scope.max_operations
                && is_action_allowed(&key.scope, &action.system_name)
            {
                // Unverified session: refuse to treat it as authorization.
                return Err(AccountError::Unauthorized);
            }
        }
    }
    // Fallback: direct authorization
    account.authorize(env, action)
}

/// Batch-execute actions if the account supports batching,
/// otherwise authorize each action sequentially.
pub fn batch_or_sequential<A: CougrAccount>(
    env: &Env,
    account: &A,
    actions: &[GameAction],
) -> Result<(), AccountError> {
    if actions.is_empty() {
        return Err(AccountError::BatchEmpty);
    }

    if account.capabilities().can_batch {
        let mut batch = BatchBuilder::new();
        for action in actions {
            batch.add(action.clone());
        }
        batch.execute(env, account)?;
    } else {
        for action in actions {
            account.authorize(env, action)?;
        }
    }
    Ok(())
}

/// Check if a capability is available, returning a descriptive error if not.
///
/// `capability` must be one of: `"session_keys"`, `"batch"`, `"social_recovery"`.
pub fn require_capability(
    capabilities: &AccountCapabilities,
    capability: &str,
) -> Result<(), AccountError> {
    let supported = match capability {
        "session_keys" => capabilities.has_session_keys,
        "batch" => capabilities.can_batch,
        "social_recovery" => capabilities.has_social_recovery,
        "passkey_auth" => capabilities.has_passkey_auth,
        _ => true, // Unknown capabilities are assumed available
    };
    if supported {
        Ok(())
    } else {
        Err(AccountError::CapabilityNotSupported)
    }
}

/// Check if an action is allowed by a session scope.
fn is_action_allowed(scope: &SessionScope, action: &Symbol) -> bool {
    if scope.allowed_actions.is_empty() {
        return true; // Empty allowed list = all actions permitted
    }
    for i in 0..scope.allowed_actions.len() {
        if scope.allowed_actions.get(i).unwrap() == *action {
            return true;
        }
    }
    false
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use crate::accounts::testing::MockAccount;
    use soroban_sdk::testutils::Ledger as _;
    use soroban_sdk::{symbol_short, vec, Bytes, BytesN, Env};

    fn make_session(env: &Env, expires_at: u64, max_ops: u32) -> SessionKey {
        SessionKey {
            key_id: BytesN::from_array(env, &[1u8; 32]),
            scope: SessionScope {
                allowed_actions: vec![env, symbol_short!("move"), symbol_short!("attack")],
                max_operations: max_ops,
                expires_at,
            },
            created_at: 0,
            operations_used: 0,
            next_nonce: 0,
        }
    }

    fn make_action(env: &Env, name: &str) -> GameAction {
        GameAction {
            system_name: Symbol::new(env, name),
            data: Bytes::new(env),
        }
    }

    #[test]
    fn test_authorize_with_session_valid_is_refused() {
        let env = Env::default();
        let account = MockAccount::new(&env);
        let session = make_session(&env, 5000, 100);
        let action = make_action(&env, "move");

        // An unverified session must never be treated as authorization.
        let result = authorize_with_fallback(&env, &account, &action, Some(&session));
        assert_eq!(result.unwrap_err(), AccountError::Unauthorized);
    }

    #[test]
    fn test_authorize_with_session_expired_falls_back() {
        let env = Env::default();
        env.ledger().with_mut(|li| {
            li.timestamp = 6000;
        });
        let account = MockAccount::new(&env);
        let session = make_session(&env, 5000, 100); // expired
        let action = make_action(&env, "move");

        // Falls back to direct auth (MockAccount always succeeds)
        let result = authorize_with_fallback(&env, &account, &action, Some(&session));
        assert!(result.is_ok());
    }

    #[test]
    fn test_authorize_with_session_ops_exhausted_falls_back() {
        let env = Env::default();
        let account = MockAccount::new(&env);
        let mut session = make_session(&env, 5000, 10);
        session.operations_used = 10; // exhausted

        let action = make_action(&env, "move");
        let result = authorize_with_fallback(&env, &account, &action, Some(&session));
        assert!(result.is_ok()); // falls back to direct auth
    }

    #[test]
    fn test_authorize_with_session_action_not_allowed_falls_back() {
        let env = Env::default();
        let account = MockAccount::new(&env);
        let session = make_session(&env, 5000, 100);

        let action = make_action(&env, "trade"); // not in allowed actions
        let result = authorize_with_fallback(&env, &account, &action, Some(&session));
        assert!(result.is_ok()); // falls back to direct auth
    }

    #[test]
    fn test_authorize_no_session_falls_back() {
        let env = Env::default();
        let account = MockAccount::new(&env);
        let action = make_action(&env, "move");

        let result = authorize_with_fallback(&env, &account, &action, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_authorize_no_session_keys_capability_falls_back() {
        let env = Env::default();
        let caps = AccountCapabilities {
            can_batch: false,
            has_session_keys: false,
            has_social_recovery: false,
            has_passkey_auth: false,
        };
        let account = MockAccount::with_capabilities(&env, caps);
        let session = make_session(&env, 5000, 100);
        let action = make_action(&env, "move");

        // Has session but account doesn't support them -> falls back
        let result = authorize_with_fallback(&env, &account, &action, Some(&session));
        assert!(result.is_ok());
    }

    #[test]
    fn test_batch_or_sequential_empty_fails() {
        let env = Env::default();
        let account = MockAccount::new(&env);
        let result = batch_or_sequential(&env, &account, &[]);
        assert_eq!(result.unwrap_err(), AccountError::BatchEmpty);
    }

    #[test]
    fn test_batch_or_sequential_with_batch_support() {
        let env = Env::default();
        let account = MockAccount::new(&env); // can_batch = true
        let actions = [make_action(&env, "move"), make_action(&env, "attack")];
        let result = batch_or_sequential(&env, &account, &actions);
        assert!(result.is_ok());
    }

    #[test]
    fn test_batch_or_sequential_without_batch_support() {
        let env = Env::default();
        let caps = AccountCapabilities {
            can_batch: false,
            has_session_keys: false,
            has_social_recovery: false,
            has_passkey_auth: false,
        };
        let account = MockAccount::with_capabilities(&env, caps);
        let actions = [make_action(&env, "move"), make_action(&env, "attack")];
        let result = batch_or_sequential(&env, &account, &actions);
        assert!(result.is_ok());
    }

    #[test]
    fn test_require_capability_session_keys_present() {
        let caps = AccountCapabilities {
            can_batch: true,
            has_session_keys: true,
            has_social_recovery: true,
            has_passkey_auth: true,
        };
        assert!(require_capability(&caps, "session_keys").is_ok());
    }

    #[test]
    fn test_require_capability_session_keys_missing() {
        let caps = AccountCapabilities {
            can_batch: true,
            has_session_keys: false,
            has_social_recovery: false,
            has_passkey_auth: false,
        };
        assert_eq!(
            require_capability(&caps, "session_keys").unwrap_err(),
            AccountError::CapabilityNotSupported
        );
    }

    #[test]
    fn test_require_capability_batch_missing() {
        let caps = AccountCapabilities {
            can_batch: false,
            has_session_keys: true,
            has_social_recovery: false,
            has_passkey_auth: false,
        };
        assert_eq!(
            require_capability(&caps, "batch").unwrap_err(),
            AccountError::CapabilityNotSupported
        );
    }

    #[test]
    fn test_require_capability_unknown_always_ok() {
        let caps = AccountCapabilities {
            can_batch: false,
            has_session_keys: false,
            has_social_recovery: false,
            has_passkey_auth: false,
        };
        assert!(require_capability(&caps, "unknown").is_ok());
    }

    #[test]
    fn test_is_action_allowed_empty_scope() {
        let env = Env::default();
        let scope = SessionScope {
            allowed_actions: vec![&env],
            max_operations: 100,
            expires_at: 5000,
        };
        assert!(is_action_allowed(&scope, &symbol_short!("anything")));
    }

    #[test]
    fn test_is_action_allowed_matches() {
        let env = Env::default();
        let scope = SessionScope {
            allowed_actions: vec![&env, symbol_short!("move")],
            max_operations: 100,
            expires_at: 5000,
        };
        assert!(is_action_allowed(&scope, &symbol_short!("move")));
    }

    #[test]
    fn test_is_action_allowed_not_in_list() {
        let env = Env::default();
        let scope = SessionScope {
            allowed_actions: vec![&env, symbol_short!("move")],
            max_operations: 100,
            expires_at: 5000,
        };
        assert!(!is_action_allowed(&scope, &symbol_short!("trade")));
    }
}
