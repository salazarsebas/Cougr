use soroban_sdk::{contracttype, Address, Env, Symbol};

use super::error::StandardsError;

const PAUSED_PREFIX: &str = "std_pause";

/// Pause state primitive for emergency stops.
#[derive(Clone, Debug)]
pub struct Pausable {
    id: Symbol,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PausedEvent {
    pub account: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnpausedEvent {
    pub account: Address,
}

impl Pausable {
    pub fn new(id: Symbol) -> Self {
        Self { id }
    }

    pub fn is_paused(&self, env: &Env) -> bool {
        env.storage()
            .persistent()
            .get(&self.paused_key(env))
            .unwrap_or(false)
    }

    pub fn require_paused(&self, env: &Env) -> Result<(), StandardsError> {
        if self.is_paused(env) {
            return Ok(());
        }
        Err(StandardsError::NotPaused)
    }

    pub fn require_not_paused(&self, env: &Env) -> Result<(), StandardsError> {
        if self.is_paused(env) {
            return Err(StandardsError::Paused);
        }
        Ok(())
    }

    /// Transitions the module into the paused state and emits a [`PausedEvent`].
    ///
    /// Note: `caller` is recorded in the returned event only and is not
    /// authenticated by this helper; the integrating contract must authenticate
    /// and authorize `caller` before invoking this method.
    pub fn pause(&self, env: &Env, caller: &Address) -> Result<PausedEvent, StandardsError> {
        if self.is_paused(env) {
            return Err(StandardsError::Paused);
        }
        env.storage().persistent().set(&self.paused_key(env), &true);
        Ok(PausedEvent {
            account: caller.clone(),
        })
    }

    /// Transitions the module out of the paused state and emits an [`UnpausedEvent`].
    ///
    /// Note: `caller` is recorded in the returned event only and is not
    /// authenticated by this helper; the integrating contract must authenticate
    /// and authorize `caller` before invoking this method.
    pub fn unpause(&self, env: &Env, caller: &Address) -> Result<UnpausedEvent, StandardsError> {
        if !self.is_paused(env) {
            return Err(StandardsError::NotPaused);
        }
        env.storage()
            .persistent()
            .set(&self.paused_key(env), &false);
        Ok(UnpausedEvent {
            account: caller.clone(),
        })
    }

    fn paused_key(&self, env: &Env) -> (Symbol, Symbol) {
        (Symbol::new(env, PAUSED_PREFIX), self.id.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{contract, contractimpl, symbol_short, testutils::Address as _};

    #[contract]
    struct DummyContract;

    #[contractimpl]
    impl DummyContract {}

    #[test]
    fn pausable_guards_and_transitions_cover_paused_and_not_paused_errors() {
        let env = Env::default();
        let contract_id = env.register(DummyContract, ());
        let caller = Address::generate(&env);
        let pausable = Pausable::new(symbol_short!("pause"));

        env.as_contract(&contract_id, || {
            assert!(!pausable.is_paused(&env));
            assert_eq!(pausable.require_not_paused(&env), Ok(()));
            assert_eq!(
                pausable.require_paused(&env),
                Err(StandardsError::NotPaused)
            );
            assert_eq!(
                pausable.unpause(&env, &caller),
                Err(StandardsError::NotPaused)
            );

            let paused_event = pausable.pause(&env, &caller).unwrap();
            assert_eq!(
                paused_event,
                PausedEvent {
                    account: caller.clone(),
                }
            );
            assert!(pausable.is_paused(&env));
            assert_eq!(pausable.require_paused(&env), Ok(()));
            assert_eq!(
                pausable.require_not_paused(&env),
                Err(StandardsError::Paused)
            );
            assert_eq!(pausable.pause(&env, &caller), Err(StandardsError::Paused));

            let unpaused_event = pausable.unpause(&env, &caller).unwrap();
            assert_eq!(
                unpaused_event,
                UnpausedEvent {
                    account: caller.clone(),
                }
            );
            assert!(!pausable.is_paused(&env));
            assert_eq!(pausable.require_not_paused(&env), Ok(()));
            assert_eq!(
                pausable.require_paused(&env),
                Err(StandardsError::NotPaused)
            );
        });
    }
}
