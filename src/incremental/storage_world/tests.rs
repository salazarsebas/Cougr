use super::{StorageWorld, TtlPolicy};
use crate::component::ComponentStorage;
use crate::error::CougrError;
use crate::incremental::keys;
use crate::query::SimpleQuery;
use crate::simple_world::SimpleWorld;
use soroban_sdk::{
    contract, contractimpl, symbol_short,
    testutils::{storage::Persistent as _, Ledger as _},
    Bytes, Env, Symbol,
};

// Dummy contract for persistent storage context
#[contract]
pub struct TestContract;

#[contractimpl]
impl TestContract {}

#[test]
fn test_load_fresh_metadata() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());

    env.as_contract(&contract_id, || {
        let world = StorageWorld::load_metadata(&env);
        assert_eq!(world.next_entity_id(), 1);
        assert_eq!(world.version(), 0);
        assert_eq!(world.entity_count(), 0);
    });
}

#[test]
fn test_spawn_entity() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        let e1 = world.spawn_entity(&env);
        let e2 = world.spawn_entity(&env);
        assert_eq!(e1, 1);
        assert_eq!(e2, 2);
        assert_eq!(world.entity_count(), 2);
        assert_eq!(world.next_entity_id(), 3);
    });
}

#[test]
fn test_add_and_get_component() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        let eid = world.spawn_entity(&env);

        let data = Bytes::from_array(&env, &[1, 2, 3, 4]);
        world
            .add_component(&env, eid, symbol_short!("pos"), data.clone())
            .unwrap();

        assert!(world.has_component(eid, &symbol_short!("pos")));
        assert_eq!(world.get_component(eid, &symbol_short!("pos")), Some(data));
        assert!(!world.has_component(eid, &symbol_short!("vel")));
    });
}

#[test]
fn test_remove_component() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        let eid = world.spawn_entity(&env);

        let data = Bytes::from_array(&env, &[1]);
        world
            .add_component(&env, eid, symbol_short!("pos"), data)
            .unwrap();

        assert!(world.remove_component(eid, &symbol_short!("pos")));
        assert!(!world.has_component(eid, &symbol_short!("pos")));
        assert!(!world.remove_component(eid, &symbol_short!("pos")));
    });
}

#[test]
fn test_despawn_entity() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        let eid = world.spawn_entity(&env);

        let data = Bytes::from_array(&env, &[1]);
        world
            .add_component(&env, eid, symbol_short!("pos"), data.clone())
            .unwrap();
        world
            .add_component(&env, eid, symbol_short!("vel"), data)
            .unwrap();

        world.despawn_entity(eid);
        assert!(!world.has_component(eid, &symbol_short!("pos")));
        assert!(!world.has_component(eid, &symbol_short!("vel")));
        assert_eq!(world.entity_count(), 0);
    });
}

#[test]
fn test_flush_and_reload() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        let eid = world.spawn_entity(&env);
        let data = Bytes::from_array(&env, &[42, 43]);
        world
            .add_component(&env, eid, symbol_short!("pos"), data)
            .unwrap();
        world.flush(&env);
    });

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        assert_eq!(world.next_entity_id(), 2);
        assert_eq!(world.entity_count(), 1);

        world.load_entity(&env, 1).unwrap();
        assert!(world.has_component(1, &symbol_short!("pos")));
        let data = world.get_component(1, &symbol_short!("pos")).unwrap();
        assert_eq!(data, Bytes::from_array(&env, &[42, 43]));
    });
}

#[test]
fn test_flush_despawn_removes_storage() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        let eid = world.spawn_entity(&env);
        let data = Bytes::from_array(&env, &[1]);
        world
            .add_component(&env, eid, symbol_short!("pos"), data)
            .unwrap();
        world.flush(&env);
    });

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        world.load_entity(&env, 1).unwrap();
        world.despawn_entity(1);
        world.flush(&env);
    });

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        assert_eq!(world.entity_count(), 0);
        let result = world.load_entity(&env, 1);
        assert_eq!(result, Err(CougrError::EntityNotFound));
    });
}

