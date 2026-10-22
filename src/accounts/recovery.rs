use soroban_sdk::{contracttype, Address, Env, Vec};

use super::error::AccountError;
use super::recovery_storage::RecoveryStorage;

/// A guardian that can participate in account recovery.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Guardian {
    /// The guardian's Stellar address.
    pub address: Address,
    /// Ledger timestamp when the guardian was added.
    pub added_at: u64,
}

/// A pending account recovery request.
#[contracttype]
#[derive(Clone, Debug)]
pub struct RecoveryRequest {
    /// The proposed new owner address.
    pub new_owner: Address,
    /// Addresses of guardians that have approved so far.
    pub approvals: Vec<Address>,
    /// Ledger timestamp when recovery was initiated.
    pub initiated_at: u64,
    /// Earliest ledger timestamp when recovery can be executed.
    pub timelock_until: u64,
    /// Whether this request has been cancelled.
    pub cancelled: bool,
}

/// Configuration for the recovery mechanism.
#[contracttype]
#[derive(Clone, Debug)]
pub struct RecoveryConfig {
    /// Number of guardians required to approve a recovery.
    pub threshold: u32,
    /// Ledger duration to wait after threshold is met before execution.
    pub timelock_period: u64,
    /// Maximum number of guardians allowed.
    pub max_guardians: u32,
}

/// Trait for account types that support social recovery.
pub trait RecoveryProvider {
    /// Add a guardian to the account.
    fn add_guardian(&mut self, env: &Env, guardian: Address) -> Result<(), AccountError>;

    /// Remove a guardian from the account.
    fn remove_guardian(&mut self, env: &Env, guardian: &Address) -> Result<(), AccountError>;

    /// Initiate account recovery to a new owner.
    fn initiate_recovery(
        &mut self,
        env: &Env,
        new_owner: Address,
    ) -> Result<RecoveryRequest, AccountError>;

    /// Approve an active recovery request as a guardian.
    fn approve_recovery(&mut self, env: &Env, guardian: &Address) -> Result<(), AccountError>;

    /// Cancel the active recovery request.
    fn cancel_recovery(&mut self, env: &Env) -> Result<(), AccountError>;

    /// Execute recovery after timelock has passed and threshold is met.
    /// Returns the new owner address.
    fn execute_recovery(&mut self, env: &Env) -> Result<Address, AccountError>;

    /// Returns the number of guardians.
    fn guardian_count(&self, env: &Env) -> usize;

    /// Returns the recovery configuration.
    fn recovery_config(&self, env: &Env) -> RecoveryConfig;
}

/// Persistent implementation of `RecoveryProvider`.
///
/// Guardians, config, and recovery state are stored in Soroban persistent
/// storage via [`RecoveryStorage`], surviving across contract invocations.
pub struct RecoverableAccount {
    address: Address,
}

impl RecoverableAccount {
    /// Create a new recoverable account and persist the initial config.
    ///
    /// Returns `AccountError::InvalidThreshold` if `threshold` is zero or
    /// greater than `max_guardians`.
    pub fn new(
        address: Address,
        config: RecoveryConfig,
        env: &Env,
    ) -> Result<Self, AccountError> {
        if config.threshold == 0 || config.threshold > config.max_guardians {
            return Err(AccountError::InvalidThreshold);
        }
        RecoveryStorage::store_config(env, &address, &config);
        RecoveryStorage::store_guardians(env, &address, &Vec::new(env));
        Ok(Self { address })
    }

    /// Load an existing recoverable account (config must already be stored).
    pub fn load(address: Address) -> Self {
        Self { address }
    }

    /// Returns the account address.
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the active recovery request, if any.
    pub fn active_request(&self, env: &Env) -> Option<RecoveryRequest> {
        RecoveryStorage::load_request(env, &self.address)
    }
}

impl RecoveryProvider for RecoverableAccount {
    fn add_guardian(&mut self, env: &Env, guardian: Address) -> Result<(), AccountError> {
        let config =
            RecoveryStorage::load_config(env, &self.address).ok_or(AccountError::StorageError)?;
        let guardians = RecoveryStorage::load_guardians(env, &self.address);

        // Check max guardians
        if guardians.len() >= config.max_guardians {
            return Err(AccountError::MaxGuardiansReached);
        }

        // Check for duplicates
        for i in 0..guardians.len() {
            if let Some(g) = guardians.get(i) {
                if g.address == guardian {
                    return Err(AccountError::GuardianAlreadyExists);
                }
            }
        }

        let mut new_guardians = guardians;
        new_guardians.push_back(Guardian {
            address: guardian,
            added_at: env.ledger().timestamp(),
        });
        RecoveryStorage::store_guardians(env, &self.address, &new_guardians);
        Ok(())
    }

