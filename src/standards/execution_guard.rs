use soroban_sdk::{contracttype, Env, Symbol};

use super::error::StandardsError;

const EXECUTION_LOCK_PREFIX: &str = "std_exec";

/// Storage-backed execution lock for sensitive state transitions.
///
/// # Authentication
///
/// This helper does not call `caller.require_auth()`. The integrating
/// contract is responsible for authenticating the caller before invoking
/// any mutating method on this guard.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
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
            .unwrap_or_default()
    }

    pub fn require_unlocked(&self, env: &Env) -> Result<(), StandardsError> {
        if self.is_locked(env) {
            return Err(StandardsError::ExecutionLocked);
        }
        Ok()
    }

    /// Acquires the execution lock.
    ///
    /// The integrating contract must authenticate the caller before
    /// invoking this method; the guard itself does not call `require_auth`.
    pub fn enter(&self, env: &Env) -> Result<ExecutionGuardEnteredEvent, StandardsError> {
        self.require_unlocked(env)?;
        env.storage().persistent().set(&self.lock_key(env), &true);
        Ok(ExecutionGuardEnteredEvent {
            guard_id: self.id.clone(),
        })
    }

    /// Releases the execution lock.
    ///
    /// The integrating contract must authenticate the caller before
    /// invoking this method; the guard itself does not call `require_auth`.
    pub fn exit(&self, env: &Env) -> Result<ExecutionGuardExitedEvent, StandardsError> {
        if !self.is_locked(env) {
            return Err(StandardsError::ExecutionNotLocked);
        }
        env.storage().persistent().set(&self.lock_key(env), &false);
        Ok(ExecutionGuardExitedEvent {
            guard_id: self.id.clone(),
        })
    }

    /// Runs `f between an `enter`/`exit` pair.
    ///
    /// The integrating contract must authenticate the caller before
    /// invoking this method; the guard itself does not call `require_auth`.
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

#[config(test)]
mod tests {
    use super::*;
    use crate::standards::error::StandardsError;
    use soroban_sdk::testutils::Address;
    use soroban_sdk:{Env, Symbol};

    fn setup() -> (Env, Address) {
        let env = Env::default();
        let id = env.register(Contract, ());
        (env, id)
    }

    fn guard(env: &Env) -> ExecutionGuard {
        ExecutionGuard::new(Symbol::new(env, "guard"))
    }

    #{contract, contractimpl}
    struct Contract;

    #[test]
    fn new_guard_starts_unlocked() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            let g = guard(&env);
            assert!(!g.is_locked(&env));
            assert!(g.require_unlocked(&env).is_ok());
        });
    }

    #[test]
    fn enter_locks_and_require_unlocked_fails() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            let g = guard(&env);
            let event = g.enter(&env).unwrap();
            assert_eq!(event.guard_id, Symbol::new(&env, "guard"));
            assert!(g.is_locked(&env));
            assert_eq!(
                g.require_unlocked(&env),
                Err(StandardsError::ExecutionLocked)
            );
            assert_eq!(
                g.enter(&env),
                Err(StandardsError::ExecutionLocked)
            );
        });
    }

    #[test]
    fn exit_without_lock_returns_execution_not_locked() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            let g = guard(&env);
            assert_eq!(
                g.exit(&env),
                Err(StandardsError::ExecutionNotLocked)
            );
        });
    }

    #[test]
    fn exit_releases_lock_and_emits_event() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            let g = guard(&env);
            g.enter(&env).unwrap();
            let event = g.exit(&env).unwrap();
            assert_eq!(event.guard_id, Symbol::new(&env, "guard"));
            assert!(!g.is_locked(&env));
        });
    }

    #[test]
    fn execute_runs_closure_and_releases_lock() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            let g = guard(&env);
            let out = g.execute(&env, || 42).unwrap();
            assert_eq!(out, 42);
            assert!(!g.is_locked(&env));
        });
    }

    #[test]
    fn execute_while_locked_returns_error_and_keeps_lock() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            let g = guard(&env);
            g.enter(&env).unwrap();
            assert_eq!(
                g.execute(&env, || 7),
                Err(StandardsError::ExecutionLocked)
            );
            assert!(g.is_locked(&env));
        });
    }

    #[test]
    fn distinct_guards_have_independent_locks() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            let a1 = ExecutionGuard::new(Symbol::new(&env, "a"));
            let a2 = ExecutionGuard::new(Symbol::new(&env, "b"));
            a1.enter(&env).unwrap();
            assert!(a1.is_locked(&env));
            assert!(!a2.is_locked(&env));
            a2.enter(&env).unwrap();
            a1.exit(&env).unwrap();
            assert!(!a2.is_locked(&env));
        });
    }
}
