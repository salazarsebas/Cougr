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

    pub fn pause(&self, env: &Env, caller: &Address) -> Result<PausedEvent, StandardsError> {
        if self.is_paused(env) {
            return Err(StandardsError::Paused);
        }
        env.storage().persistent().set(&self.paused_key(env), &true);
        Ok(PausedEvent {
            account: caller.clone(),
        })
    }

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
