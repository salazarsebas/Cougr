//! Stress tests for ECS operations at scale.
//!
//! Tests with 50-100+ entities to verify performance and correctness
//! under load.

use cougr_core::{query::SimpleQueryCache, CommandQueue, ComponentStorage, SimpleWorld};
use soroban_sdk::{symbol_short, Bytes, Env, Symbol};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_100_entities_with_physics() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    // Spawn 100 entities with pos + vel
    for i in 0u8..100 {
        let e = world.spawn_entity();
        world.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[i]));
        world.add_component(e, symbol_short!("vel"), Bytes::from_array(&env, &[1]));
    }

    let entities = world.get_entities_with_component(&symbol_short!("pos"), &env);
    assert_eq!(entities.len(), 100);

    // Run physics: pos = pos + vel
    let entities = world.get_entities_with_component(&symbol_short!("pos"), &env);
    for i in 0..entities.len() {
        let eid = entities.get(i).unwrap();
        if let (Some(pos_data), Some(vel_data)) = (
            world.get_component(eid, &symbol_short!("pos")),
            world.get_component(eid, &symbol_short!("vel")),
        ) {
            let px = pos_data.get(0).unwrap_or(0);
            let vx = vel_data.get(0).unwrap_or(0);
            let new_pos = Bytes::from_array(&env, &[px.wrapping_add(vx)]);
            world.add_component(eid, symbol_short!("pos"), new_pos);
        }
    }

    // Verify first and last entity
    let pos1 = world.get_component(1, &symbol_short!("pos")).unwrap();
    assert_eq!(pos1.get(0).unwrap(), 1); // 0 + 1

    let pos100 = world.get_component(100, &symbol_short!("pos")).unwrap();
    assert_eq!(pos100.get(0).unwrap(), 100); // 99 + 1
}

#[test]
fn test_spawn_50_despawn_25() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    // Spawn 50 entities
    let mut ids = alloc::vec::Vec::new();
    for _ in 0..50 {
        let e = world.spawn_entity();
        world.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[1]));
        ids.push(e);
    }

    let before = world.get_entities_with_component(&symbol_short!("pos"), &env);
    assert_eq!(before.len(), 50);

    // Despawn first 25
    for entity_id in ids.iter().take(25) {
        world.despawn_entity(*entity_id);
    }

    let after = world.get_entities_with_component(&symbol_short!("pos"), &env);
    assert_eq!(after.len(), 25);

    // Remaining entities are ids[25..50]
    for entity_id in ids.iter().take(50).skip(25) {
        assert!(world.has_component(*entity_id, &symbol_short!("pos")));
    }
    for entity_id in ids.iter().take(25) {
        assert!(!world.has_component(*entity_id, &symbol_short!("pos")));
    }
}

#[test]
fn test_10_entities_10_components_each() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    let comp_names: [&str; 10] = [
        "pos", "vel", "hp", "mp", "atk", "def", "spd", "lck", "xp", "lvl",
    ];

    for _ in 0..10 {
        let e = world.spawn_entity();
        for (idx, name) in comp_names.iter().enumerate() {
            let sym = Symbol::new(&env, name);
            let data = Bytes::from_array(&env, &[idx as u8]);
            world.add_component(e, sym, data);
        }
    }

    // Verify all combinations
    for eid in 1..=10u32 {
        for name in &comp_names {
            let sym = Symbol::new(&env, name);
            assert!(world.has_component(eid, &sym));
        }
    }

    // Query for specific component
    let with_hp = world.get_entities_with_component(&Symbol::new(&env, "hp"), &env);
    assert_eq!(with_hp.len(), 10);

    let with_xp = world.get_entities_with_component(&Symbol::new(&env, "xp"), &env);
    assert_eq!(with_xp.len(), 10);
}

