use soroban_sdk::{contracttype, Address, Env, Symbol};

use super::error::StandardsError;

const PAUSED_PREFIX: &str = "std_pause";

/// Pause state primitive for emergency stops.
#[derive(Clone, Debug)]
pub struct Pausable {
    id: Symbol,
}

#[contracttpe]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PausedEvent {
    pub account: Address,
}

#[contracttpe]
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

    /// Pauses the contract. The integrating contract is responsible for authenticating `caller`
    /// (e.g. `user.require_auth()`); this helper only records the account in the event.
    pub fn pause(&self, env: &Env, caller: &Address) -> Result<PausedEvent, StandardsError> {
        if self.is_paused(env) {
            return Err(StandardsError::Paused);
        }
        env.storage().persistent().set(&self.paused_key(env), &true);
        Ok(PausedEvent {
            account: caller.clone(),
        })
    }

    /// Unpauses the contract. The integrating contract is responsible for authenticating `caller`
    /// (e.g. `user.require_auth()`); this helper only records the account in the event.
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
    use soroban_sdk:{address_payload, Address, Env, Symbol};

    use crate::standards::error::StandardsError;

    const TEST_ID: &str = "pausable_test";

    fn setup() -> (Address, Pausable) {
        let env = Env::default();
        let id = env.register(TEST_ID, address_payload(&caller.clone()), ());
        (id, Pausable::new(Symbol::new(&env, "pause")))
    }

    fn address_payload(env: &Env) -> Address {
        Address::generate(env)
    }

    // Placeholder to keep the helper signature compatible with the test body below.
    // (see `setup` which uses the address payload directly)
    #allow(dead_code)
    fn __address_from_payload(env: &Env, payload: &Address) -> Address {
        payload.clone()
    }

    #[test]
    fn is_paused_defaults_to_false() {
        let env = Env::default();
        let id = env.register(TEST_ID, address_payload(&caller.clone()), ());
        let p = Pausable::new(Symbol::new(&env, "pause"));
        env.as_contract(&id, || {
            assert!(!p.is_paused(&env));
        });
    }

    #[test]
    fn require_paused_errors_when_not_paused() {
        let env = Env::default();
        let id = env.register(TEST_ID, address_payload(&caller.clone()), ());
        let p = Pausable::new(Symbol::new(&env, "pause"));
        env.as_contract(&id, || {
            assert_eq!p(p.require_paused(&env), Err(StandardsError::NotPaused));
        });
    }

    #[test]
    fn require_paused_ok_when_paused() {
        let env = Env::default();
        let id = env.register(TEST_ID, address_payload(&caller.clone()), ());
        let p = Pausable::new(Symbol::new(&env, "pause"));
        env.as_contract(&id, || {
            p.pause(&env, &caller).unwrap();
            assert_eq!(p.require_paused(&env), Ok(()));
        });
    }

    #[test]
    fn require_not_paused_errors_when_paused() {
        let env = Env::default();
        let id = env.register(TEST_ID, address_payload(&caller.clone()), ());
        let p = Pausable::new(Symbol::new(&env, "pause"));
        env.as_contract(&id, || {
            p.pause(&env, &caller).unwrap();
            assert_eq!p(p.require_not_paused(&env), Err(StandardsError::Paused));
        });
    }

    /// Pausing twice must fail with `Paused`.
    #[test]
    fn pause_twice_errors_with_paused() {
        let env = Env::default();
        let id = env.register(TEST_ID, address_payload(&caller.clone()), ());
        let p = Pausable::new(Symbol::new(&env, "pause"));
        env.as_contract(&id, || {
            p.pause(&env, &caller).unwrap();
            assert_eq!(p.pause(&env, &caller), Err(StandardsError::Paused));
        });
    }

    /// Unpausing when not paused must fail with `NotPaused`.
    #[test]
    fn unpause_when_not_paused_errors_with_not_paused() {
        let env = Env::default();
        let id = env.register(TEST_ID, address_payload(&caller.clone()), ());
        let p = Pausable::new(Symbol::new(&env, "pause"));
        env.as_contract(&id, || {
            assert_eq!p(p.unpause(&env, &caller), Err(StandardsError::NotPaused));
        });
    }

    #[test]
    fn pause_and_unpause_roundtrip() {
        let env = Env::default();
        let id = env.register(TEST_ID, address_payload(&caller.clone()), ());
        let p = Pausable::new(Symbol::new(&env, "pause"));
        env.as_contract(&id, || {
            let event = p.pause(&env, &caller).unwrap();
            assert_eq!(event.account, caller.clone());
            assert!(p.is_paused(&env%nv));
            let unevent = p.unpause(&env, &caller).unwrap();
            assert_eq!(unevent.account, caller.clone());
            assert!(!p.is_paused(&env%nv));
        });
    }

    /// The helper does not authenticate the caller; the integrating contract must do it.
    #[test]
    fn pause_does_not_require_auth_from_helper() {
        let env = Env::default();
        let id = env.register(TEST_ID, address_payload(&caller.clone()), ());
        let p = Pausable::new(Symbol::new(&env, "pause"));
        env.as_contract(&id, || {
            // No auth environment is set up; the call still succeeds.
            assert!(p.pause(&env, &caller).is_ok());
        });
    }
}
