use soroban_sdk::{Address, Env};

use super::error::AccountError;
use super::intent::{AuthResult, SignedIntent};
use super::kernel::AccountKernel;
use super::traits::{CougrAccount, IntentAccount};
use super::types::{AccountCapabilities, GameAction};

/// A Classic Stellar account (G-address).
///
/// Wraps a standard Stellar address and provides basic authorization
/// via `require_auth()`. Does not support session keys or social recovery.
pub struct ClassicAccount {
    address: Address,
}

impl ClassicAccount {
    /// Create a new Classic account wrapper.
    pub fn new(address: Address) -> Self {
        Self { address }
    }
}

impl CougrAccount for ClassicAccount {
    fn address(&self) -> &Address {
        &self.address
    }

    fn capabilities(&self) -> AccountCapabilities {
        AccountCapabilities {
            can_batch: false,
            has_session_keys: false,
            has_social_recovery: false,
            has_passkey_auth: false,
        }
    }

    fn authorize(&self, _env: &Env, _action: &GameAction) -> Result<(), AccountError> {
        self.address.require_auth();
        Ok(())
    }
}

impl IntentAccount for ClassicAccount {
    fn authorize_intent(
        &mut self,
        env: &Env,
        intent: &SignedIntent,
    ) -> Result<AuthResult, AccountError> {
        AccountKernel::new(self.address.clone()).authorize(env, intent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_classic_account_creation() {
        let env = Env::default();
        let addr = Address::generate(&env);
        let account = ClassicAccount::new(addr.clone());
        assert_eq!(*account.address(), addr);
    }

    #[test]
    fn test_classic_account_capabilities() {
        let env = Env::default();
        let addr = Address::generate(&env);
        let account = ClassicAccount::new(addr);
        let caps = account.capabilities();
        assert!(!caps.can_batch);
        assert!(!caps.has_session_keys);
        assert!(!caps.has_social_recovery);
    }
}
