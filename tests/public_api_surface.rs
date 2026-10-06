//! Public API contract smoke tests.
//!
//! These tests are intentionally shallow: they verify that the sanctioned
//! public entrypoints remain available and that stable versus experimental
//! namespaces stay explicit.

use cougr_core::accounts::{
    verify_secp256r1, ClassicAccount, GameAction, Secp256r1Key, Secp256r1Storage, SessionBuilder,
};
use cougr_core::standards::{
    AccessControl, BatchExecutor, DelayedExecutionPolicy, ExecutionGuard, Ownable, Ownable2Step,
    Pausable, RecoveryGuard, StandardsError,
};
use cougr_core::zk::experimental::{
    bytes32_to_scalar, open_state_channel, u32_to_scalar, CustomCircuit, FogOfWarSnapshot,
    GameCircuit, MovementCircuit, RecursiveProofLayout,
};
use cougr_core::zk::stable::{encode_commit_reveal, CommitReveal, COMMIT_REVEAL_TYPE};
use cougr_core::{
    ArchetypeQueryBuilder, GameApp, Position, QueryStorage, RuntimeWorld, RuntimeWorldMut,
    ScheduleStage, SimpleQueryBuilder, SimpleWorld, SystemConfig, WorldBackend,
};
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Bytes, BytesN, Env, Vec};

#[test]
fn sanctioned_root_api_supports_basic_ecs_flow() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let entity = world.spawn_entity();

    world.set_typed(&env, entity, &Position::new(3, 4));

    let pos: Position = world.get_typed(&env, entity).unwrap();
    assert_eq!(pos.x, 3);
    assert_eq!(pos.y, 4);
}

#[test]
fn sanctioned_root_api_supports_game_app_and_simple_query_flow() {
    let env = Env::default();
    let mut app = GameApp::new(&env);
    app.add_system_with_config(
        "spawn_player",
        |world: &mut SimpleWorld, env: &Env| {
            let entity = world.spawn_entity();
            world.set_typed(env, entity, &Position::new(1, 2));
        },
        SystemConfig::new().in_stage(ScheduleStage::Update),
    );

    app.run(&env).unwrap();

    let query = SimpleQueryBuilder::new(&env)
        .with_component(symbol_short!("position"))
        .build();
    assert_eq!(query.storage(), QueryStorage::Table);
    assert_eq!(query.execute(app.world(), &env).len(), 1);
    assert_eq!(app.world().backend(), WorldBackend::Simple);
}

#[test]
fn sanctioned_app_module_supports_runtime_registration_flow() {
    let env = Env::default();
    let mut app = cougr_core::app::GameApp::new(&env);
    app.add_systems(cougr_core::app::named_context_system(
        "spawn_player",
        |context| {
            let entity = context.world_mut().spawn_entity();
            let env = context.env().clone();
            context
                .world_mut()
                .set_typed(&env, entity, &Position::new(7, 8));
        },
    ));

    app.run(&env).unwrap();

    let query = cougr_core::app::SimpleQueryBuilder::new(&env)
        .with_component(symbol_short!("position"))
        .build();
    assert_eq!(query.execute(app.world(), &env).len(), 1);
}

#[test]
fn sanctioned_app_module_supports_grouped_system_registration() {
    let env = Env::default();
    let mut app = cougr_core::app::GameApp::new(&env);

    app.add_systems((
        cougr_core::app::named_system("spawn_player", |world: &mut SimpleWorld, env: &Env| {
            let entity = world.spawn_entity();
            world.set_typed(env, entity, &Position::new(1, 1));
        })
        .in_stage(ScheduleStage::Update)
        .in_set("spawn"),
        cougr_core::app::named_context_system("mark_spawned", |context| {
            let query = cougr_core::app::SimpleQueryBuilder::new(context.env())
                .with_component(symbol_short!("position"))
                .build();
            let entities = query.execute(context.world(), context.env());
            let env = context.env().clone();
            for i in 0..entities.len() {
                let entity = entities.get(i).unwrap();
                context.commands().add_component(
                    entity,
                    symbol_short!("tag"),
                    Bytes::from_array(&env, &[1]),
                );
            }
        })
        .in_stage(ScheduleStage::Update)
        .after_set("spawn"),
    ));

    app.run(&env).unwrap();

    let query = cougr_core::app::SimpleQueryBuilder::new(&env)
        .with_component(symbol_short!("position"))
        .with_any_component(symbol_short!("tag"))
        .include_sparse()
        .build();
    assert_eq!(query.execute(app.world(), &env).len(), 1);
}