#[test]
fn test_query_cache_under_rapid_mutation() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let mut cache = SimpleQueryCache::new(symbol_short!("pos"), &env);

    // Cycle: add 10 entities, query, remove 5, query, repeat
    for cycle in 0..5u32 {
        // Add 10
        let mut added = alloc::vec::Vec::new();
        for _ in 0..10 {
            let e = world.spawn_entity();
            world.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[1]));
            added.push(e);
        }

        let results = cache.execute(&world, &env);
        let expected_after_add = cycle * 5 + 10;
        assert_eq!(results.len(), expected_after_add);

        // Remove first 5 of what we just added
        for entity_id in added.iter().take(5) {
            world.remove_component(*entity_id, &symbol_short!("pos"));
        }

        let results = cache.execute(&world, &env);
        let expected_after_remove = cycle * 5 + 5;
        assert_eq!(results.len(), expected_after_remove);
    }

    // Final count: 5 cycles × 5 remaining = 25
    let final_results = cache.execute(&world, &env);
    assert_eq!(final_results.len(), 25);
}

#[test]
fn test_50_command_queue_operations() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    // Pre-existing entity
    let base = world.spawn_entity();
    world.add_component(base, symbol_short!("base"), Bytes::from_array(&env, &[1]));

    let mut queue = CommandQueue::new();

    // 20 spawns
    for _ in 0..20 {
        queue.spawn();
    }

    // 10 add-component operations to base
    for i in 0..10u8 {
        let sym = Symbol::new(&env, &alloc::format!("c{}", i));
        queue.add_component(base, sym, Bytes::from_array(&env, &[i]));
    }

    // 10 add-sparse-component operations to base
    for i in 10..20u8 {
        let sym = Symbol::new(&env, &alloc::format!("s{}", i));
        queue.add_sparse_component(base, sym, Bytes::from_array(&env, &[i]));
    }

    assert_eq!(queue.len(), 40);

    let spawned = queue.apply(&mut world);
    assert_eq!(spawned.len(), 20);

    // Verify all 10 table components
    for i in 0..10u8 {
        let sym = Symbol::new(&env, &alloc::format!("c{}", i));
        assert!(world.has_component(base, &sym));
    }

    // Verify all 10 sparse components
    for i in 10..20u8 {
        let sym = Symbol::new(&env, &alloc::format!("s{}", i));
        assert!(world.has_component(base, &sym));
    }
}

#[test]
fn test_version_counter_under_heavy_mutation() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    let e = world.spawn_entity();
    let initial_version = world.version();

    // 100 add operations
    for i in 0..100u8 {
        world.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[i]));
    }

    // Each add increments version
    assert_eq!(world.version(), initial_version + 100);

    // 50 remove + re-add cycles
    for i in 0..50u8 {
        world.remove_component(e, &symbol_short!("pos"));
        world.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[i]));
    }

    // Each remove + add = 2 version increments × 50 = 100 more
    assert_eq!(world.version(), initial_version + 100 + 100);
}

#[test]
fn test_large_component_data() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let e = world.spawn_entity();

    // 1KB of data
    let large_data: [u8; 1024] = [0xAB; 1024];
    let bytes = Bytes::from_slice(&env, &large_data);
    world.add_component(e, symbol_short!("big"), bytes.clone());

    let retrieved = world.get_component(e, &symbol_short!("big")).unwrap();
    assert_eq!(retrieved.len(), 1024);
    assert_eq!(retrieved.get(0).unwrap(), 0xAB);
    assert_eq!(retrieved.get(1023).unwrap(), 0xAB);
}

#[test]
fn test_mixed_storage_stress() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    // 30 entities: 15 with table components, 15 with sparse components, 10 with both
    for i in 0u32..30 {
        let e = world.spawn_entity();
        let data = Bytes::from_array(&env, &[(i as u8)]);

        if !(15..20).contains(&i) {
            world.add_component(e, symbol_short!("table"), data.clone());
        }
        if i >= 15 || i >= 20 {
            world.add_component_with_storage(
                e,
                symbol_short!("sparse"),
                data,
                ComponentStorage::Sparse,
            );
        }
    }

    // Table-only query for "table"
    let table_ents = world.get_table_entities_with_component(&symbol_short!("table"), &env);
    assert_eq!(table_ents.len(), 25); // 0..15 + 20..30

    // All query for "sparse"
    let sparse_ents = world.get_all_entities_with_component(&symbol_short!("sparse"), &env);
    assert_eq!(sparse_ents.len(), 15); // 15..30
}

extern crate alloc;
