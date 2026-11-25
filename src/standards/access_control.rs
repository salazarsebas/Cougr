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

    /// Initializes access control by granting `DEFAULT_ADMIN_ROLE` to `admin`.
    ///
    /// Note: The integrating contract is responsible for authenticating `admin` / `caller`.
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
    /// Note: The integrating contract is responsible for authenticating `caller`.
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
    /// Note: The integrating contract is responsible for authenticating `caller`.
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

    /// Allows `caller` to renounce `role` for themselves.
    ///
    /// Note: The integrating contract is responsible for authenticating `caller`.
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

    /// Updates the admin role controlling `role` to `new_admin_role`.
    ///
    /// Note: The integrating contract is responsible for authenticating `caller`.
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
    use soroban_sdk::{
        contract, contractimpl, symbol_short, testutils::Address as _, Address, Env,
    };

    use super::*;

    #[contract]
    pub struct TestContract;

    #[contractimpl]
    impl TestContract {}

    #[test]
    fn access_control_initialize_and_require_role_enforce_state() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let admin = Address::generate(&env);
        let outsider = Address::generate(&env);
        let module = AccessControl::new(symbol_short!("acl"));
        let default_admin = module.default_admin_role(&env);

        env.as_contract(&contract_id, || {
            assert!(!module.has_role(&env, &default_admin, &admin));
            assert_eq!(
                module.require_role(&env, &default_admin, &admin),
                Err(StandardsError::Unauthorized)
            );

            let granted = module.initialize(&env, &admin).unwrap();
            assert_eq!(
                granted,
                RoleGrantedEvent {
                    role: default_admin.clone(),
                    account: admin.clone(),
                    sender: admin.clone(),
                }
            );
            assert_eq!(
                module.initialize(&env, &admin),
                Err(StandardsError::AlreadyInitialized)
            );
            assert!(module.require_role(&env, &default_admin, &admin).is_ok());
            assert_eq!(
                module.require_role(&env, &default_admin, &outsider),
                Err(StandardsError::Unauthorized)
            );
        });
    }

    #[test]
    fn access_control_grant_revoke_renounce_and_admin_errors() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);
        let custom_admin = Address::generate(&env);
        let outsider = Address::generate(&env);
        let module = AccessControl::new(symbol_short!("acl"));
        let ops_role = symbol_short!("OPS");
        let ops_admin_role = symbol_short!("OPADM");

        env.as_contract(&contract_id, || {
            module.initialize(&env, &admin).unwrap();

            assert_eq!(
                module.grant_role(&env, &outsider, &ops_role, &operator),
                Err(StandardsError::Unauthorized)
            );
            module
                .grant_role(&env, &admin, &ops_role, &operator)
                .unwrap();
            assert_eq!(
                module.grant_role(&env, &admin, &ops_role, &operator),
                Err(StandardsError::RoleAlreadyGranted)
            );

            assert_eq!(
                module.revoke_role(&env, &outsider, &ops_role, &operator),
                Err(StandardsError::Unauthorized)
            );
            let revoked = module
                .revoke_role(&env, &admin, &ops_role, &operator)
                .unwrap();
            assert_eq!(
                revoked,
                RoleRevokedEvent {
                    role: ops_role.clone(),
                    account: operator.clone(),
                    sender: admin.clone(),
                }
            );
            assert_eq!(
                module.revoke_role(&env, &admin, &ops_role, &operator),
                Err(StandardsError::RoleNotGranted)
            );

            assert_eq!(
                module.renounce_role(&env, &ops_role, &operator),
                Err(StandardsError::RoleNotGranted)
            );
            module
                .grant_role(&env, &admin, &ops_role, &operator)
                .unwrap();
            let renounced = module.renounce_role(&env, &ops_role, &operator).unwrap();
            assert_eq!(
                renounced,
                RoleRevokedEvent {
                    role: ops_role.clone(),
                    account: operator.clone(),
                    sender: operator.clone(),
                }
            );
            assert!(!module.has_role(&env, &ops_role, &operator));

            assert_eq!(
                module.set_role_admin(&env, &outsider, &ops_role, &ops_admin_role),
                Err(StandardsError::Unauthorized)
            );
            let admin_changed = module
                .set_role_admin(&env, &admin, &ops_role, &ops_admin_role)
                .unwrap();
            assert_eq!(
                admin_changed,
                RoleAdminChangedEvent {
                    role: ops_role.clone(),
                    previous_admin_role: module.default_admin_role(&env),
                    new_admin_role: ops_admin_role.clone(),
                    sender: admin.clone(),
                }
            );
            assert_eq!(module.role_admin(&env, &ops_role), ops_admin_role);

            module
                .grant_role(&env, &admin, &ops_admin_role, &custom_admin)
                .unwrap();
            assert_eq!(
                module.grant_role(&env, &admin, &ops_role, &operator),
                Err(StandardsError::Unauthorized)
            );
            module
                .grant_role(&env, &custom_admin, &ops_role, &operator)
                .unwrap();
            assert!(module.has_role(&env, &ops_role, &operator));
        });
    }

    #[test]
    fn access_control_mutators_do_not_call_require_auth_internally() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);
        let module = AccessControl::new(symbol_short!("noauth"));
        let ops_role = symbol_short!("OPS");

        env.as_contract(&contract_id, || {
            module.initialize(&env, &admin).unwrap();
            module
                .grant_role(&env, &admin, &ops_role, &operator)
                .unwrap();
            module.renounce_role(&env, &ops_role, &operator).unwrap();
            assert_eq!(env.auths().len(), 0);
        });
    }
}