#[test]
fn test_incremental_update() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        let e1 = world.spawn_entity(&env);
        let e2 = world.spawn_entity(&env);
        world
            .add_component(
                &env,
                e1,
                symbol_short!("pos"),
                Bytes::from_array(&env, &[10]),
            )
            .unwrap();
        world
            .add_component(
                &env,
                e2,
                symbol_short!("pos"),
                Bytes::from_array(&env, &[20]),
            )
            .unwrap();
        world.flush(&env);
    });

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        world.load_entity(&env, 1).unwrap();
        world
            .add_component(
                &env,
                1,
                symbol_short!("pos"),
                Bytes::from_array(&env, &[99]),
            )
            .unwrap();
        world.flush(&env);
    });

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        world.load_entity(&env, 1).unwrap();
        world.load_entity(&env, 2).unwrap();

        assert_eq!(
            world.get_component(1, &symbol_short!("pos")),
            Some(Bytes::from_array(&env, &[99]))
        );
        assert_eq!(
            world.get_component(2, &symbol_short!("pos")),
            Some(Bytes::from_array(&env, &[20]))
        );
    });
}

#[test]
fn test_version_tracking() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        assert_eq!(world.version(), 0);

        let eid = world.spawn_entity(&env);
        world
            .add_component(
                &env,
                eid,
                symbol_short!("pos"),
                Bytes::from_array(&env, &[1]),
            )
            .unwrap();
        let v1 = world.version();
        assert!(v1 > 0);

        world.remove_component(eid, &symbol_short!("pos"));
        assert!(world.version() > v1);
    });
}

#[test]
fn test_to_simple_world() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        let e1 = world.spawn_entity(&env);
        let e2 = world.spawn_entity(&env);
        world
            .add_component(
                &env,
                e1,
                symbol_short!("pos"),
                Bytes::from_array(&env, &[1, 2]),
            )
            .unwrap();
        world
            .add_component_with_storage(
                &env,
                e2,
                symbol_short!("vel"),
                Bytes::from_array(&env, &[3, 4]),
                ComponentStorage::Sparse,
            )
            .unwrap();

        let simple = world.to_simple_world(&env);
        assert!(simple.has_component(e1, &symbol_short!("pos")));
        assert!(simple.has_component(e2, &symbol_short!("vel")));
        assert!(!simple.has_component(e1, &symbol_short!("vel")));

        // Verify SimpleWorld indexes and SimpleQuery work properly
        let pos_entities = simple.get_entities_with_component(&symbol_short!("pos"), &env);
        assert_eq!(pos_entities.len(), 1);
        assert_eq!(pos_entities.get(0), Some(e1));

        // Sparse component is excluded from table-only query, included in all-entities query
        let vel_table = simple.get_entities_with_component(&symbol_short!("vel"), &env);
        assert_eq!(vel_table.len(), 0);
        let vel_all = simple.get_all_entities_with_component(&symbol_short!("vel"), &env);
        assert_eq!(vel_all.len(), 1);
        assert_eq!(vel_all.get(0), Some(e2));

        let query = SimpleQuery::new(&env).with_component(symbol_short!("pos"));
        assert_eq!(query.execute(&simple, &env).len(), 1);
    });
}

#[test]
fn test_from_simple_world() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());

    env.as_contract(&contract_id, || {
        let mut simple = SimpleWorld::new(&env);
        let e1 = simple.spawn_entity();
        let e2 = simple.spawn_entity();
        simple.add_component(e1, symbol_short!("pos"), Bytes::from_array(&env, &[10, 20]));
        simple.add_component(e2, symbol_short!("vel"), Bytes::from_array(&env, &[30, 40]));

        let mut storage = StorageWorld::from_simple_world(&simple, &env);
        assert_eq!(storage.entity_count(), 2);
        assert!(storage.has_component(e1, &symbol_short!("pos")));
        assert!(storage.has_component(e2, &symbol_short!("vel")));

        storage.flush(&env);
    });

    env.as_contract(&contract_id, || {
        let mut reloaded = StorageWorld::load_metadata(&env);
        reloaded.load_entity(&env, 1).unwrap();
        reloaded.load_entity(&env, 2).unwrap();
        assert_eq!(
            reloaded.get_component(1, &symbol_short!("pos")),
            Some(Bytes::from_array(&env, &[10, 20]))
        );
        assert_eq!(
            reloaded.get_component(2, &symbol_short!("vel")),
            Some(Bytes::from_array(&env, &[30, 40]))
        );
    });
}

