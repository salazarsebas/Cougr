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
