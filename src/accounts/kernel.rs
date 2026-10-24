use soroban_sdk::{Address, Env};

use super::error::AccountError;
use super::intent::{AuthMethod, AuthResult, IntentSigner, SignedIntent};
use super::policy::{IntentContext, IntentExpiryPolicy, Policy, SessionContext, SessionPolicy};
use super::replay::ReplayProtection;
use super::signer::{AccountSigner, DirectAuthSigner, Secp256r1PasskeySigner, SessionAuthSigner};
use super::storage::SessionStorage;

/// Account kernel that separates signer verification, policy evaluation and replay protection.
pub struct AccountKernel {
    owner: Address,
}

impl AccountKernel {
    pub fn new(owner: Address) -> Self {
        Self { owner }
    }

    pub fn owner(&self) -> &Address {
        &self.owner
    }

    pub fn authorize_direct(
        &self,
        env: &Env,
        intent: &SignedIntent,
    ) -> Result<AuthResult, AccountError> {
        self.ensure_target(intent)?;
        self.ensure_hash(env, intent)?;

        let signer = DirectAuthSigner;
        signer.verify(env, &self.owner, intent)?;

        let policy = IntentExpiryPolicy;
        policy.evaluate(
            env,
            &IntentContext {
                account: &self.owner,
                intent,
            },
        )?;

        let consumed =
            ReplayProtection::verify_and_consume_account_nonce(env, &self.owner, intent.nonce)?;

        Ok(AuthResult {
            method: AuthMethod::Direct,
            nonce_consumed: consumed,
            session_key_id: zero_key(env),
            remaining_operations: 0,
        })
    }

    pub fn authorize_session(
        &self,
        env: &Env,
        intent: &SignedIntent,
    ) -> Result<AuthResult, AccountError> {
        self.ensure_target(intent)?;
        self.ensure_hash(env, intent)?;

        // Signer verification runs first: the delegate must authorize this exact
        // intent (account, key id, nonce, action hash) before any policy check
        // or budget consumption touches session state.
        let signer = SessionAuthSigner;
        signer.verify(env, &self.owner, intent)?;

        let policy = SessionPolicy;
        policy.evaluate(
            env,
            &SessionContext {
                account: &self.owner,
                intent,
            },
        )?;

        let updated = SessionStorage::consume_authorized_session(
            env,
            &self.owner,
            &intent.signer.session_key_id,
        )?;

        Ok(AuthResult {
            method: AuthMethod::Session,
            nonce_consumed: updated.next_nonce - 1,
            session_key_id: updated.key_id,
            remaining_operations: updated.scope.max_operations - updated.operations_used,
        })
    }

    pub fn authorize_passkey(
        &self,
        env: &Env,
        intent: &SignedIntent,
    ) -> Result<AuthResult, AccountError> {
        self.ensure_target(intent)?;
        self.ensure_hash(env, intent)?;

        let signer = Secp256r1PasskeySigner;
        signer.verify(env, &self.owner, intent)?;

        let policy = IntentExpiryPolicy;
        policy.evaluate(
            env,
            &IntentContext {
                account: &self.owner,
                intent,
            },
        )?;

        let consumed =
            ReplayProtection::verify_and_consume_account_nonce(env, &self.owner, intent.nonce)?;

        Ok(AuthResult {
            method: AuthMethod::Passkey,
            nonce_consumed: consumed,
            session_key_id: zero_key(env),
            remaining_operations: 0,
        })
    }

    pub fn authorize(&self, env: &Env, intent: &SignedIntent) -> Result<AuthResult, AccountError> {
        match intent.signer.kind {
            IntentSigner::Direct => self.authorize_direct(env, intent),
            IntentSigner::Session => self.authorize_session(env, intent),
            IntentSigner::Passkey => self.authorize_passkey(env, intent),
        }
    }

    fn ensure_target(&self, intent: &SignedIntent) -> Result<(), AccountError> {
        if intent.account != self.owner {
            return Err(AccountError::Unauthorized);
        }
        Ok(())
    }

    fn ensure_hash(&self, env: &Env, intent: &SignedIntent) -> Result<(), AccountError> {
        if intent.recompute_hash(env) != intent.action_hash {
            return Err(AccountError::InvalidIntent);
        }
        Ok(())
    }
}

