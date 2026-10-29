use soroban_sdk::{Address, Env};

use super::error::AccountError;
use super::intent::{AuthMethod, AuthResult, SignedIntent};
use super::traits::{CougrAccount, IntentAccount};
use super::types::{AccountCapabilities, GameAction};

/// A mock account for testing that always authorizes actions.
///
/// Capabilities are configurable at construction time.
pub struct MockAccount {
    address: Address,
    capabilities: AccountCapabilities,
}

impl MockAccount {
    /// Create a mock account with default (full) capabilities.
    pub fn new(env: &Env) -> Self {
        use soroban_sdk::testutils::Address as _;
        Self {
            address: Address::generate(env),
            capabilities: AccountCapabilities {
                can_batch: true,
                has_session_keys: true,
                has_social_recovery: true,
                has_passkey_auth: true,
            },
        }
    }

    /// Create a mock account with custom capabilities.
    pub fn with_capabilities(env: &Env, capabilities: AccountCapabilities) -> Self {
        use soroban_sdk::testutils::Address as _;
        Self {
            address: Address::generate(env),
            capabilities,
        }
    }
}

impl CougrAccount for MockAccount {
    fn address(&self) -> &Address {
        &self.address
    }

    fn capabilities(&self) -> AccountCapabilities {
        self.capabilities.clone()
    }

    fn authorize(&self, _env: &Env, _action: &GameAction) -> Result<(), AccountError> {
        // Mock always succeeds
        Ok(())
    }
}

impl IntentAccount for MockAccount {
    fn authorize_intent(
        &mut self,
        _env: &Env,
        intent: &SignedIntent,
    ) -> Result<AuthResult, AccountError> {
        Ok(AuthResult {
            method: match intent.signer.kind {
                super::intent::IntentSigner::Direct => AuthMethod::Direct,
                super::intent::IntentSigner::Session => AuthMethod::Session,
                super::intent::IntentSigner::Passkey => AuthMethod::Passkey,
            },
            nonce_consumed: intent.nonce,
            session_key_id: intent.signer.session_key_id.clone(),
            remaining_operations: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, Bytes, Env};

    #[test]
    fn test_mock_account_creation() {
        let env = Env::default();
        let account = MockAccount::new(&env);
        let caps = account.capabilities();
        assert!(caps.can_batch);
        assert!(caps.has_session_keys);
    }

    #[test]
    fn test_mock_account_custom_capabilities() {
        let env = Env::default();
        let caps = AccountCapabilities {
            can_batch: false,
            has_session_keys: false,
            has_social_recovery: true,
            has_passkey_auth: false,
        };
        let account = MockAccount::with_capabilities(&env, caps);
        let result = account.capabilities();
        assert!(!result.can_batch);
        assert!(!result.has_session_keys);
        assert!(result.has_social_recovery);
    }

    #[test]
    fn test_mock_account_always_authorizes() {
        let env = Env::default();
        let account = MockAccount::new(&env);
        let action = GameAction {
            system_name: symbol_short!("attack"),
            data: Bytes::new(&env),
        };
        assert!(account.authorize(&env, &action).is_ok());
    }
}
