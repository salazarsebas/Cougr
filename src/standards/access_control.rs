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

    /// Initializes the module and grants [`DEFAULT_ADMIN_ROLE_NAME`] to `admin`.
    ///
    /// Note: This helper does not call `admin.require_auth()`; the integrating
    /// contract must authenticate the caller before invoking this method.
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

    /// Grants `role` to `account` when `caller` holds the admin role for `role`.
    ///
    /// Note: This helper checks `caller`'s role membership in storage but does
    /// not call `caller.require_auth()`; the integrating contract must
    /// authenticate `caller` before invoking this method.
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

    /// Revokes `role` from `account` when `caller` holds the admin role for `role`.
    ///
    /// Note: This helper checks `caller`'s role membership in storage but does
    /// not call `caller.require_auth()`; the integrating contract must
    /// authenticate `caller` before invoking this method.
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

    /// Renounces `role` for `caller`.
    ///
    /// Note: This helper checks `caller`'s role membership in storage but does
    /// not call `caller.require_auth()`; the integrating contract must
    /// authenticate `caller` before invoking this method.
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

    /// Updates the admin role that governs `role` to `new_admin_role`.
    ///
    /// Note: This helper checks `caller`'s admin role in storage but does not
    /// call `caller.require_auth()`; the integrating contract must
    /// authenticate `caller` before invoking this method.
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

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{contract, contractimpl, symbol_short, testutils::Address as _};

    #[contract]
    struct DummyContract;

    #[contractimpl]
    impl DummyContract {}

    #[test]
    fn initialize_grants_default_admin_and_rejects_duplicate_init() {
        let env = Env::default();
        let contract_id = env.register(DummyContract, ());
        let admin = Address::generate(&env);
        let other = Address::generate(&env);
        let acl = AccessControl::new(symbol_short!("acl"));

        env.as_contract(&contract_id, || {
            let default_admin = acl.default_admin_role(&env);
            assert!(!acl.has_role(&env, &default_admin, &admin));

            let event = acl.initialize(&env, &admin).unwrap();
            assert_eq!(
                event,
                RoleGrantedEvent {
                    role: default_admin.clone(),
                    account: admin.clone(),
                    sender: admin.clone(),
                }
            );
            assert!(acl.has_role(&env, &default_admin, &admin));
            assert_eq!(acl.require_role(&env, &default_admin, &admin), Ok(()));
            assert_eq!(
                acl.require_role(&env, &default_admin, &other),
                Err(StandardsError::Unauthorized)
            );
            assert_eq!(
                acl.initialize(&env, &other),
                Err(StandardsError::AlreadyInitialized)
            );
        });
    }

    #[test]
    fn grant_and_revoke_role_enforce_admin_and_membership_errors() {
        let env = Env::default();
        let contract_id = env.register(DummyContract, ());
        let admin = Address::generate(&env);
        let member = Address::generate(&env);
        let outsider = Address::generate(&env);
        let acl = AccessControl::new(symbol_short!("acl"));
        let ops_role = symbol_short!("OPS");

        env.as_contract(&contract_id, || {
            acl.initialize(&env, &admin).unwrap();

            assert_eq!(
                acl.grant_role(&env, &outsider, &ops_role, &member),
                Err(StandardsError::Unauthorized)
            );

            let granted = acl.grant_role(&env, &admin, &ops_role, &member).unwrap();
            assert_eq!(
                granted,
                RoleGrantedEvent {
                    role: ops_role.clone(),
                    account: member.clone(),
                    sender: admin.clone(),
                }
            );
            assert!(acl.has_role(&env, &ops_role, &member));

            assert_eq!(
                acl.grant_role(&env, &admin, &ops_role, &member),
                Err(StandardsError::RoleAlreadyGranted)
            );

            assert_eq!(
                acl.revoke_role(&env, &outsider, &ops_role, &member),
                Err(StandardsError::Unauthorized)
            );

            let revoked = acl.revoke_role(&env, &admin, &ops_role, &member).unwrap();
            assert_eq!(
                revoked,
                RoleRevokedEvent {
                    role: ops_role.clone(),
                    account: member.clone(),
                    sender: admin.clone(),
                }
            );
            assert!(!acl.has_role(&env, &ops_role, &member));

            assert_eq!(
                acl.revoke_role(&env, &admin, &ops_role, &member),
                Err(StandardsError::RoleNotGranted)
            );
        });
    }

    #[test]
    fn renounce_role_and_set_role_admin_cover_success_and_errors() {
        let env = Env::default();
        let contract_id = env.register(DummyContract, ());
        let admin = Address::generate(&env);
        let delegated_admin = Address::generate(&env);
        let member = Address::generate(&env);
        let acl = AccessControl::new(symbol_short!("acl"));
        let ops_role = symbol_short!("OPS");
        let ops_admin_role = symbol_short!("OPADM");

        env.as_contract(&contract_id, || {
            acl.initialize(&env, &admin).unwrap();
            let default_admin = acl.default_admin_role(&env);
            assert_eq!(acl.role_admin(&env, &ops_role), default_admin);

            assert_eq!(
                acl.renounce_role(&env, &ops_role, &member),
                Err(StandardsError::RoleNotGranted)
            );
            assert_eq!(
                acl.set_role_admin(&env, &member, &ops_role, &ops_admin_role),
                Err(StandardsError::Unauthorized)
            );

            let admin_changed = acl
                .set_role_admin(&env, &admin, &ops_role, &ops_admin_role)
                .unwrap();
            assert_eq!(
                admin_changed,
                RoleAdminChangedEvent {
                    role: ops_role.clone(),
                    previous_admin_role: default_admin,
                    new_admin_role: ops_admin_role.clone(),
                    sender: admin.clone(),
                }
            );
            assert_eq!(acl.role_admin(&env, &ops_role), ops_admin_role);

            acl.grant_role(&env, &admin, &ops_admin_role, &delegated_admin)
                .unwrap();
            acl.grant_role(&env, &delegated_admin, &ops_role, &member)
                .unwrap();
            assert!(acl.has_role(&env, &ops_role, &member));

            let renounced = acl.renounce_role(&env, &ops_role, &member).unwrap();
            assert_eq!(
                renounced,
                RoleRevokedEvent {
                    role: ops_role.clone(),
                    account: member.clone(),
                    sender: member.clone(),
                }
            );
            assert!(!acl.has_role(&env, &ops_role, &member));
        });
    }
}
