//! Integration tests for the reusable standards layer.

use cougr_core::standards::{
    AccessControl, BatchExecutor, DelayedExecutionPolicy, ExecutionGuard, Ownable, Ownable2Step,
    Pausable, RecoveryGuard, StandardsError, DEFAULT_ADMIN_ROLE_NAME,
};
use soroban_sdk::{
    contract, contractimpl, symbol_short, testutils::Address as _, testutils::Ledger as _, Address,
    Bytes, Env, Symbol,
};

#[contract]
pub struct TestContract;

#[contractimpl]
impl TestContract {}

#[test]
fn ownable_supports_transfer_and_renounce() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let owner = Address::generate(&env);
    let next_owner = Address::generate(&env);
    let module = Ownable::new(symbol_short!("own"));

    env.as_contract(&contract_id, || {
        module.initialize(&env, &owner).unwrap();
        assert_eq!(module.owner(&env), Some(owner.clone()));

        module
            .transfer_ownership(&env, &owner, &next_owner)
            .unwrap();
        assert_eq!(module.owner(&env), Some(next_owner.clone()));

        module.renounce_ownership(&env, &next_owner).unwrap();
        assert_eq!(module.owner(&env), None);
    });
}

#[test]
fn ownable_2step_requires_pending_owner_acceptance() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let owner = Address::generate(&env);
    let pending_owner = Address::generate(&env);
    let outsider = Address::generate(&env);
    let module = Ownable2Step::new(symbol_short!("own2"));

    env.as_contract(&contract_id, || {
        module.initialize(&env, &owner).unwrap();
        module.begin_transfer(&env, &owner, &pending_owner).unwrap();

        let unauthorized = module.accept_transfer(&env, &outsider);
        assert_eq!(unauthorized, Err(StandardsError::PendingOwnerMismatch));

        module.accept_transfer(&env, &pending_owner).unwrap();
        assert_eq!(module.owner(&env), Some(pending_owner));
        assert_eq!(module.pending_owner(&env), None);
    });
}

#[test]
fn access_control_enforces_admin_hierarchy() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let grantee = Address::generate(&env);
    let module = AccessControl::new(symbol_short!("acl"));
    let ops_role = symbol_short!("OPS");
    let ops_admin_role = symbol_short!("OPADM");

    env.as_contract(&contract_id, || {
        module.initialize(&env, &admin).unwrap();
        module
            .grant_role(&env, &admin, &ops_role, &operator)
            .unwrap();
        assert!(module.has_role(&env, &ops_role, &operator));

        module
            .set_role_admin(&env, &admin, &ops_role, &ops_admin_role)
            .unwrap();
        module
            .grant_role(&env, &admin, &ops_admin_role, &operator)
            .unwrap();
        module
            .grant_role(&env, &operator, &ops_role, &grantee)
            .unwrap();
        assert!(module.has_role(&env, &ops_role, &grantee));

        let default_admin_role = Symbol::new(&env, DEFAULT_ADMIN_ROLE_NAME);
        module
            .revoke_role(&env, &admin, &default_admin_role, &admin)
            .unwrap();
        assert!(!module.has_role(&env, &default_admin_role, &admin));
    });
}

#[test]
fn pausable_blocks_execution_until_unpaused() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let caller = Address::generate(&env);
    let module = Pausable::new(symbol_short!("pause"));

    env.as_contract(&contract_id, || {
        assert!(!module.is_paused(&env));
        module.pause(&env, &caller).unwrap();
        assert_eq!(module.require_not_paused(&env), Err(StandardsError::Paused));
        module.unpause(&env, &caller).unwrap();
        assert!(module.require_not_paused(&env).is_ok());
    });
}

#[test]
fn execution_guard_prevents_reentrancy_like_nesting() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let module = ExecutionGuard::new(symbol_short!("exec"));

    env.as_contract(&contract_id, || {
        module.enter(&env).unwrap();
        assert_eq!(module.enter(&env), Err(StandardsError::ExecutionLocked));
        module.exit(&env).unwrap();

        let value = module.execute(&env, || 42u32).unwrap();
        assert_eq!(value, 42);
        assert!(!module.is_locked(&env));
    });
}

#[test]
fn recovery_guard_blocks_sensitive_paths_while_active() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let caller = Address::generate(&env);
    let module = RecoveryGuard::new(symbol_short!("recov"));

    env.as_contract(&contract_id, || {
        module.activate(&env, &caller).unwrap();
        assert_eq!(
            module.require_inactive(&env),
            Err(StandardsError::RecoveryActive)
        );
        module.clear(&env, &caller).unwrap();
        assert!(module.require_inactive(&env).is_ok());
    });
}

#[test]
fn batch_executor_enforces_limits_and_executes_items() {
    let batch = BatchExecutor::new(3);
    let items = [1u32, 2, 3];

    let results = batch.execute(&items, |item| Ok(item * 2)).unwrap();
    assert_eq!(results, vec![2, 4, 6]);

    assert_eq!(
        batch.execute::<u32, u32>(&[], |_| Ok(0)),
        Err(StandardsError::BatchEmpty)
    );
    assert_eq!(
        batch.execute(&[1u32, 2, 3, 4], |item| Ok(*item)),
        Err(StandardsError::BatchTooLarge)
    );
}

#[test]
fn delayed_execution_enforces_readiness_and_expiry() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let module = DelayedExecutionPolicy::new(symbol_short!("delay"));

    env.as_contract(&contract_id, || {
        env.ledger().with_mut(|li| {
            li.timestamp = 100;
        });

        let scheduled = module
            .schedule(
                &env,
                symbol_short!("UPGD"),
                Bytes::from_array(&env, &[1, 2, 3]),
                10,
                20,
            )
            .unwrap();
        assert_eq!(scheduled.operation_id, 1);
        assert_eq!(module.pending_operations(&env).len(), 1);

        let too_early = module.execute_ready(&env, scheduled.operation_id);
        assert_eq!(too_early, Err(StandardsError::OperationNotReady));

        env.ledger().with_mut(|li| {
            li.timestamp = 111;
        });
        module.execute_ready(&env, scheduled.operation_id).unwrap();
        assert_eq!(module.pending_operations(&env).len(), 0);

        let second = module
            .schedule(
                &env,
                symbol_short!("CANC"),
                Bytes::from_array(&env, &[9]),
                5,
                5,
            )
            .unwrap();
        module.cancel(&env, second.operation_id).unwrap();
        assert_eq!(
            module.execute_ready(&env, second.operation_id),
            Err(StandardsError::OperationNotFound)
        );

        let third = module
            .schedule(
                &env,
                symbol_short!("EXPR"),
                Bytes::from_array(&env, &[8]),
                1,
                1,
            )
            .unwrap();
        env.ledger().with_mut(|li| {
            li.timestamp = 114;
        });
        assert_eq!(
            module.execute_ready(&env, third.operation_id),
            Err(StandardsError::OperationExpired)
        );
    });
}