#[test]
fn sanctioned_root_api_supports_archetype_query_builder() {
    let env = Env::default();
    let mut world = cougr_core::ArchetypeWorld::new(&env);
    let entity = world.spawn_entity();
    world.set_typed(&env, entity, &Position::new(9, 9));

    let results = ArchetypeQueryBuilder::new()
        .with_component(symbol_short!("position"))
        .build()
        .execute(&world, &env);

    assert_eq!(results.len(), 1);
    assert_eq!(world.backend(), WorldBackend::Archetype);
}

fn exercise_runtime_world_mut<W: RuntimeWorldMut>(world: &mut W, env: &Env) {
    let entity = world.spawn_entity();
    world.set_typed(env, entity, &Position::new(9, 1));
    assert!(world.has_typed::<Position>(entity));
    let loaded: Position = world.get_typed(env, entity).unwrap();
    assert_eq!(loaded.y, 1);
}

#[test]
fn sanctioned_root_api_exposes_shared_runtime_mut_contract() {
    let env = Env::default();
    let mut simple = SimpleWorld::new(&env);
    let mut archetype = cougr_core::ArchetypeWorld::new(&env);

    exercise_runtime_world_mut(&mut simple, &env);
    exercise_runtime_world_mut(&mut archetype, &env);
}

#[test]
fn sanctioned_root_api_supports_plugin_groups() {
    struct NoopA;
    struct NoopB;

    impl cougr_core::Plugin for NoopA {
        fn name(&self) -> &'static str {
            "noop_a"
        }

        fn build(&self, _app: &mut GameApp) {}
    }

    impl cougr_core::Plugin for NoopB {
        fn name(&self) -> &'static str {
            "noop_b"
        }

        fn build(&self, _app: &mut GameApp) {}
    }

    let env = Env::default();
    let mut app = GameApp::new(&env);
    app.add_plugins((NoopA, NoopB));
    assert_eq!(app.plugin_count(), 2);
}

#[test]
fn stable_zk_namespace_exposes_stable_commit_reveal_flow() {
    let env = Env::default();
    let commitment = BytesN::from_array(&env, &[9u8; 32]);
    let encoded = encode_commit_reveal(&env, &commitment, 123, false);

    assert!(!COMMIT_REVEAL_TYPE.is_empty());
    assert_eq!(encoded.len(), 41);

    let _stable_component: Option<CommitReveal> = None;
}

#[test]
fn privacy_namespace_exposes_split_maturity_explicitly() {
    let env = Env::default();
    let commitment = BytesN::from_array(&env, &[3u8; 32]);
    let encoded = cougr_core::privacy::stable::encode_commit_reveal(&env, &commitment, 55, false);
    assert_eq!(encoded.len(), 41);

    let _exp_fn: fn(
        &Env,
        &cougr_core::privacy::VerificationKey,
        &cougr_core::privacy::Groth16Proof,
        &[cougr_core::privacy::Scalar],
    ) -> Result<bool, cougr_core::privacy::ZKError> =
        cougr_core::privacy::experimental::verify_groth16;
}

#[test]
fn experimental_zk_namespace_exposes_proof_helpers_explicitly() {
    let env = Env::default();
    let g1 = cougr_core::zk::G1Point {
        bytes: BytesN::from_array(&env, &[0u8; 64]),
    };
    let g2 = cougr_core::zk::G2Point {
        bytes: BytesN::from_array(&env, &[0u8; 128]),
    };
    let mut ic = Vec::new(&env);
    for _ in 0..6 {
        ic.push_back(g1.clone());
    }
    let vk = cougr_core::zk::VerificationKey {
        alpha: g1.clone(),
        beta: g2.clone(),
        gamma: g2.clone(),
        delta: g2,
        ic,
    };

    let _movement = MovementCircuit::new(vk.clone(), 10);
    let _game_circuit: &dyn GameCircuit = &_movement;
    let _custom = CustomCircuit::new(
        vk,
        vec![
            u32_to_scalar(&env, 42),
            bytes32_to_scalar(&BytesN::from_array(&env, &[1u8; 32])),
        ],
    );
    let _fog = FogOfWarSnapshot {
        map_root: BytesN::from_array(&env, &[2u8; 32]),
        explored_root: BytesN::from_array(&env, &[3u8; 32]),
        origin_x: 0,
        origin_y: 0,
        visibility_radius: 3,
    };
    let _channel = open_state_channel(
        BytesN::from_array(&env, &[4u8; 32]),
        BytesN::from_array(&env, &[5u8; 32]),
        BytesN::from_array(&env, &[6u8; 32]),
        10,
    )
    .unwrap();
    let _layout = RecursiveProofLayout::from_step_roots(
        &env,
        BytesN::from_array(&env, &[7u8; 32]),
        BytesN::from_array(&env, &[8u8; 32]),
        &[BytesN::from_array(&env, &[9u8; 32])],
    )
    .unwrap();
}

