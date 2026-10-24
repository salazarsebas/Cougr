use soroban_sdk::{contracttype, Address, Bytes, BytesN, Symbol, Vec};

/// A game action that can be authorized by an account.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameAction {
    pub system_name: Symbol,
    pub data: Bytes,
}

/// Defines the scope of a session key's permissions.
#[contracttype]
#[derive(Clone, Debug)]
pub struct SessionScope {
    pub allowed_actions: Vec<Symbol>,
    pub max_operations: u32,
    pub expires_at: u64,
}

/// A session key with its scope and usage tracking.
///
/// Every session is bound to a `delegate` chosen by the owner at approval
/// time. Session intents are only accepted when the delegate has authorized
/// the exact intent via Soroban's auth framework.
#[contracttype]
#[derive(Clone, Debug)]
pub struct SessionKey {
    pub key_id: BytesN<32>,
    pub delegate: Address,
    pub scope: SessionScope,
    pub created_at: u64,
    pub operations_used: u32,
    pub next_nonce: u64,
}

/// Capabilities supported by an account.
#[contracttype]
#[derive(Clone, Debug)]
pub struct AccountCapabilities {
    pub can_batch: bool,
    pub has_session_keys: bool,
    pub has_social_recovery: bool,
    pub has_passkey_auth: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, vec, Env};

    #[test]
    fn test_game_action_creation() {
        let env = Env::default();
        let action = GameAction {
            system_name: symbol_short!("move"),
            data: Bytes::new(&env),
        };
        assert_eq!(action.system_name, symbol_short!("move"));
    }

    #[test]
    fn test_session_scope_creation() {
        let env = Env::default();
        let scope = SessionScope {
            allowed_actions: vec![&env, symbol_short!("move"), symbol_short!("attack")],
            max_operations: 100,
            expires_at: 1000,
        };
        assert_eq!(scope.allowed_actions.len(), 2);
        assert_eq!(scope.max_operations, 100);
    }

    #[test]
    fn test_account_capabilities() {
        let caps = AccountCapabilities {
            can_batch: true,
            has_session_keys: false,
            has_social_recovery: false,
            has_passkey_auth: false,
        };
        assert!(caps.can_batch);
        assert!(!caps.has_session_keys);
    }

    #[test]
    fn test_session_key_creation() {
        let env = Env::default();
        let key = SessionKey {
            key_id: BytesN::from_array(&env, &[1u8; 32]),
            delegate: Address::generate(&env),
            scope: SessionScope {
                allowed_actions: vec![&env, symbol_short!("move")],
                max_operations: 50,
                expires_at: 2000,
            },
            created_at: 500,
            operations_used: 0,
            next_nonce: 0,
        };
        assert_eq!(key.operations_used, 0);
        assert_eq!(key.created_at, 500);
    }
}