    fn remove_guardian(&mut self, env: &Env, guardian: &Address) -> Result<(), AccountError> {
        let config =
            RecoveryStorage::load_config(env, &self.address).ok_or(AccountError::StorageError)?;
        let guardians = RecoveryStorage::load_guardians(env, &self.address);
        let mut new_guardians: Vec<Guardian> = Vec::new(env);
        let mut found = false;

        for i in 0..guardians.len() {
            if let Some(g) = guardians.get(i) {
                if &g.address == guardian {
                    found = true;
                } else {
                    new_guardians.push_back(g);
                }
            }
        }

        if !found {
            return Err(AccountError::InvalidScope);
        }

        if (new_guardians.len() as u32) < config.threshold {
            return Err(AccountError::ThresholdNotMet);
        }

        RecoveryStorage::store_guardians(env, &self.address, &new_guardians);
        Ok(())
    }

    fn initiate_recovery(
        &mut self,
        env: &Env,
        new_owner: Address,
    ) -> Result<RecoveryRequest, AccountError> {
        if RecoveryStorage::load_request(env, &self.address).is_some() {
            return Err(AccountError::RecoveryAlreadyActive);
        }

        let config =
            RecoveryStorage::load_config(env, &self.address).ok_or(AccountError::StorageError)?;

        let now = env.ledger().timestamp();
        let request = RecoveryRequest {
            new_owner,
            approvals: Vec::new(env),
            initiated_at: now,
            timelock_until: now + config.timelock_period,
            cancelled: false,
        };
        RecoveryStorage::store_request(env, &self.address, &request);
        Ok(request)
    }

    fn approve_recovery(&mut self, env: &Env, guardian: &Address) -> Result<(), AccountError> {
        let mut request = RecoveryStorage::load_request(env, &self.address)
            .ok_or(AccountError::RecoveryNotInitiated)?;

        if request.cancelled {
            return Err(AccountError::RecoveryNotInitiated);
        }

        // Verify guardian is valid
        let guardians = RecoveryStorage::load_guardians(env, &self.address);
        let mut is_guardian = false;
        for i in 0..guardians.len() {
            if let Some(g) = guardians.get(i) {
                if &g.address == guardian {
                    is_guardian = true;
                    break;
                }
            }
        }
        if !is_guardian {
            return Err(AccountError::Unauthorized);
        }

        // Check not already approved
        for i in 0..request.approvals.len() {
            if let Some(addr) = request.approvals.get(i) {
                if &addr == guardian {
                    return Ok(()); // already approved, idempotent
                }
            }
        }

        request.approvals.push_back(guardian.clone());
        RecoveryStorage::store_request(env, &self.address, &request);
        Ok(())
    }

    fn cancel_recovery(&mut self, env: &Env) -> Result<(), AccountError> {
        let request = RecoveryStorage::load_request(env, &self.address)
            .ok_or(AccountError::RecoveryNotInitiated)?;

        if request.cancelled {
            return Err(AccountError::RecoveryNotInitiated);
        }

        RecoveryStorage::remove_request(env, &self.address);
        Ok(())
    }

    fn execute_recovery(&mut self, env: &Env) -> Result<Address, AccountError> {
        let request = RecoveryStorage::load_request(env, &self.address)
            .ok_or(AccountError::RecoveryNotInitiated)?;

        if request.cancelled {
            return Err(AccountError::RecoveryNotInitiated);
        }

        // Check threshold
        let config =
            RecoveryStorage::load_config(env, &self.address).ok_or(AccountError::StorageError)?;
        if request.approvals.len() < config.threshold {
            return Err(AccountError::ThresholdNotMet);
        }

        // Check timelock
        let now = env.ledger().timestamp();
        if now < request.timelock_until {
            return Err(AccountError::TimelockNotExpired);
        }

        let new_owner = request.new_owner.clone();
        RecoveryStorage::remove_request(env, &self.address);
        self.address = new_owner.clone();
        Ok(new_owner)
    }

