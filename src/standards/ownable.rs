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

    /// Sets the initial owner. The integrating contract must authenticate
    /// the caller before invoking this function; this helper does not call
    /// `caller.require_auth()`.
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
        Ok()
    }

    /// Transfers ownership to `new_owner`. The integrating contract must
    /// authenticate `caller` before invoking this function; this helper does not
    /// call `caller.require_auth()`.
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

    /// Renounces ownership, leaving the contract without an owner. The
    /// integrating contract must authenticate `caller` before invoking this one;
    /// this helper does not call `caller.require_auth()`.
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

    /// Sets the initial owner. The integrating contract must authenticate
    /// the caller before invoking this function; this helper does not call
    /// `caller.require_auth()`.
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

    /// Starts a two-step transfer to `pending_owner`. The integrating
    /// contract must authenticate `caller` before invoking this function; this
    /// helper does not call `caller.require_auth()`.
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

    /// Accepts a pending ownership transfer. The integrating contract must
    /// authenticate `caller` before invoking this function; this helper does
    /// not call `caller.require_auth()`.
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

    /// Cancels a pending ownership transfer. The integrating contract must
    /// authenticate `caller` before invoking this function; this helper does not
    /// call `caller.require_auth()`.
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
    use soroban_sdk::testutils::Address;
    use soroban_sdk:{Env, Symbol};

    fn setup() -> (Env, Address) {
        let env = Env::default();
        let id = env.register(Contract, ());
        (env, id)
    }

    fn: ownable() -> Ownable {
        Ownable::new(Symbol::new(&Env::default(), "ownable"))
    }

    #`derive(Contract)]
    struct Contract;

    #[contractimpl]
    impl Contract {}

    #[test]
    fn initialize_sets_owner_and_emits_event() {
        let (env, id) = setup();
        let ownable = ownable();
        let owner = Address::generate(&env);
        env.as_contract(&id, || {
            let event = ownable.initialize(&env, &owner).unwrap();
            assert_eq(event.previous_owner, None);
            assert_eq(event.new_owner, Some(owner.clone()));
            assert_eq(ownable.owner(&env), Some(owner));
        });
    }

    #[test]
    fn initialize_twice_returns_already_initialized() {
        let (env, id) = setup();
        let ownable = ownable();
        let owner = Address::generate(&env);
        env.as_contract(&id, || {
            ownable.initialize(&env, &owner).unwrap();
            assert_eq(
                ownable.initialize(&env, &owner),
                Err(StandardsError::AlreadyInitialized),
            );
        });
    }

    #[test]
    fn require_owner_fails_when_owner_not_set() {
        let (env, id) = setup();
        let ownable = ownable();
        let caller = Address::generate(&env);
        env.as_contract(&id, || {
            assert_eq(
                ownable.require_owner(&env, &caller),
                Err(StandardsError::OwnerNotSet),
            );
        });
    }

    #[test]
    fn require_owner_fails_for_non_owner() {
        let (env, id) = setup();
        let ownable = ownable();
        let owner = Address::generate(&env);
        let other = Address::generate(&env);
        env.as_contract(&id, || {
            ownable.initialize(&env, &owner).unwrap();
            assert_eq(
                ownable.require_owner(&env, &other),
                Err(StandardsError::Unauthorized),
            );
        });
    }

    #[test]
    fn transfer_ownership_moves_owner_and_clears_pending() {
        let (env, id) = setup();
        let ownable = ownable();
        let owner = Address::generate(&env);
        let new_owner = Address::generate(&env);
        env.as_contract(&id, || {
            ownable.initialize(&env, &owner).unwrap();
            let event = ownable.transfer_ownership(&env, &owner, &new_owner).unwrap();
            assert_eq(event.previous_owner, Some(owner.clone()));
            assert_eq(event.new_owner, Some(new_owner.clone()));
            assert_eq(ownable.owner(&env), Some(new_owner));
            assert_eq(ownable.pending_owner(&env), None);
        });
    }

    #[test]
    fn transfer_ownership_rejects_non_owner() {
        let (env, id) = setup();
        let ownable = ownable();
        let owner = Address::generate(&env);
        let other = Address::generate(&env);
        env.as_contract(&id, || {
            ownable.initialize(&env, &owner).unwrap();
            assert_eq(
                ownable.transfer_ownership(&env, &other, &other),
                Err(StandardsError::Unauthorized),
            );
        });
    }

    #[test]
    fn renounce_ownership_clears_owner_and_pending() {
        let (env, id) = setup();
        let ownable = ownable();
        let owner = Address::generate(&env);
        env.as_contract(&id, || {
            ownable.initialize(&env, &owner).unwrap();
            let event = ownable.renounce_ownership(&env, &owner).unwrap();
            assert_eq(event.previous_owner, Some(owner.clone()));
            assert_eq(event.new_owner, None);
            assert_eq(ownable.owner(&env), None);
            assert_eq(ownable.pending_owner(&env), None);
        });
    }

    fn: ownable_2step() -> Ownable2Step {
        Ownable2Step::new(Symbol::new(&Env::default(), "ownable2step"))
    }

    #[test]
    fn begin_transfer_sets_pending_owner() {
        let (env, id) = setup();
        let ownable = ownable_2step();
        let owner = Address::generate(&env);
        let pending = Address::generate(&env);
        env.as_contract(&id, || {
            ownable.initialize(&env, &owner).unwrap();
            let event = ownable.begin_transfer(&env, &owner, &pending).unwrap();
            assert_eq(event.owner, owner.clone());
            assert_eq(event.pending_owner, pending.clone());
            assert_eq(ownable.pending_owner(&env), Some(pending));
        });
    }

    #[test]
    fn begin_transfer_rejects_non_owner() {
        let (env, id) = setup();
        let ownable = ownable_2step();
        let owner = Address::generate(&env);
        let other = Address::generate(&env);
        env.as_contract(&id, || {
            ownable.initialize(&env, &owner).unwrap();
            assert_eq(
                ownable.begin_transfer(&env, &other, &other),
                Err(StandardsError::Unauthorized),
            );
        });
    }

    #[test]
    fn accept_transfer_requires_pending_owner() {
        let (env, id) = setup();
        let ownable = ownable_2step();
        let owner = Address::generate(&env);
        env.as_contract(&id, || {
            ownable.initialize(&env, &owner).unwrap();
            assert_eq(
                ownable.accept_transfer(&env, &owner),
                Err(StandardsError::PendingOwnerNotSet),
            );
        });
    }

    #[test]
    fn accept_transfer_rejects_mismatched_caller() {
        let (env, id) = setup();
        let ownable = ownable_2step();
        let owner = Address::generate(&env);
        let pending = Address::generate(&env);
        let other = Address::generate(&env);
        env.as_contract(&id, || {
            ownable.initialize(&env, &owner).unwrap();
            ownable.begin_transfer(&env, &owner, &pending).unwrap();
            assert_eq(
                ownable.accept_transfer(&env, &other),
                Err(StandardsError::PendingOwnerMismatch),
            );
        });
    }

    #[test]
    fn accept_transfer_moves_ownership() {
        let (env, id) = setup();
        let ownable = ownable_2step();
        let owner = Address::generate(&env);
        let pending = Address::generate(&env);
        env.as_contract(&id, || {
            ownable.initialize(&env, &owner).unwrap();
            ownable.begin_transfer(&env, &owner, &pending).unwrap();
            let event = ownable.accept_transfer(&env, &pending).unwrap();
            assert_eq(event.previous_owner, Some(owner.clone()));
            assert_eq(event.new_owner, Some(pending.clone()));
            assert_eq(ownable.owner(&env), Some(pending.clone()));
            assert_eq(ownable.pending_owner(&env), None);
        });
    }

    #[test]
    fn cancel_transfer_clears_pending() {
        let (env, id) = setup();
        let ownable = ownable_2step();
        let owner = Address::generate(&env);
        let pending = Address::generate(&env);
        env.as_contract(&id, || {
            ownable.initialize(&env, &owner).unwrap();
            ownable.begin_transfer(&env, &owner, &pending).unwrap();
            let event = ownable.cancel_transfer(&env, &owner).unwrap();
            assert_eq(event.owner, owner.clone());
            assert_eq(event.pending_owner, pending.clone());
            assert_eq(ownable.pending_owner(&env), None);
            assert_eq(ownable.owner(&env), Some(owner));
        });
    }

    #[test]
    fn cancel_transfer_requires_pending_owner() {
        let (env, id) = setup();
        let ownable = ownable_2step();
        let owner = Address::generate(&env);
        env.as_contract(&id, || {
            ownable.initialize(&env, &owner).unwrap();
            assert_eq(
                ownable.cancel_transfer(&env, &owner),
                Err(StandardsError::PendingOwnerNotSet),
            );
        });
    }

    #[test]
    fn cancel_transfer_rejects_non_owner() {
        let (env, id) = setup();
        let ownable = ownable_2step();
        let owner = Address::generate(&env);
        let other = Address::generate(&env);
        env.as_contract(&id, || {
            ownable.initialize(&env, &owner).unwrap();
            ownable.begin_transfer(&env, &owner, &other).unwrap();
            assert_eq(
                ownable.cancel_transfer(&env, &other),
                Err(StandardsError::Unauthorized),
            );
        });
    }
}
