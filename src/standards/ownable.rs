use soroban_sdk::{contracttype, Address, Env, Symbol};

use super::error::StandardsError;

const OWNER_PREFIX: &str = "std_owner";
const PENDING_OWNER_PREFIX: &str = "std_powner";

/// OpenZeppelin-style single-owner control primitive.
#[derive(Clone, Debug)]
pub struct Ownable {
    id: Symbol,
}

/// Two-step ownership handoff built on top of [`Ownable`].
#[derive(Clone, Debug)]
pub struct Ownable2Step {
    inner: Ownable,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnershipTransferredEvent {
    pub previous_owner: Option<Address>,
    pub new_owner: Option<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnershipTransferStartedEvent {
    pub owner: Address,
    pub pending_owner: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnershipTransferCancelledEvent {
    pub owner: Address,
    pub pending_owner: Address,
}

impl Ownable {
    pub fn new(id: Symbol) -> Self {
        Self { id }
    }

    /// Initializes the module with an initial `owner`.
    ///
    /// Note: This helper does not call `owner.require_auth()`; the integrating
    /// contract must authenticate the caller before invoking this method.
    pub fn initialize(
        &self,
        env: &Env,
        owner: &Address,
    ) -> Result<OwnershipTransferredEvent, StandardsError> {
        if self.owner(env).is_some() {
            return Err(StandardsError::AlreadyInitialized);
        }

        env.storage().persistent().set(&self.owner_key(env), owner);
        env.storage()
            .persistent()
            .remove(&self.pending_owner_key(env));

        Ok(OwnershipTransferredEvent {
            previous_owner: None,
            new_owner: Some(owner.clone()),
        })
    }

    pub fn owner(&self, env: &Env) -> Option<Address> {
        env.storage().persistent().get(&self.owner_key(env))
    }

    pub fn pending_owner(&self, env: &Env) -> Option<Address> {
        env.storage().persistent().get(&self.pending_owner_key(env))
    }

    pub fn require_owner(&self, env: &Env, caller: &Address) -> Result<(), StandardsError> {
        let owner = self.owner(env).ok_or(StandardsError::OwnerNotSet)?;
        if owner != *caller {
            return Err(StandardsError::Unauthorized);
        }
        Ok(())
    }

    /// Transfers ownership from `caller` to `new_owner`.
    ///
    /// Note: This helper compares `caller` against the stored owner but does
    /// not call `caller.require_auth()`; the integrating contract must
    /// authenticate `caller` before invoking this method.
    pub fn transfer_ownership(
        &self,
        env: &Env,
        caller: &Address,
        new_owner: &Address,
    ) -> Result<OwnershipTransferredEvent, StandardsError> {
        self.require_owner(env, caller)?;
        let previous_owner = self.owner(env);
        env.storage()
            .persistent()
            .set(&self.owner_key(env), new_owner);
        env.storage()
            .persistent()
            .remove(&self.pending_owner_key(env));

        Ok(OwnershipTransferredEvent {
            previous_owner,
            new_owner: Some(new_owner.clone()),
        })
    }

    /// Renounces ownership when called by the current owner.
    ///
    /// Note: This helper compares `caller` against the stored owner but does
    /// not call `caller.require_auth()`; the integrating contract must
    /// authenticate `caller` before invoking this method.
    pub fn renounce_ownership(
        &self,
        env: &Env,
        caller: &Address,
    ) -> Result<OwnershipTransferredEvent, StandardsError> {
        self.require_owner(env, caller)?;
        let previous_owner = self.owner(env);
        env.storage().persistent().remove(&self.owner_key(env));
        env.storage()
            .persistent()
            .remove(&self.pending_owner_key(env));

        Ok(OwnershipTransferredEvent {
            previous_owner,
            new_owner: None,
        })
    }

    fn owner_key(&self, env: &Env) -> (Symbol, Symbol) {
        (Symbol::new(env, OWNER_PREFIX), self.id.clone())
    }

    fn pending_owner_key(&self, env: &Env) -> (Symbol, Symbol) {
        (Symbol::new(env, PENDING_OWNER_PREFIX), self.id.clone())
    }
}

impl Ownable2Step {
    pub fn new(id: Symbol) -> Self {
        Self {
            inner: Ownable::new(id),
        }
    }

    /// Initializes the two-step ownership module with an initial `owner`.
    ///
    /// Note: This helper does not call `owner.require_auth()`; the integrating
    /// contract must authenticate the caller before invoking this method.
    pub fn initialize(
        &self,
        env: &Env,
        owner: &Address,
    ) -> Result<OwnershipTransferredEvent, StandardsError> {
        self.inner.initialize(env, owner)
    }

    pub fn owner(&self, env: &Env) -> Option<Address> {
        self.inner.owner(env)
    }

    pub fn pending_owner(&self, env: &Env) -> Option<Address> {
        self.inner.pending_owner(env)
    }

    pub fn require_owner(&self, env: &Env, caller: &Address) -> Result<(), StandardsError> {
        self.inner.require_owner(env, caller)
    }

    /// Starts a two-step ownership handoff to `pending_owner`.
    ///
    /// Note: This helper compares `caller` against the stored owner but does
    /// not call `caller.require_auth()`; the integrating contract must
    /// authenticate `caller` before invoking this method.
    pub fn begin_transfer(
        &self,
        env: &Env,
        caller: &Address,
        pending_owner: &Address,
    ) -> Result<OwnershipTransferStartedEvent, StandardsError> {
        self.inner.require_owner(env, caller)?;
        env.storage()
            .persistent()
            .set(&self.inner.pending_owner_key(env), pending_owner);

        Ok(OwnershipTransferStartedEvent {
            owner: caller.clone(),
            pending_owner: pending_owner.clone(),
        })
    }

    /// Completes a pending ownership transfer when called by `pending_owner`.
    ///
    /// Note: This helper compares `caller` against the stored pending owner but
    /// does not call `caller.require_auth()`; the integrating contract must
    /// authenticate `caller` before invoking this method.
    pub fn accept_transfer(
        &self,
        env: &Env,
        caller: &Address,
    ) -> Result<OwnershipTransferredEvent, StandardsError> {
        let pending_owner = self
            .pending_owner(env)
            .ok_or(StandardsError::PendingOwnerNotSet)?;
        if pending_owner != *caller {
            return Err(StandardsError::PendingOwnerMismatch);
        }

        let previous_owner = self.owner(env);
        env.storage()
            .persistent()
            .set(&self.inner.owner_key(env), caller);
        env.storage()
            .persistent()
            .remove(&self.inner.pending_owner_key(env));

        Ok(OwnershipTransferredEvent {
            previous_owner,
            new_owner: Some(caller.clone()),
        })
    }

    /// Cancels an active two-step ownership transfer when called by the owner.
    ///
    /// Note: This helper compares `caller` against the stored owner but does
    /// not call `caller.require_auth()`; the integrating contract must
    /// authenticate `caller` before invoking this method.
    pub fn cancel_transfer(
        &self,
        env: &Env,
        caller: &Address,
    ) -> Result<OwnershipTransferCancelledEvent, StandardsError> {
        self.inner.require_owner(env, caller)?;
        let pending_owner = self
            .pending_owner(env)
            .ok_or(StandardsError::PendingOwnerNotSet)?;
        env.storage()
            .persistent()
            .remove(&self.inner.pending_owner_key(env));

        Ok(OwnershipTransferCancelledEvent {
            owner: caller.clone(),
            pending_owner,
        })
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
    fn ownable_initialize_sets_owner_and_rejects_duplicate_init() {
        let env = Env::default();
        let contract_id = env.register(DummyContract, ());
        let owner = Address::generate(&env);
        let other = Address::generate(&env);
        let ownable = Ownable::new(symbol_short!("own"));

        env.as_contract(&contract_id, || {
            assert_eq!(ownable.owner(&env), None);
            assert_eq!(ownable.pending_owner(&env), None);

            let event = ownable.initialize(&env, &owner).unwrap();
            assert_eq!(
                event,
                OwnershipTransferredEvent {
                    previous_owner: None,
                    new_owner: Some(owner.clone()),
                }
            );
            assert_eq!(ownable.owner(&env), Some(owner.clone()));

            assert_eq!(
                ownable.initialize(&env, &other),
                Err(StandardsError::AlreadyInitialized)
            );
        });
    }

    #[test]
    fn ownable_require_owner_enforces_initialized_and_matching_caller() {
        let env = Env::default();
        let contract_id = env.register(DummyContract, ());
        let owner = Address::generate(&env);
        let outsider = Address::generate(&env);
        let ownable = Ownable::new(symbol_short!("own"));

        env.as_contract(&contract_id, || {
            assert_eq!(
                ownable.require_owner(&env, &owner),
                Err(StandardsError::OwnerNotSet)
            );

            ownable.initialize(&env, &owner).unwrap();
            assert_eq!(ownable.require_owner(&env, &owner), Ok(()));
            assert_eq!(
                ownable.require_owner(&env, &outsider),
                Err(StandardsError::Unauthorized)
            );
        });
    }

    #[test]
    fn ownable_transfer_and_renounce_check_caller_and_clear_pending_owner() {
        let env = Env::default();
        let contract_id = env.register(DummyContract, ());
        let owner = Address::generate(&env);
        let next_owner = Address::generate(&env);
        let outsider = Address::generate(&env);
        let ownable = Ownable::new(symbol_short!("own"));
        let two_step = Ownable2Step::new(symbol_short!("own"));

        env.as_contract(&contract_id, || {
            assert_eq!(
                ownable.transfer_ownership(&env, &owner, &next_owner),
                Err(StandardsError::OwnerNotSet)
            );
            assert_eq!(
                ownable.renounce_ownership(&env, &owner),
                Err(StandardsError::OwnerNotSet)
            );

            ownable.initialize(&env, &owner).unwrap();
            two_step.begin_transfer(&env, &owner, &outsider).unwrap();
            assert_eq!(ownable.pending_owner(&env), Some(outsider.clone()));

            assert_eq!(
                ownable.transfer_ownership(&env, &outsider, &next_owner),
                Err(StandardsError::Unauthorized)
            );

            let transferred = ownable
                .transfer_ownership(&env, &owner, &next_owner)
                .unwrap();
            assert_eq!(
                transferred,
                OwnershipTransferredEvent {
                    previous_owner: Some(owner.clone()),
                    new_owner: Some(next_owner.clone()),
                }
            );
            assert_eq!(ownable.owner(&env), Some(next_owner.clone()));
            assert_eq!(ownable.pending_owner(&env), None);

            assert_eq!(
                ownable.renounce_ownership(&env, &owner),
                Err(StandardsError::Unauthorized)
            );

            let renounced = ownable.renounce_ownership(&env, &next_owner).unwrap();
            assert_eq!(
                renounced,
                OwnershipTransferredEvent {
                    previous_owner: Some(next_owner),
                    new_owner: None,
                }
            );
            assert_eq!(ownable.owner(&env), None);
        });
    }

    #[test]
    fn ownable_2step_begin_accept_and_cancel_transfer_cover_all_paths() {
        let env = Env::default();
        let contract_id = env.register(DummyContract, ());
        let owner = Address::generate(&env);
        let pending = Address::generate(&env);
        let outsider = Address::generate(&env);
        let ownable = Ownable2Step::new(symbol_short!("own2"));

        env.as_contract(&contract_id, || {
            assert_eq!(
                ownable.begin_transfer(&env, &owner, &pending),
                Err(StandardsError::OwnerNotSet)
            );
            assert_eq!(
                ownable.cancel_transfer(&env, &owner),
                Err(StandardsError::OwnerNotSet)
            );
            assert_eq!(
                ownable.accept_transfer(&env, &pending),
                Err(StandardsError::PendingOwnerNotSet)
            );

            ownable.initialize(&env, &owner).unwrap();
            assert_eq!(ownable.require_owner(&env, &owner), Ok(()));
            assert_eq!(
                ownable.cancel_transfer(&env, &owner),
                Err(StandardsError::PendingOwnerNotSet)
            );
            assert_eq!(
                ownable.begin_transfer(&env, &outsider, &pending),
                Err(StandardsError::Unauthorized)
            );

            let started = ownable.begin_transfer(&env, &owner, &pending).unwrap();
            assert_eq!(
                started,
                OwnershipTransferStartedEvent {
                    owner: owner.clone(),
                    pending_owner: pending.clone(),
                }
            );
            assert_eq!(ownable.pending_owner(&env), Some(pending.clone()));

            assert_eq!(
                ownable.cancel_transfer(&env, &outsider),
                Err(StandardsError::Unauthorized)
            );

            let cancelled = ownable.cancel_transfer(&env, &owner).unwrap();
            assert_eq!(
                cancelled,
                OwnershipTransferCancelledEvent {
                    owner: owner.clone(),
                    pending_owner: pending.clone(),
                }
            );
            assert_eq!(ownable.pending_owner(&env), None);
            assert_eq!(
                ownable.accept_transfer(&env, &pending),
                Err(StandardsError::PendingOwnerNotSet)
            );

            ownable.begin_transfer(&env, &owner, &pending).unwrap();
            assert_eq!(
                ownable.accept_transfer(&env, &outsider),
                Err(StandardsError::PendingOwnerMismatch)
            );

            let accepted = ownable.accept_transfer(&env, &pending).unwrap();
            assert_eq!(
                accepted,
                OwnershipTransferredEvent {
                    previous_owner: Some(owner),
                    new_owner: Some(pending.clone()),
                }
            );
            assert_eq!(ownable.owner(&env), Some(pending));
            assert_eq!(ownable.pending_owner(&env), None);
        });
    }
}