#[test]
fn accounts_namespace_exposes_curated_beta_entrypoints() {
    let env = Env::default();
    let account = Address::generate(&env);
    let _classic = ClassicAccount::new(account.clone());

    let _action = GameAction {
        system_name: symbol_short!("move"),
        data: Bytes::new(&env),
    };

    let _session_builder = SessionBuilder::new(&env).allow_action(symbol_short!("move"));

    let key = Secp256r1Key {
        public_key: BytesN::from_array(&env, &[4u8; 65]),
        label: symbol_short!("passkey"),
        registered_at: 0,
    };
    let _storage_marker = core::mem::size_of::<Secp256r1Storage>();
    let _stored_key = key;

    let _verify_fn: fn(
        &Env,
        &BytesN<65>,
        &Bytes,
        &BytesN<64>,
    ) -> Result<(), cougr_core::accounts::AccountError> = verify_secp256r1;
}

#[test]
fn auth_namespace_mirrors_beta_accounts_surface() {
    let env = Env::default();
    let _session_builder = cougr_core::auth::SessionBuilder::new(&env)
        .allow_action(symbol_short!("move"))
        .max_operations(3);
}

#[test]
fn standards_namespace_exposes_reusable_contract_primitives() {
    let env = Env::default();
    let account = Address::generate(&env);

    let _ownable = Ownable::new(symbol_short!("own"));
    let _ownable_2step = Ownable2Step::new(symbol_short!("own2"));
    let _access = AccessControl::new(symbol_short!("acl"));
    let _pausable = Pausable::new(symbol_short!("pause"));
    let _guard = ExecutionGuard::new(symbol_short!("exec"));
    let _recovery_guard = RecoveryGuard::new(symbol_short!("reco"));
    let _delayed = DelayedExecutionPolicy::new(symbol_short!("delay"));
    let _batch = BatchExecutor::new(8);

    let _error_marker = StandardsError::Paused;
    let _ = account;
}

#[test]
fn ops_namespace_mirrors_standards_surface() {
    let _guard = cougr_core::ops::ExecutionGuard::new(symbol_short!("exec"));
    let _ownable = cougr_core::ops::Ownable::new(symbol_short!("own"));
}

#[test]
fn session_module_exposes_manager_and_status() {
    assert_eq!(cougr_core::session::RENEWAL_HINT_SECONDS, 300);
    let _ = cougr_core::session::SessionManager::cleanup_expired;
}

#[test]
fn circuits_module_exposes_game_builders() {
    let env = Env::default();
    let _ = cougr_core::circuits::hidden_cards(&env, 52, 5).unwrap();
    let _ = cougr_core::circuits::fog_of_war(&env, 32, 32, 3).unwrap();
    let seed = BytesN::from_array(&env, &[1u8; 32]);
    let _ = cougr_core::circuits::fair_dice(&env, 6, &seed).unwrap();
    let _ = cougr_core::circuits::sealed_bid(&env, 1000).unwrap();
    assert_eq!(cougr_core::circuits::MODULE_VERSION, "0.1.0-circuits");
}

#[test]
fn competitive_layers_expose_version_markers() {
    assert_eq!(cougr_core::session::MODULE_VERSION, "0.1.0-session");
}

#[test]
#[cfg(feature = "testutils")]
fn test_sandbox_module_available_with_testutils() {
    assert_eq!(cougr_core::test::MODULE_VERSION, "0.1.1-sandbox");
}