    fn guardian_count(&self, env: &Env) -> usize {
        RecoveryStorage::load_guardians(env, &self.address).len() as usize
    }

    fn recovery_config(&self, env: &Env) -> RecoveryConfig {
        RecoveryStorage::load_config(env, &self.address).unwrap_or(RecoveryConfig {
            threshold: 0,
            timelock_period: 0,
            max_guardians: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{contract, contractimpl, testutils::Address as _, Env};
    use soroban_sdk::testutils::Ledger as _;

    #[contract]
    pub struct TestContract;

    #[contractimpl]
    impl TestContract {}

    fn default_config() -> RecoveryConfig {
        RecoveryConfig {
            threshold: 2,
            timelock_period: 100,
            max_guardians: 5,
        }
    }

    #[test]
    fn test_add_guardian() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let guardian = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut account = RecoverableAccount::new(addr, default_config(), &env).unwrap();
            account.add_guardian(&env, guardian).unwrap();
            assert_eq!(account.guardian_count(&env), 1);
        });
    }

    #[test]
    fn test_add_duplicate_guardian() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let guardian = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut account = RecoverableAccount::new(addr, default_config(), &env).unwrap();
            account.add_guardian(&env, guardian.clone()).unwrap();
            let result = account.add_guardian(&env, guardian);
            assert_eq!(result, Err(AccountError::GuardianAlreadyExists));
        });
    }

    #[test]
    fn test_max_guardians() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let config = RecoveryConfig {
            threshold: 1,
            timelock_period: 0,
            max_guardians: 2,
        };

        env.as_contract(&contract_id, || {
            let mut account = RecoverableAccount::new(addr, config, &env).unwrap();
            account.add_guardian(&env, Address::generate(&env)).unwrap();
            account.add_guardian(&env, Address::generate(&env)).unwrap();
            let result = account.add_guardian(&env, Address::generate(&env));
            assert_eq!(result, Err(AccountError::MaxGuardiansReached));
        });
    }

    #[test]
    fn test_remove_guardian() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let guardian = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut account = RecoverableAccount::new(addr, default_config(), &env).unwrap();
            account.add_guardian(&env, guardian.clone()).unwrap();
            account.remove_guardian(&env, &guardian).unwrap();
            assert_eq!(account.guardian_count(&env), 0);
        });
    }

    #[test]
    fn test_initiate_recovery() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let new_owner = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut account = RecoverableAccount::new(addr, default_config(), &env).unwrap();
            let request = account.initiate_recovery(&env, new_owner).unwrap();
            assert!(!request.cancelled);
            assert_eq!(request.approvals.len(), 0);
        });
    }

    #[test]
    fn test_double_initiate_fails() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut account = RecoverableAccount::new(addr, default_config(), &env).unwrap();
            account
                .initiate_recovery(&env, Address::generate(&env))
                .unwrap();
            let result = account.initiate_recovery(&env, Address::generate(&env));
            assert!(matches!(result, Err(AccountError::RecoveryAlreadyActive)));
        });
    }

    #[test]
    fn test_approve_recovery() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let guardian1 = Address::generate(&env);
        let guardian2 = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut account = RecoverableAccount::new(addr, default_config(), &env).unwrap();
            account.add_guardian(&env, guardian1.clone()).unwrap();
            account.add_guardian(&env, guardian2.clone()).unwrap();
            account
                .initiate_recovery(&env, Address::generate(&env))
                .unwrap();

            account.approve_recovery(&env, &guardian1).unwrap();
            account.approve_recovery(&env, &guardian2).unwrap();

            let request = account.active_request(&env).unwrap();
            assert_eq!(request.approvals.len(), 2);
        });
    }

    #[test]
    fn test_approve_non_guardian_fails() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let non_guardian = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut account = RecoverableAccount::new(addr, default_config(), &env).unwrap();
            account
                .initiate_recovery(&env, Address::generate(&env))
                .unwrap();

            let result = account.approve_recovery(&env, &non_guardian);
            assert_eq!(result, Err(AccountError::Unauthorized));
        });
    }

    #[test]
    fn test_cancel_recovery() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut account = RecoverableAccount::new(addr, default_config(), &env).unwrap();
            account
                .initiate_recovery(&env, Address::generate(&env))
                .unwrap();
            account.cancel_recovery(&env).unwrap();
            assert!(account.active_request(&env).is_none());
        });
    }

    #[test]
    fn test_execute_recovery_threshold_not_met() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let guardian1 = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut account = RecoverableAccount::new(addr, default_config(), &env).unwrap();
            account.add_guardian(&env, guardian1.clone()).unwrap();
            account
                .initiate_recovery(&env, Address::generate(&env))
                .unwrap();
            account.approve_recovery(&env, &guardian1).unwrap();

            // Only 1 approval, threshold is 2
            let result = account.execute_recovery(&env);
            assert_eq!(result, Err(AccountError::ThresholdNotMet));
        });
    }

    #[test]
    fn test_execute_recovery_timelock() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let guardian1 = Address::generate(&env);
        let guardian2 = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut account = RecoverableAccount::new(addr, default_config(), &env).unwrap();
            account.add_guardian(&env, guardian1.clone()).unwrap();
            account.add_guardian(&env, guardian2.clone()).unwrap();
            account
                .initiate_recovery(&env, Address::generate(&env))
                .unwrap();
            account.approve_recovery(&env, &guardian1).unwrap();
            account.approve_recovery(&env, &guardian2).unwrap();

            // Timelock not expired (timestamp 0, timelock_until = 100)
            let result = account.execute_recovery(&env);
            assert_eq!(result, Err(AccountError::TimelockNotExpired));
        });
    }

    #[test]
    fn test_recovery_config() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let config = default_config();

        env.as_contract(&contract_id, || {
            let account = RecoverableAccount::new(addr, config, &env).unwrap();
            assert_eq!(account.recovery_config(&env).threshold, 2);
            assert_eq!(account.recovery_config(&env).timelock_period, 100);
            assert_eq!(account.recovery_config(&env).max_guardians, 5);
        });
    }

    #[test]
    fn test_new_rejects_zero_threshold() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let config = RecoveryConfig {
            threshold: 0,
            timelock_period: 100,
            max_guardians: 5,
        };

        env.as_contract(&contract_id, || {
            let result = RecoverableAccount::new(addr, config, &env);
            assert_eq!(result.err(), Some(AccountError::InvalidThreshold));
        });
    }

    #[test]
    fn test_new_rejects_threshold_above_max_guardians() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let config = RecoveryConfig {
            threshold: 6,
            timelock_period: 100,
            max_guardians: 5,
        };

        env.as_contract(&contract_id, || {
            let result = RecoverableAccount::new(addr, config, &env);
            assert_eq!(result.err(), Some(AccountError::InvalidThreshold));
        });
    }

    #[test]
    fn test_remove_guardian_cannot_go_below_threshold() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let guardian1 = Address::generate(&env);
        let guardian2 = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut account = RecoverableAccount::new(addr, default_config(), &env).unwrap();
            account.add_guardian(&env, guardian1.clone()).unwrap();
            account.add_guardian(&env, guardian2.clone()).unwrap();

            // Removing one guardian leaves 1, below threshold of 2.
            let result = account.remove_guardian(&env, &guardian1);
            assert_eq!(result, Err(AccountError::ThresholdNotMet));
            assert_eq!(account.guardian_count(&env), 2);
        });
    }

    #[test]
    fn test_execute_recovery_clears_request() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let addr = Address::generate(&env);
        let guardian1 = Address::generate(&env);
        let guardian2 = Address::generate(&env);
        let new_owner = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut account = RecoverableAccount::new(addr.clone(), default_config(), &env).unwrap();
            account.add_guardian(&env, guardian1.clone()).unwrap();
            account.add_guardian(&env, guardian2.clone()).unwrap();
            account.initiate_recovery(&env, new_owner.clone()).unwrap();
            account.approve_recovery(&env, &guardian1).unwrap();
            account.approve_recovery(&env, &guardian2).unwrap();

            env.ledger().set_timestamp(200);
            let executed = account.execute_recovery(&env).unwrap();
            assert_eq!(executed, new_owner);

            // Request under the original address must be gone.
            let old_account = RecoverableAccount::load(addr.clone());
            assert!(old_account.active_request(&env).is_none());

            // A second execute must fail.
            let result = account.execute_recovery(&env);
            assert_eq!(result, Err(AccountError::RecoveryNotInitiated));

            // The old account can initiate a new recovery.
            let mut old_account = RecoverableAccount::load(addr);
            assert!(old_account
                .initiate_recovery(&env, Address::generate(&env))
                .is_ok());
        });
    }
}
