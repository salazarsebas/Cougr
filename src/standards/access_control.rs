use soroban_sdk::{contracttype, Address, Env, Symbol};

use super::error::StandardsError;

const ROLE_MEMBER_PREFIX: &str = "std_role_m";
const ROLE_ADMIN_PREFIX: &str = "std_role_a";

pub const DEFAULT_ADMIN_ROLE_NAME: &str = "DEFAULT_ADMIN_ROLE";

/// Role-based access control with per-role admin delegation.
#[derive(Clone, Debug)]
pub struct AccessControl {
    id: Symbol,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleGrantedEvent {
    pub role: Symbol,
    pub account: Address,
    pub sender: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleRevokedEvent {
    pub role: Symbol,
    pub account: Address,
    pub sender: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleAdminChangedEvent {
    pub role: Symbol,
    pub previous_admin_role: Symbol,
    pub new_admin_role: Symbol,
    pub sender: Address,
}

impl AccessControl {
    pub fn new(id: Symbol) -> Self {
        Self { id }
    }

    pub fn initialize(
        &self,
        env: &Env,
        admin: &Address,
    ) -> Result<RoleGrantedEvent, StandardsError> {
        let default_admin_role = self.default_admin_role(env);
        let admin_key = self.role_admin_key(env, &default_admin_role);
        if env.storage().persistent().has(&admin_key) {
            return Err(StandardsError::AlreadyInitialized);
        }

        env.storage()
            .persistent()
            .set(&admin_key, &default_admin_role);
        self.grant_role_unchecked(env, &default_admin_role, admin);

        Ok(RoleGrantedEvent {
            role: default_admin_role,
            account: admin.clone(),
            sender: admin.clone(),
        })
    }

    pub fn default_admin_role(&self, env: &Env) -> Symbol {
        Symbol::new(env, DEFAULT_ADMIN_ROLE_NAME)
    }

    pub fn has_role(&self, env: &Env, role: &Symbol, account: &Address) -> bool {
        env.storage()
            .persistent()
            .has(&self.role_member_key(env, role, account))
    }

    pub fn require_role(
        &self,
        env: &Env,
        role: &Symbol,
        account: &Address,
    ) -> Result<(), StandardsError> {
        if self.has_role(env, role, account) {
            return Ok(());
        }
        Err(StandardsError::Unauthorized)
    }

    pub fn role_admin(&self, env: &Env, role: &Symbol) -> Symbol {
        env.storage()
            .persistent()
            .get(&self.role_admin_key(env, role))
            .unwrap_or_else(|| self.default_admin_role(env))
    }

    pub fn grant_role(
        &self,
        env: &Env,
        caller: &Address,
        role: &Symbol,
        account: &Address,
    ) -> Result<RoleGrantedEvent, StandardsError> {
        let admin_role = self.role_admin(env, role);
        self.require_role(env, &admin_role, caller)?;
        if self.has_role(env, role, account) {
            return Err(StandardsError::RoleAlreadyGranted);
        }

        self.grant_role_unchecked(env, role, account);
        Ok(RoleGrantedEvent {
            role: role.clone(),
            account: account.clone(),
            sender: caller.clone(),
        })
    }

    pub fn revoke_role(
        &self,
        env: &Env,
        caller: &Address,
        role: &Symbol,
        account: &Address,
    ) -> Result<RoleRevokedEvent, StandardsError> {
        let admin_role = self.role_admin(env, role);
        self.require_role(env, &admin_role, caller)?;
        if !self.has_role(env, role, account) {
            return Err(StandardsError::RoleNotGranted);
        }

        env.storage()
            .persistent()
            .remove(&self.role_member_key(env, role, account));

        Ok(RoleRevokedEvent {
            role: role.clone(),
            account: account.clone(),
            sender: caller.clone(),
        })
    }

    pub fn renounce_role(
        &self,
        env: &Env,
        role: &Symbol,
        caller: &Address,
    ) -> Result<RoleRevokedEvent, StandardsError> {
        if !self.has_role(env, role, caller) {
            return Err(StandardsError::RoleNotGranted);
        }

        env.storage()
            .persistent()
            .remove(&self.role_member_key(env, role, caller));

        Ok(RoleRevokedEvent {
            role: role.clone(),
            account: caller.clone(),
            sender: caller.clone(),
        })
    }

    pub fn set_role_admin(
        &self,
        env: &Env,
        caller: &Address,
        role: &Symbol,
        new_admin_role: &Symbol,
    ) -> Result<RoleAdminChangedEvent, StandardsError> {
        let previous_admin_role = self.role_admin(env, role);
        self.require_role(env, &previous_admin_role, caller)?;

        env.storage()
            .persistent()
            .set(&self.role_admin_key(env, role), new_admin_role);

        Ok(RoleAdminChangedEvent {
            role: role.clone(),
            previous_admin_role,
            new_admin_role: new_admin_role.clone(),
            sender: caller.clone(),
        })
    }

    fn grant_role_unchecked(&self, env: &Env, role: &Symbol, account: &Address) {
        env.storage()
            .persistent()
            .set(&self.role_member_key(env, role, account), &true);
    }

    fn role_member_key(
        &self,
        env: &Env,
        role: &Symbol,
        account: &Address,
    ) -> (Symbol, Symbol, Symbol, Address) {
        (
            Symbol::new(env, ROLE_MEMBER_PREFIX),
            self.id.clone(),
            role.clone(),
            account.clone(),
        )
    }

    fn role_admin_key(&self, env: &Env, role: &Symbol) -> (Symbol, Symbol, Symbol) {
        (
            Symbol::new(env, ROLE_ADMIN_PREFIX),
            self.id.clone(),
            role.clone(),
        )
    }
}