#[test]
fn test_no_flush_when_clean() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        world.flush(&env);
        assert_eq!(world.version(), 0);
    });
}

#[test]
fn test_namespaced_worlds_coexist() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());

    env.as_contract(&contract_id, || {
        let mut world_a = StorageWorld::open(&env, Symbol::new(&env, "arena"));
        let mut world_b = StorageWorld::open(&env, Symbol::new(&env, "lobby"));

        let ea = world_a.spawn_entity(&env);
        world_a
            .add_component(
                &env,
                ea,
                symbol_short!("pos"),
                Bytes::from_array(&env, &[1]),
            )
            .unwrap();
        world_a.flush(&env);

        let eb = world_b.spawn_entity(&env);
        world_b
            .add_component(
                &env,
                eb,
                symbol_short!("pos"),
                Bytes::from_array(&env, &[2]),
            )
            .unwrap();
        let eb2 = world_b.spawn_entity(&env);
        world_b
            .add_component(
                &env,
                eb2,
                symbol_short!("vel"),
                Bytes::from_array(&env, &[3]),
            )
            .unwrap();
        world_b.flush(&env);
    });

    env.as_contract(&contract_id, || {
        let mut world_a = StorageWorld::open(&env, Symbol::new(&env, "arena"));
        let mut world_b = StorageWorld::open(&env, Symbol::new(&env, "lobby"));

        assert_eq!(world_a.entity_count(), 1);
        assert_eq!(world_b.entity_count(), 2);

        world_a.load_entity(&env, 1).unwrap();
        world_b.load_entity(&env, 1).unwrap();
        world_b.load_entity(&env, 2).unwrap();

        assert_eq!(
            world_a.get_component(1, &symbol_short!("pos")),
            Some(Bytes::from_array(&env, &[1]))
        );
        assert_eq!(
            world_b.get_component(1, &symbol_short!("pos")),
            Some(Bytes::from_array(&env, &[2]))
        );
        assert_eq!(
            world_b.get_component(2, &symbol_short!("vel")),
            Some(Bytes::from_array(&env, &[3]))
        );
    });
}

#[test]
fn test_add_component_on_unloaded_entity_preserves_existing_components() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());

    // Invocation 1: spawn entity with pos and hp
    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        let eid = world.spawn_entity(&env);
        world
            .add_component(
                &env,
                eid,
                symbol_short!("pos"),
                Bytes::from_array(&env, &[1, 2]),
            )
            .unwrap();
        world
            .add_component(
                &env,
                eid,
                symbol_short!("hp"),
                Bytes::from_array(&env, &[100]),
            )
            .unwrap();
        world.flush(&env);
    });

    // Invocation 2: add vel WITHOUT calling load_entity first
    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        world
            .add_component(
                &env,
                1,
                symbol_short!("vel"),
                Bytes::from_array(&env, &[5, 6]),
            )
            .unwrap();
        world.flush(&env);
    });

    // Invocation 3: reload and verify all three components remain reachable
    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        world.load_entity(&env, 1).unwrap();
        assert_eq!(
            world.get_component(1, &symbol_short!("pos")),
            Some(Bytes::from_array(&env, &[1, 2]))
        );
        assert_eq!(
            world.get_component(1, &symbol_short!("hp")),
            Some(Bytes::from_array(&env, &[100]))
        );
        assert_eq!(
            world.get_component(1, &symbol_short!("vel")),
            Some(Bytes::from_array(&env, &[5, 6]))
        );
    });
}

