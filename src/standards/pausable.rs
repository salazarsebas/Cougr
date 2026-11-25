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

    /// Marks the module as paused and returns a [`PausedEvent`] recording `caller`.
    ///
    /// Note: The integrating contract is responsible for authenticating `caller`.
    pub fn pause(&self, env: &Env, caller: &Address) -> Result<PausedEvent, StandardsError> {
        if self.is_paused(env) {
            return Err(StandardsError::Paused);
        }
        env.storage().persistent().set(&self.paused_key(env), &true);
        Ok(PausedEvent {
            account: caller.clone(),
        })
    }

    /// Marks the module as unpaused and returns an [`UnpausedEvent`] recording `caller`.
    ///
    /// Note: The integrating contract is responsible for authenticating `caller`.
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
    use soroban_sdk::{
        contract, contractimpl, symbol_short, testutils::Address as _, Address, Env,
    };

    use super::*;

    #[contract]
    pub struct TestContract;

    #[contractimpl]
    impl TestContract {}

    #[test]
    fn pausable_covers_pause_unpause_and_double_transition_errors() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let caller = Address::generate(&env);
        let module = Pausable::new(symbol_short!("pause"));

        env.as_contract(&contract_id, || {
            assert!(!module.is_paused(&env));
            assert!(module.require_not_paused(&env).is_ok());
            assert_eq!(module.require_paused(&env), Err(StandardsError::NotPaused));
            assert_eq!(
                module.unpause(&env, &caller),
                Err(StandardsError::NotPaused)
            );

            let paused = module.pause(&env, &caller).unwrap();
            assert_eq!(
                paused,
                PausedEvent {
                    account: caller.clone()
                }
            );
            assert!(module.is_paused(&env));
            assert!(module.require_paused(&env).is_ok());
            assert_eq!(module.require_not_paused(&env), Err(StandardsError::Paused));
            assert_eq!(module.pause(&env, &caller), Err(StandardsError::Paused));

            let unpaused = module.unpause(&env, &caller).unwrap();
            assert_eq!(
                unpaused,
                UnpausedEvent {
                    account: caller.clone()
                }
            );
            assert!(!module.is_paused(&env));
            assert_eq!(module.require_paused(&env), Err(StandardsError::NotPaused));
        });
    }

    #[test]
    fn pausable_mutators_do_not_call_require_auth_internally() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let caller = Address::generate(&env);
        let module = Pausable::new(symbol_short!("noauth"));

        env.as_contract(&contract_id, || {
            module.pause(&env, &caller).unwrap();
            module.unpause(&env, &caller).unwrap();
            assert_eq!(env.auths().len(), 0);
        });
    }
}