fn zero_key(env: &Env) -> soroban_sdk::BytesN<32> {
    soroban_sdk::BytesN::from_array(env, &[0u8; 32])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::intent::SignedIntent;
    use crate::accounts::multi_device::{DeviceManager, DevicePolicy, MultiDeviceProvider};
    use crate::accounts::policy::{
        ActiveDevicePolicy, DeviceContext, GuardianPolicy, RecoveryContext,
    };
    use crate::accounts::recovery::{RecoverableAccount, RecoveryConfig, RecoveryProvider};
    use crate::accounts::storage::SessionStorage;
    use crate::accounts::types::{GameAction, SessionKey, SessionScope};
    use soroban_sdk::{
        contract, contractimpl, symbol_short, testutils::Address as _, vec, Address, Bytes, BytesN,
        Env,
    };

    #[contract]
    pub struct TestContract;

    #[contractimpl]
    impl TestContract {}

    fn make_action(env: &Env, name: &str) -> GameAction {
        GameAction {
            system_name: soroban_sdk::Symbol::new(env, name),
            data: Bytes::new(env),
        }
    }

    #[test]
    fn test_direct_intent_consumes_account_nonce() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TestContract, ());
        let owner = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let kernel = AccountKernel::new(owner.clone());
            let action = make_action(&env, "move");
            let intent = SignedIntent::direct(&env, owner, action, 0, 99999);

            let result = kernel.authorize(&env, &intent).unwrap();
            assert_eq!(result.method, AuthMethod::Direct);
            assert_eq!(
                ReplayProtection::next_account_nonce(&env, kernel.owner()),
                1
            );
        });
    }

    #[test]
    fn test_session_intent_enforces_scope_budget_and_nonce() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let owner = Address::generate(&env);
        let key_id = BytesN::from_array(&env, &[7u8; 32]);

        env.as_contract(&contract_id, || {
            let kernel = AccountKernel::new(owner.clone());
            let session = SessionKey {
                key_id: key_id.clone(),
                scope: SessionScope {
                    allowed_actions: vec![&env, symbol_short!("move")],
                    max_operations: 2,
                    expires_at: 99999,
                },
                created_at: 0,
                operations_used: 0,
                next_nonce: 0,
            };
            SessionStorage::store(&env, &owner, &session);

            let move_1 = SignedIntent::session(
                &env,
                owner.clone(),
                &key_id,
                make_action(&env, "move"),
                0,
                99999,
            );
            let result_1 = kernel.authorize(&env, &move_1).unwrap();
            assert_eq!(result_1.remaining_operations, 1);

            let move_2 = SignedIntent::session(
                &env,
                owner.clone(),
                &key_id,
                make_action(&env, "move"),
                1,
                99999,
            );
            let result_2 = kernel.authorize(&env, &move_2).unwrap();
            assert_eq!(result_2.remaining_operations, 0);

            let replay = SignedIntent::session(
                &env,
                owner.clone(),
                &key_id,
                make_action(&env, "move"),
                1,
                99999,
            );
            assert_eq!(
                kernel.authorize(&env, &replay),
                Err(AccountError::SessionBudgetExceeded)
            );
        });
    }

    #[test]
    fn test_session_intent_rejects_wrong_nonce_before_budget_consumption() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let owner = Address::generate(&env);
        let key_id = BytesN::from_array(&env, &[8u8; 32]);

        env.as_contract(&contract_id, || {
            let kernel = AccountKernel::new(owner.clone());
            let session = SessionKey {
                key_id: key_id.clone(),
                scope: SessionScope {
                    allowed_actions: vec![&env, symbol_short!("move")],
                    max_operations: 3,
                    expires_at: 99999,
                },
                created_at: 0,
                operations_used: 0,
                next_nonce: 2,
            };
            SessionStorage::store(&env, &owner, &session);

            let wrong_nonce =
                SignedIntent::session(&env, owner, &key_id, make_action(&env, "move"), 1, 99999);
            assert_eq!(
                kernel.authorize(&env, &wrong_nonce),
                Err(AccountError::NonceMismatch)
            );
        });
    }

    #[test]
    fn test_device_and_guardian_policies_share_same_policy_model() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let owner = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let mut devices = DeviceManager::new(
                owner.clone(),
                DevicePolicy {
                    max_devices: 2,
                    auto_revoke_after: 0,
                },
                &env,
            );
            let device_key = BytesN::from_array(&env, &[5u8; 32]);
            devices
                .register_device(&env, device_key.clone(), symbol_short!("phone"))
                .unwrap();

            let mut recovery = RecoverableAccount::new(
                owner.clone(),
                RecoveryConfig {
                    threshold: 1,
                    timelock_period: 0,
                    max_guardians: 2,
                },
                &env,
            );
            let guardian = Address::generate(&env);
            recovery.add_guardian(&env, guardian.clone()).unwrap();

            let device_policy = ActiveDevicePolicy;
            device_policy
                .evaluate(
                    &env,
                    &DeviceContext {
                        account: &owner,
                        key_id: &device_key,
                    },
                )
                .unwrap();

            let guardian_policy = GuardianPolicy;
            guardian_policy
                .evaluate(
                    &env,
                    &RecoveryContext {
                        account: &owner,
                        guardian: &guardian,
                    },
                )
                .unwrap();
        });
    }
}