#[test]
fn test_add_component_unknown_or_despawned_entity_returns_not_found() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        let res = world.add_component(
            &env,
            999,
            symbol_short!("pos"),
            Bytes::from_array(&env, &[1]),
        );
        assert_eq!(res, Err(CougrError::EntityNotFound));
        assert_eq!(world.version(), 0);
        assert!(!env
            .storage()
            .persistent()
            .has(&keys::entity_key(&env, world.namespace(), 999)));
        assert!(!env.storage().persistent().has(&keys::component_key(
            &env,
            world.namespace(),
            999,
            &symbol_short!("pos")
        )));

        let eid = world.spawn_entity(&env);
        world
            .add_component(
                &env,
                eid,
                symbol_short!("pos"),
                Bytes::from_array(&env, &[1]),
            )
            .unwrap();
        world.flush(&env);
    });

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        world.despawn_entity(1);
        let res = world.add_component(&env, 1, symbol_short!("vel"), Bytes::from_array(&env, &[2]));
        assert_eq!(res, Err(CougrError::EntityNotFound));
        world.flush(&env);
    });

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        let res = world.add_component(&env, 1, symbol_short!("pos"), Bytes::from_array(&env, &[3]));
        assert_eq!(res, Err(CougrError::EntityNotFound));
    });
}

#[test]
fn test_remove_component_deletes_persistent_entry() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let ns = Symbol::new(&env, "default");

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::open(&env, ns.clone());
        let eid = world.spawn_entity(&env);
        world
            .add_component(
                &env,
                eid,
                symbol_short!("pos"),
                Bytes::from_array(&env, &[1]),
            )
            .unwrap();
        world
            .add_component(
                &env,
                eid,
                symbol_short!("vel"),
                Bytes::from_array(&env, &[2]),
            )
            .unwrap();
        world.flush(&env);

        assert!(env.storage().persistent().has(&keys::component_key(
            &env,
            &ns,
            eid,
            &symbol_short!("vel")
        )));
    });

    // Remove vel and flush -> persistent entry must be deleted
    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::open(&env, ns.clone());
        world.load_entity(&env, 1).unwrap();
        assert!(world.remove_component(1, &symbol_short!("vel")));
        world.flush(&env);

        assert!(!env.storage().persistent().has(&keys::component_key(
            &env,
            &ns,
            1,
            &symbol_short!("vel")
        )));
        assert!(env.storage().persistent().has(&keys::component_key(
            &env,
            &ns,
            1,
            &symbol_short!("pos")
        )));
    });

    // Remove then re-add in same session -> entry must be persisted with new value
    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::open(&env, ns.clone());
        world.load_entity(&env, 1).unwrap();
        assert!(world.remove_component(1, &symbol_short!("pos")));
        world
            .add_component(&env, 1, symbol_short!("pos"), Bytes::from_array(&env, &[9]))
            .unwrap();
        world.flush(&env);
    });

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::open(&env, ns.clone());
        world.load_entity(&env, 1).unwrap();
        assert_eq!(
            world.get_component(1, &symbol_short!("pos")),
            Some(Bytes::from_array(&env, &[9]))
        );
        assert!(!world.has_component(1, &symbol_short!("vel")));
    });
}

