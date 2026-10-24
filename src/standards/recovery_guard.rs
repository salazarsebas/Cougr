use soroban_sdk::{contracttype, Address, Env, Symbol};

use super::error::StandardsError;

const RECOVERY_GUARD_PREFIX: &str = "std_recov";

/// Guard used to block sensitive flows while recovery is active.
#[derive(Clone, Debug)]
pub struct RecoveryGuard {
    id: Symbol,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryGuardActivatedEvent {
    pub account: Address,
    pub activated_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryGuardClearedEvent {
    pub account: Address,
    pub cleared_at: u64,
}

impl RecoveryGuard {
    pub fn new(id: Symbol) -> Self {
        Self { id }
    }

    pub fn is_active(&self, env: &Env) -> bool {
        env.storage()
            .persistent()
            .get::<_, bool>(&self.guard_key(env))
            .unwrap_or(false)
    }

    pub fn require_active(&self, env: &Env) -> Result<(), StandardsError> {
        if self.is_active(env) {
            return Ok(());
        }
        Err(StandardsError::RecoveryInactive)
    }

    pub fn require_inactive(&self, env: &Env) -> Result<(), StandardsError> {
        if self.is_active(env) {
            return Err(StandardsError::RecoveryActive);
        }
        Ok(())
    }

    /// Activates the recovery guard and returns a [`RecoveryGuardActivatedEvent`].
    ///
    /// Note: `caller` is recorded in the returned event only and is not
    /// authenticated by this helper; the integrating contract must authenticate
    /// and authorize `caller` before invoking this method.
    pub fn activate(
        &self,
        env: &Env,
        caller: &Address,
    ) -> Result<RecoveryGuardActivatedEvent, StandardsError> {
        self.require_inactive(env)?;
        env.storage().persistent().set(&self.guard_key(env), &true);
        Ok(RecoveryGuardActivatedEvent {
            account: caller.clone(),
            activated_at: env.ledger().timestamp(),
        })
    }

    /// Clears the recovery guard and returns a [`RecoveryGuardClearedEvent`].
    ///
    /// Note: `caller` is recorded in the returned event only and is not
    /// authenticated by this helper; the integrating contract must authenticate
    /// and authorize `caller` before invoking this method.
    pub fn clear(
        &self,
        env: &Env,
        caller: &Address,
    ) -> Result<RecoveryGuardClearedEvent, StandardsError> {
        self.require_active(env)?;
        env.storage().persistent().set(&self.guard_key(env), &false);
        Ok(RecoveryGuardClearedEvent {
            account: caller.clone(),
            cleared_at: env.ledger().timestamp(),
        })
    }

    fn guard_key(&self, env: &Env) -> (Symbol, Symbol) {
        (Symbol::new(env, RECOVERY_GUARD_PREFIX), self.id.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        contract, contractimpl, symbol_short, testutils::Address as _, testutils::Ledger as _,
    };

    #[contract]
    struct DummyContract;

    #[contractimpl]
    impl DummyContract {}

    #[test]
    fn recovery_guard_activate_and_clear_cover_active_and_inactive_errors() {
        let env = Env::default();
        let contract_id = env.register(DummyContract, ());
        let caller = Address::generate(&env);
        let guard = RecoveryGuard::new(symbol_short!("recov"));

        env.as_contract(&contract_id, || {
            env.ledger().with_mut(|li| {
                li.timestamp = 123;
            });

            assert!(!guard.is_active(&env));
            assert_eq!(guard.require_inactive(&env), Ok(()));
            assert_eq!(
                guard.require_active(&env),
                Err(StandardsError::RecoveryInactive)
            );
            assert_eq!(
                guard.clear(&env, &caller),
                Err(StandardsError::RecoveryInactive)
            );

            let activated = guard.activate(&env, &caller).unwrap();
            assert_eq!(
                activated,
                RecoveryGuardActivatedEvent {
                    account: caller.clone(),
                    activated_at: 123,
                }
            );
            assert!(guard.is_active(&env));
            assert_eq!(guard.require_active(&env), Ok(()));
            assert_eq!(
                guard.require_inactive(&env),
                Err(StandardsError::RecoveryActive)
            );
            assert_eq!(
                guard.activate(&env, &caller),
                Err(StandardsError::RecoveryActive)
            );

            env.ledger().with_mut(|li| {
                li.timestamp = 456;
            });

            let cleared = guard.clear(&env, &caller).unwrap();
            assert_eq!(
                cleared,
                RecoveryGuardClearedEvent {
                    account: caller.clone(),
                    cleared_at: 456,
                }
            );
            assert!(!guard.is_active(&env));
            assert_eq!(guard.require_inactive(&env), Ok(()));
            assert_eq!(
                guard.require_active(&env),
                Err(StandardsError::RecoveryInactive)
            );
        });
    }
}
