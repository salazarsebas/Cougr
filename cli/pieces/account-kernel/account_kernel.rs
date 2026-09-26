use cougr_core::accounts::{
    AccountError, AccountKernel, GameAction, SessionBuilder, SessionKey, SessionScope,
};
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Bytes, BytesN, Env, Symbol};

/// The only action this sample piece authorizes by default.
pub fn sample_action_name() -> Symbol {
    symbol_short!("tap")
}

/// Counter action used to prove authorization flow end-to-end.
pub fn counter_action(env: &Env) -> GameAction {
    GameAction {
        system_name: sample_action_name(),
        data: Bytes::new(env),
    }
}

/// Build a session scope permitting exactly one action for `max_ops` calls
/// before `expires_at` (an absolute ledger timestamp).
pub fn build_scope(env: &Env, max_ops: u32, expires_at: u64) -> SessionScope {
    SessionBuilder::new(env)
        .allow_action(sample_action_name())
        .max_operations(max_ops)
        .expires_at(expires_at)
        .build_scope()
}

/// Reject if `session.scope.expires_at` has passed.
pub fn check_session_expired(env: &Env, session: &SessionScope) -> bool {
    env.ledger().timestamp() >= session.expires_at
}

/// Authorize a session action against the live kernel.
pub fn authorize_action(
    env: &Env,
    kernel: &AccountKernel,
    account: &Address,
    key_id: &BytesN<32>,
    action: GameAction,
    nonce: u64,
    expires_at: u64,
) -> Result<(), AccountError> {
    let intent = cougr_core::accounts::SignedIntent::session(
        env,
        account.clone(),
        key_id,
        action,
        nonce,
        expires_at,
    );
    kernel.authorize_session(env, &intent)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cougr_core::accounts::SessionStorage;
    use soroban_sdk::{contract, contractimpl};

    #[contract]
    #[derive(Clone)]
    pub struct TestContract;

    #[contractimpl]
    impl TestContract {}

    fn make_session_key(env: &Env, key_id: BytesN<32>, scope: SessionScope) -> SessionKey {
        SessionKey {
            key_id,
            scope,
            created_at: env.ledger().timestamp(),
            operations_used: 0,
            next_nonce: 0,
        }
    }

    #[test]
    fn allow_in_policy_action_succeeds() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());

        env.as_contract(&contract_id, || {
            let account = Address::generate(&env);
            let key_id = BytesN::from_array(&env, &[7u8; 32]);
            let scope = build_scope(&env, 10, env.ledger().timestamp().saturating_add(3600));
            SessionStorage::store(
                &env,
                &account,
                &make_session_key(&env, key_id.clone(), scope),
            );

            let kernel = AccountKernel::new(account.clone());
            let action = counter_action(&env);

            let result = authorize_action(
                &env,
                &kernel,
                &account,
                &key_id,
                action,
                0,
                env.ledger().timestamp().saturating_add(60),
            );

            assert!(result.is_ok());
        });
    }

    #[test]
    fn expired_session_is_rejected() {
        let env = Env::default();

        let scope = build_scope(&env, 10, env.ledger().timestamp().saturating_sub(1));
        assert!(check_session_expired(&env, &scope));
    }

    #[test]
    fn reused_nonce_is_rejected() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());

        env.as_contract(&contract_id, || {
            let account = Address::generate(&env);
            let key_id = BytesN::from_array(&env, &[9u8; 32]);

            let scope = build_scope(&env, 10, env.ledger().timestamp().saturating_add(3600));
            SessionStorage::store(
                &env,
                &account,
                &make_session_key(&env, key_id.clone(), scope),
            );

            let kernel = AccountKernel::new(account.clone());
            let action = counter_action(&env);

            // First use with nonce 0 succeeds.
            let first = authorize_action(
                &env,
                &kernel,
                &account,
                &key_id,
                action.clone(),
                0,
                env.ledger().timestamp().saturating_add(60),
            );
            assert!(first.is_ok());

            // Second use with same nonce 0 fails with NonceMismatch.
            let second = authorize_action(
                &env,
                &kernel,
                &account,
                &key_id,
                action,
                0,
                env.ledger().timestamp().saturating_add(60),
            );
            assert_eq!(second.unwrap_err(), AccountError::NonceMismatch);
        });
    }

    #[test]
    fn build_scope_sets_action_and_budget() {
        let env = Env::default();
        let scope = build_scope(&env, 5, 99999);

        assert_eq!(scope.allowed_actions.len(), 1);
        assert_eq!(scope.allowed_actions.get(0).unwrap(), symbol_short!("tap"));
        assert_eq!(scope.max_operations, 5);
        assert_eq!(scope.expires_at, 99999);
    }

    #[test]
    fn live_session_passes_expiry_check() {
        let env = Env::default();
        let scope = build_scope(&env, 1, env.ledger().timestamp().saturating_add(3600));
        assert!(!check_session_expired(&env, &scope));
    }

    #[test]
    fn boundary_exact_expiry_timestamp_is_rejected() {
        let env = Env::default();
        let scope = build_scope(&env, 1, env.ledger().timestamp());
        assert!(check_session_expired(&env, &scope));
    }
}