#[test]
fn test_ttl_policy_extends_entries_across_ledger_advances() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let ns = Symbol::new(&env, "ttl_test");

    let initial_ttl = env.as_contract(&contract_id, || {
        let mut world = StorageWorld::open(&env, ns.clone());
        let eid = world.spawn_entity(&env);
        world
            .add_component(
                &env,
                eid,
                symbol_short!("pos"),
                Bytes::from_array(&env, &[7]),
            )
            .unwrap();
        world.flush(&env);

        env.storage().persistent().get_ttl(&keys::component_key(
            &env,
            &ns,
            eid,
            &symbol_short!("pos"),
        ))
    });

    // Advance ledger so remaining TTL falls below DEFAULT_THRESHOLD (1_000)
    env.ledger().with_mut(|li| {
        li.sequence_number += initial_ttl.saturating_sub(100);
    });

    // Reading the entity extends TTL to DEFAULT_EXTEND_TO (10_000)
    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::open(&env, ns.clone());
        world.load_entity(&env, 1).unwrap();
        assert_eq!(
            world.get_component(1, &symbol_short!("pos")),
            Some(Bytes::from_array(&env, &[7]))
        );
        let ttl = env.storage().persistent().get_ttl(&keys::component_key(
            &env,
            &ns,
            1,
            &symbol_short!("pos"),
        ));
        assert!(ttl >= TtlPolicy::DEFAULT_EXTEND_TO);
    });

    // Advance past the original initial_ttl — entry is still alive and loadable
    env.ledger().with_mut(|li| {
        li.sequence_number += 5_000;
    });

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::open(&env, ns.clone());
        world.load_entity(&env, 1).unwrap();
        assert_eq!(
            world.get_component(1, &symbol_short!("pos")),
            Some(Bytes::from_array(&env, &[7]))
        );
    });

    // Opt-out policy does not extend TTL even when below threshold
    let opt_ns = Symbol::new(&env, "no_ttl");
    let opt_initial_ttl = env.as_contract(&contract_id, || {
        let mut world = StorageWorld::open_with_ttl(&env, opt_ns.clone(), TtlPolicy::disabled());
        let eid = world.spawn_entity(&env);
        world
            .add_component(
                &env,
                eid,
                symbol_short!("pos"),
                Bytes::from_array(&env, &[1]),
            )
            .unwrap();
        world.flush(&env);
        env.storage()
            .persistent()
            .get_ttl(&keys::meta_key(&env, &opt_ns))
    });

    env.ledger().with_mut(|li| {
        li.sequence_number += opt_initial_ttl.saturating_sub(100);
    });

    env.as_contract(&contract_id, || {
        let _world = StorageWorld::open_with_ttl(&env, opt_ns.clone(), TtlPolicy::disabled());
        let ttl = env
            .storage()
            .persistent()
            .get_ttl(&keys::meta_key(&env, &opt_ns));
        assert!(ttl <= 100);
    });
}

#[test]
fn test_bounded_spawn_and_despawn_write_footprint() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());

    let mut first_spawn_cost = 0u64;
    let mut late_spawn_cost = 0u64;

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        env.cost_estimate().budget().reset_unlimited();
        let before = env.cost_estimate().budget().cpu_instruction_cost();
        let e1 = world.spawn_entity(&env);
        world.flush(&env);
        let after = env.cost_estimate().budget().cpu_instruction_cost();
        first_spawn_cost = after - before;
        assert_eq!(e1, 1);
    });

    // Spawn 249 more entities in batches across separate contract invocations
    for _ in 0..5 {
        env.as_contract(&contract_id, || {
            let mut world = StorageWorld::load_metadata(&env);
            for _ in 0..50 {
                if world.entity_count() < 250 {
                    world.spawn_entity(&env);
                }
            }
            world.flush(&env);
        });
    }

    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        env.cost_estimate().budget().reset_unlimited();
        let before_late = env.cost_estimate().budget().cpu_instruction_cost();
        let e251 = world.spawn_entity(&env);
        world.flush(&env);
        let after_late = env.cost_estimate().budget().cpu_instruction_cost();
        late_spawn_cost = after_late - before_late;
        assert_eq!(e251, 251);
        assert_eq!(world.entity_count(), 251);
        assert_eq!(world.entity_index_bucket_count(), 4);
    });

    // The 251st spawn must stay bounded — O(1) writes regardless of entity count.
    // The first spawn writes to an empty bucket (cold path: no existing bucket to read).
    // A later spawn must first READ the existing bucket (triggering TTL extension + deser)
    // before appending, then write it back.  That constant read overhead is why the late
    // spawn costs more, but it must NOT grow proportionally to the number of entities.
    // We use a 20x bound: if entity_ids were stored in a flat list, flushing 251 entities
    // would cost ~251x the first spawn.  20x proves the indexed design stays O(1).
    assert!(
        late_spawn_cost <= first_spawn_cost * 20,
        "late spawn cost ({}) grew disproportionately vs first spawn ({}) — expected O(1) bucket writes",
        late_spawn_cost,
        first_spawn_cost
    );

    // Despawn entity 1 (swap-remove from bucket 0 with last entity 251 in bucket 3)
    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::load_metadata(&env);
        world.despawn_entity(1);
        world.flush(&env);

        assert_eq!(world.entity_count(), 250);
        let page0 = world.entity_ids_page(&env, 0);
        assert_eq!(page0.len(), 64);
        assert_eq!(page0.get(0), Some(251));

        let page3 = world.entity_ids_page(&env, 3);
        assert_eq!(page3.len(), 250 - 64 * 3);
    });
}

