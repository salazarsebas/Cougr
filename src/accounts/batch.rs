use alloc::vec::Vec;
use soroban_sdk::Env;

use super::error::AccountError;
use super::traits::CougrAccount;
use super::types::GameAction;

/// Builder for composing multiple game actions into one atomic batch.
///
/// Actions are authorized once (via the account's `authorize` method)
/// and then returned for execution by the caller.
///
/// # Example
/// ```no_run
/// # use cougr_core::accounts::{BatchBuilder, CougrAccount, GameAction};
/// # use soroban_sdk::{symbol_short, Bytes, Env};
/// # fn run<A: CougrAccount>(env: &Env, account: &A) -> Result<(), cougr_core::accounts::AccountError> {
/// let mut batch = BatchBuilder::new();
/// batch.add(GameAction { system_name: symbol_short!("move"), data: Bytes::new(env) });
/// batch.add(GameAction { system_name: symbol_short!("attack"), data: Bytes::new(env) });
///
/// let executed = batch.execute(env, account)?;
/// assert_eq!(executed.len(), 2);
/// # Ok(())
/// # }
/// ```
pub struct BatchBuilder {
    actions: Vec<GameAction>,
}

impl BatchBuilder {
    /// Create an empty batch builder.
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    /// Add a game action to the batch.
    pub fn add(&mut self, action: GameAction) -> &mut Self {
        self.actions.push(action);
        self
    }

    /// Returns the number of actions in the batch.
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    /// Returns whether the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    /// Authorize and execute all actions in the batch atomically.
    ///
    /// Each action is authorized via the account's `authorize` method.
    /// If any authorization fails, the entire batch is rejected.
    ///
    /// Returns the list of authorized actions for the caller to apply.
    pub fn execute<A: CougrAccount>(
        self,
        env: &Env,
        account: &A,
    ) -> Result<Vec<GameAction>, AccountError> {
        if self.actions.is_empty() {
            return Err(AccountError::BatchEmpty);
        }

        // Authorize each action
        for action in &self.actions {
            account.authorize(env, action)?;
        }

        Ok(self.actions)
    }
}

impl Default for BatchBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::testing::MockAccount;
    use soroban_sdk::{symbol_short, Bytes, Env};

    #[test]
    fn test_batch_builder_new() {
        let batch = BatchBuilder::new();
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn test_batch_builder_add() {
        let env = Env::default();
        let mut batch = BatchBuilder::new();
        batch.add(GameAction {
            system_name: symbol_short!("move"),
            data: Bytes::new(&env),
        });
        batch.add(GameAction {
            system_name: symbol_short!("attack"),
            data: Bytes::new(&env),
        });

        assert_eq!(batch.len(), 2);
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_batch_execute_empty_fails() {
        let env = Env::default();
        let account = MockAccount::new(&env);
        let batch = BatchBuilder::new();

        let result = batch.execute(&env, &account);
        assert!(matches!(result, Err(AccountError::BatchEmpty)));
    }

    #[test]
    fn test_batch_execute_success() {
        let env = Env::default();
        let account = MockAccount::new(&env);

        let mut batch = BatchBuilder::new();
        batch.add(GameAction {
            system_name: symbol_short!("move"),
            data: Bytes::new(&env),
        });
        batch.add(GameAction {
            system_name: symbol_short!("attack"),
            data: Bytes::new(&env),
        });

        let result = batch.execute(&env, &account);
        assert!(result.is_ok());
        let actions = result.unwrap();
        assert_eq!(actions.len(), 2);
    }
}
