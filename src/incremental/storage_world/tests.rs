use super::StorageWorld;
use crate::error::CougrError;
use crate::simple_world::SimpleWorld;
use soroban_sdk::{contract, contractimpl, symbol_short, Bytes, Env};

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
        world.add_component(&env, eid, symbol_short!("pos"), data.clone());

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
        world.add_component(&env, eid, symbol_short!("pos"), data);

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
        world.add_component(&env, eid, symbol_short!("pos"), data.clone());
        world.add_component(&env, eid, symbol_short!("vel"), data);

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
        {
            let mut world = StorageWorld::load_metadata(&env);
            let eid = world.spawn_entity(&env);
            let data = Bytes::from_array(&env, &[42, 43]);
            world.add_component(&env, eid, symbol_short!("pos"), data);
            world.flush(&env);
        }

        {
            let mut world = StorageWorld::load_metadata(&env);
            assert_eq!(world.next_entity_id(), 2);
            assert_eq!(world.entity_count(), 1);

            world.load_entity(&env, 1).unwrap();
            assert!(world.has_component(1, &symbol_short!("pos")));
            let data = world.get_component(1, &symbol_short!("pos")).unwrap();
            assert_eq!(data, Bytes::from_array(&env, &[42, 43]));
        }
    });
}

#[test]
fn test_flush_despawn_removes_storage() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());

    env.as_contract(&contract_id, || {
        {
            let mut world = StorageWorld::load_metadata(&env);
            let eid = world.spawn_entity(&env);
            let data = Bytes::from_array(&env, &[1]);
            world.add_component(&env, eid, symbol_short!("pos"), data);
            world.flush(&env);
        }

        {
            let mut world = StorageWorld::load_metadata(&env);
            world.load_entity(&env, 1).unwrap();
            world.despawn_entity(1);
            world.flush(&env);
        }

        {
            let mut world = StorageWorld::load_metadata(&env);
            assert_eq!(world.entity_count(), 0);
            let result = world.load_entity(&env, 1);
            assert_eq!(result, Err(CougrError::EntityNotFound));
        }
    });
}

#[test]
fn test_incremental_update() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());

    env.as_contract(&contract_id, || {
        {
            let mut world = StorageWorld::load_metadata(&env);
            let e1 = world.spawn_entity(&env);
            let e2 = world.spawn_entity(&env);
            world.add_component(
                &env,
                e1,
                symbol_short!("pos"),
                Bytes::from_array(&env, &[10]),
            );
            world.add_component(
                &env,
                e2,
                symbol_short!("pos"),
                Bytes::from_array(&env, &[20]),
            );
            world.flush(&env);
        }

        {
            let mut world = StorageWorld::load_metadata(&env);
            world.load_entity(&env, 1).unwrap();
            world.add_component(
                &env,
                1,
                symbol_short!("pos"),
                Bytes::from_array(&env, &[99]),
            );
            world.flush(&env);
        }

        {
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
        }
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
        world.add_component(
            &env,
            eid,
            symbol_short!("pos"),
            Bytes::from_array(&env, &[1]),
        );
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
        world.add_component(
            &env,
            e1,
            symbol_short!("pos"),
            Bytes::from_array(&env, &[1, 2]),
        );
        world.add_component(
            &env,
            e2,
            symbol_short!("vel"),
            Bytes::from_array(&env, &[3, 4]),
        );

        let simple = world.to_simple_world(&env);
        assert!(simple.has_component(e1, &symbol_short!("pos")));
        assert!(simple.has_component(e2, &symbol_short!("vel")));
        assert!(!simple.has_component(e1, &symbol_short!("vel")));
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

        let mut reloaded = StorageWorld::load_metadata(&env);
        reloaded.load_entity(&env, e1).unwrap();
        reloaded.load_entity(&env, e2).unwrap();
        assert_eq!(
            reloaded.get_component(e1, &symbol_short!("pos")),
            Some(Bytes::from_array(&env, &[10, 20]))
        );
        assert_eq!(
            reloaded.get_component(e2, &symbol_short!("vel")),
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
