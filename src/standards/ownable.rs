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
