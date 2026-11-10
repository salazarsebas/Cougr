use soroban_sdk::{contracttype, Env, Symbol};

use super::error::StandardsError;

const EXECUTION_LOCK_PREFIX: &str = "std_exec";

/// Storage-backed execution lock for sensitive state transitions.
#[derive(Clone, Debug)]
pub struct ExecutionGuard {
    id: Symbol,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionGuardEnteredEvent {
    pub guard_id: Symbol,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionGuardExitedEvent {
    pub guard_id: Symbol,
}

impl ExecutionGuard {
    pub fn new(id: Symbol) -> Self {
        Self { id }
    }

    pub fn is_locked(&self, env: &Env) -> bool {
        env.storage()
            .persistent()
            .get(&self.lock_key(env))
            .unwrap_or(false)
    }

    pub fn require_unlocked(&self, env: &Env) -> Result<(), StandardsError> {
        if self.is_locked(env) {
            return Err(StandardsError::ExecutionLocked);
        }
        Ok(())
    }

    pub fn enter(&self, env: &Env) -> Result<ExecutionGuardEnteredEvent, StandardsError> {
        self.require_unlocked(env)?;
        env.storage().persistent().set(&self.lock_key(env), &true);
        Ok(ExecutionGuardEnteredEvent {
            guard_id: self.id.clone(),
        })
    }

    pub fn exit(&self, env: &Env) -> Result<ExecutionGuardExitedEvent, StandardsError> {
        if !self.is_locked(env) {
            return Err(StandardsError::ExecutionNotLocked);
        }
        env.storage().persistent().set(&self.lock_key(env), &false);
        Ok(ExecutionGuardExitedEvent {
            guard_id: self.id.clone(),
        })
    }

    pub fn execute<T>(&self, env: &Env, f: impl FnOnce() -> T) -> Result<T, StandardsError> {
        self.enter(env)?;
        let result = f();
        self.exit(env)?;
        Ok(result)
    }

    fn lock_key(&self, env: &Env) -> (Symbol, Symbol) {
        (Symbol::new(env, EXECUTION_LOCK_PREFIX), self.id.clone())
    }
}