#[test]
fn test_apply_simple_world_minimal_diff_and_despawn() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let ns = Symbol::new(&env, "view_diff");

    // Invocation 1: seed 3 entities
    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::open(&env, ns.clone());
        let e1 = world.spawn_entity(&env);
        let e2 = world.spawn_entity(&env);
        let e3 = world.spawn_entity(&env);
        world
            .add_component(
                &env,
                e1,
                symbol_short!("pos"),
                Bytes::from_array(&env, &[1]),
            )
            .unwrap();
        world
            .add_component(
                &env,
                e1,
                symbol_short!("hp"),
                Bytes::from_array(&env, &[10]),
            )
            .unwrap();
        world
            .add_component(
                &env,
                e2,
                symbol_short!("pos"),
                Bytes::from_array(&env, &[2]),
            )
            .unwrap();
        world
            .add_component(
                &env,
                e3,
                symbol_short!("pos"),
                Bytes::from_array(&env, &[3]),
            )
            .unwrap();
        world.flush(&env);
    });

    // Invocation 2: load e1 and e2 (leaving e3 unloaded), run changes on SimpleWorld view
    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::open(&env, ns.clone());
        world.load_entities(&env, &[1, 2]).unwrap();

        let mut view = world.to_simple_world(&env);
        // Change only e1's pos (hp stays unchanged)
        view.add_component(1, symbol_short!("pos"), Bytes::from_array(&env, &[99]));
        // Despawn e2
        view.despawn_entity(2);
        // Spawn e4 in view
        let e4 = view.spawn_entity();
        view.add_component(e4, symbol_short!("pos"), Bytes::from_array(&env, &[44]));

        world.apply_simple_world(&env, &view);

        // Verify only e1's pos and e4's pos are dirty (e1's hp was NOT modified)
        assert_eq!(world.dirty.dirty_components().len(), 2);
        assert!(world
            .dirty
            .dirty_components()
            .contains(&(1, symbol_short!("pos"))));
        assert!(world
            .dirty
            .dirty_components()
            .contains(&(e4, symbol_short!("pos"))));
        assert!(!world
            .dirty
            .dirty_components()
            .contains(&(1, symbol_short!("hp"))));
        assert_eq!(world.dirty.despawned(), &[2]);

        world.flush(&env);
    });

    // Invocation 3: verify persisted state
    env.as_contract(&contract_id, || {
        let mut world = StorageWorld::open(&env, ns.clone());
        assert_eq!(world.entity_count(), 3); // e1, e3, e4
        assert_eq!(world.load_entity(&env, 2), Err(CougrError::EntityNotFound));
        assert!(!env.storage().persistent().has(&keys::component_key(
            &env,
            &ns,
            2,
            &symbol_short!("pos")
        )));

        world.load_entities(&env, &[1, 3, 4]).unwrap();
        assert_eq!(
            world.get_component(1, &symbol_short!("pos")),
            Some(Bytes::from_array(&env, &[99]))
        );
        assert_eq!(
            world.get_component(1, &symbol_short!("hp")),
            Some(Bytes::from_array(&env, &[10]))
        );
        assert_eq!(
            world.get_component(3, &symbol_short!("pos")),
            Some(Bytes::from_array(&env, &[3]))
        );
        assert_eq!(
            world.get_component(4, &symbol_short!("pos")),
            Some(Bytes::from_array(&env, &[44]))
        );
    });
}
