//! Graceful degradation utilities for account operations.
//!
//! These functions attempt advanced features (session keys, batching) and
//! automatically fall back to simpler alternatives when the account doesn't
//! support them.

use soroban_sdk::Env;

use super::batch::BatchBuilder;
use super::error::AccountError;
use super::intent::SignedIntent;
use super::kernel::AccountKernel;
use super::traits::CougrAccount;
use super::types::{AccountCapabilities, GameAction, SessionKey, SessionScope};

/// Execute an action through a valid session, falling back to direct authorization.
///
/// Session intents are checked by `AccountKernel`, which loads the stored key
/// and requires authorization from its delegate before consuming the session.
pub fn authorize_with_fallback<A: CougrAccount>(
    env: &Env,
    account: &A,
    action: &GameAction,
    session: Option<&SessionKey>,
) -> Result<(), AccountError> {
    if let Some(key) = session {
        let now = env.ledger().timestamp();
        if account.capabilities().has_session_keys
            && now < key.scope.expires_at
            && key.operations_used < key.scope.max_operations
            && is_action_allowed(&key.scope, &action.system_name)
        {
            let intent = SignedIntent::session(
                env,
                account.address().clone(),
                &key.key_id,
                action.clone(),
                key.next_nonce,
                key.scope.expires_at,
            );
            match AccountKernel::new(account.address().clone()).authorize_session(env, &intent) {
                Ok(_) => return Ok(()),
                Err(
                    AccountError::SessionExpired
                    | AccountError::SessionBudgetExceeded
                    | AccountError::SessionRevoked
                    | AccountError::NonceMismatch
                    | AccountError::ActionNotAllowed,
                ) => {}
                Err(error) => return Err(error),
            }
        }
    }
    account.authorize(env, action)
}

fn is_action_allowed(scope: &SessionScope, action: &soroban_sdk::Symbol) -> bool {
    if scope.allowed_actions.is_empty() {
        return true;
    }
    for i in 0..scope.allowed_actions.len() {
        if scope.allowed_actions.get(i).unwrap() == *action {
            return true;
        }
    }
    false
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::testing::MockAccount;
    use soroban_sdk::{
        contract, contractimpl, symbol_short, testutils::Address as _, vec, Address, Bytes, BytesN,
        Env, Symbol,
    };

    #[contract]
    pub struct TestContract;

    #[contractimpl]
    impl TestContract {}

    fn make_session(env: &Env, expires_at: u64, max_ops: u32) -> SessionKey {
        SessionKey {
            key_id: BytesN::from_array(env, &[1u8; 32]),
            delegate: Address::generate(env),
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
    fn test_authorize_no_session_falls_back() {
        let env = Env::default();
        let account = MockAccount::new(&env);
        let action = make_action(&env, "move");

        let result = authorize_with_fallback(&env, &account, &action, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_authorize_with_unstored_session_falls_back_directly() {
        let env = Env::default();
        let account = MockAccount::new(&env);
        let session = make_session(&env, 5_000, 10);
        let action = make_action(&env, "move");
        let contract_id = env.register(TestContract, ());

        env.as_contract(&contract_id, || {
            assert!(authorize_with_fallback(&env, &account, &action, Some(&session)).is_ok());
        });
    }

    #[test]
    fn test_is_action_allowed_empty_scope() {
        let env = Env::default();
        let scope = SessionScope {
            allowed_actions: vec![&env],
            max_operations: 100,
            expires_at: 5_000,
        };
        assert!(is_action_allowed(&scope, &symbol_short!("anything")));
    }

    #[test]
    fn test_is_action_allowed_matches_and_rejects_other_actions() {
        let env = Env::default();
        let scope = SessionScope {
            allowed_actions: vec![&env, symbol_short!("move")],
            max_operations: 100,
            expires_at: 5_000,
        };
        assert!(is_action_allowed(&scope, &symbol_short!("move")));
        assert!(!is_action_allowed(&scope, &symbol_short!("trade")));
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
}
