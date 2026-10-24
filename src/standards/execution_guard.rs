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

    /// Acquires the execution lock if unlocked.
    ///
    /// Note: This helper does not perform caller authentication; the
    /// integrating contract must authenticate the caller before invoking
    /// guarded entrypoints.
    pub fn enter(&self, env: &Env) -> Result<ExecutionGuardEnteredEvent, StandardsError> {
        self.require_unlocked(env)?;
        env.storage().persistent().set(&self.lock_key(env), &true);
        Ok(ExecutionGuardEnteredEvent {
            guard_id: self.id.clone(),
        })
    }

    /// Releases the execution lock if currently locked.
    ///
    /// Note: This helper does not perform caller authentication; the
    /// integrating contract must authenticate the caller before invoking
    /// guarded entrypoints.
    pub fn exit(&self, env: &Env) -> Result<ExecutionGuardExitedEvent, StandardsError> {
        if !self.is_locked(env) {
            return Err(StandardsError::ExecutionNotLocked);
        }
        env.storage().persistent().set(&self.lock_key(env), &false);
        Ok(ExecutionGuardExitedEvent {
            guard_id: self.id.clone(),
        })
    }

    /// Executes `f` while holding the execution lock and releases it afterward.
    ///
    /// Note: This helper does not perform caller authentication; the
    /// integrating contract must authenticate the caller before invoking
    /// guarded entrypoints.
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

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{contract, contractimpl, symbol_short};

    #[contract]
    struct DummyContract;

    #[contractimpl]
    impl DummyContract {}

    #[test]
    fn execution_guard_enter_exit_and_execute_cover_locked_and_unlocked_errors() {
        let env = Env::default();
        let contract_id = env.register(DummyContract, ());
        let guard_id = symbol_short!("exec");
        let guard = ExecutionGuard::new(guard_id.clone());

        env.as_contract(&contract_id, || {
            assert!(!guard.is_locked(&env));
            assert_eq!(guard.require_unlocked(&env), Ok(()));
            assert_eq!(guard.exit(&env), Err(StandardsError::ExecutionNotLocked));

            let entered = guard.enter(&env).unwrap();
            assert_eq!(
                entered,
                ExecutionGuardEnteredEvent {
                    guard_id: guard_id.clone(),
                }
            );
            assert!(guard.is_locked(&env));
            assert_eq!(
                guard.require_unlocked(&env),
                Err(StandardsError::ExecutionLocked)
            );
            assert_eq!(guard.enter(&env), Err(StandardsError::ExecutionLocked));
            assert_eq!(
                guard.execute(&env, || 1u32),
                Err(StandardsError::ExecutionLocked)
            );

            let exited = guard.exit(&env).unwrap();
            assert_eq!(
                exited,
                ExecutionGuardExitedEvent {
                    guard_id: guard_id.clone(),
                }
            );
            assert!(!guard.is_locked(&env));

            let result = guard
                .execute(&env, || {
                    assert!(guard.is_locked(&env));
                    99u32
                })
                .unwrap();
            assert_eq!(result, 99);
            assert!(!guard.is_locked(&env));
        });
    }
}
