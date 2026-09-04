//! Edge case tests for boundary conditions and error paths.
//!
//! Tests empty world operations, non-existent entities, double operations,
//! component data boundaries, and other corner cases.

use cougr_core::{
    query::SimpleQueryCache,
    runtime::{ComponentEvent, ObservedWorld},
    CommandQueue, ComponentStorage, GameApp, Plugin, SimpleWorld,
};
use soroban_sdk::{symbol_short, Bytes, Env};

// ---------------------------------------------------------------------------
// Empty World Operations
// ---------------------------------------------------------------------------

#[test]
fn test_query_empty_world() {
    let env = Env::default();
    let world = SimpleWorld::new(&env);

    let entities = world.get_entities_with_component(&symbol_short!("pos"), &env);
    assert_eq!(entities.len(), 0);

    let table_entities = world.get_table_entities_with_component(&symbol_short!("pos"), &env);
    assert_eq!(table_entities.len(), 0);

    let all_entities = world.get_all_entities_with_component(&symbol_short!("pos"), &env);
    assert_eq!(all_entities.len(), 0);
}

#[test]
fn test_despawn_nonexistent_entity() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    // Despawning non-existent entity should not panic
    world.despawn_entity(999);
    assert_eq!(world.version(), 1); // version still increments
}

#[test]
fn test_get_component_nonexistent_entity() {
    let env = Env::default();
    let world = SimpleWorld::new(&env);

    let result = world.get_component(999, &symbol_short!("pos"));
    assert!(result.is_none());
}

#[test]
fn test_has_component_nonexistent_entity() {
    let env = Env::default();
    let world = SimpleWorld::new(&env);

    assert!(!world.has_component(999, &symbol_short!("pos")));
}

#[test]
fn test_remove_component_nonexistent_entity() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    // Removing from non-existent entity returns false
    let removed = world.remove_component(999, &symbol_short!("pos"));
    assert!(!removed);
}

// ---------------------------------------------------------------------------
// Component Operations Edge Cases
// ---------------------------------------------------------------------------

#[test]
fn test_remove_component_that_doesnt_exist() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let e = world.spawn_entity();

    // Entity exists but doesn't have "pos"
    let removed = world.remove_component(e, &symbol_short!("pos"));
    assert!(!removed);
}

#[test]
fn test_add_same_component_twice_replaces() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let e = world.spawn_entity();

    let data1 = Bytes::from_array(&env, &[1]);
    let data2 = Bytes::from_array(&env, &[2]);

    world.add_component(e, symbol_short!("pos"), data1);
    world.add_component(e, symbol_short!("pos"), data2.clone());

    // Should have latest value
    let retrieved = world.get_component(e, &symbol_short!("pos")).unwrap();
    assert_eq!(retrieved, data2);

    // Should still only count as one component
    let entities = world.get_entities_with_component(&symbol_short!("pos"), &env);
    assert_eq!(entities.len(), 1);
}

#[test]
fn test_despawn_entity_with_no_components() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let e = world.spawn_entity();

    // Entity exists but has no components
    world.despawn_entity(e);
    // Should not panic, version should increment
    assert_eq!(world.version(), 1);
}

#[test]
fn test_empty_bytes_component_data() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let e = world.spawn_entity();

    let empty = Bytes::new(&env);
    world.add_component(e, symbol_short!("empty"), empty.clone());

    assert!(world.has_component(e, &symbol_short!("empty")));
    let retrieved = world.get_component(e, &symbol_short!("empty")).unwrap();
    assert_eq!(retrieved.len(), 0);
    assert_eq!(retrieved, empty);
}

#[test]
fn test_large_bytes_component_data() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let e = world.spawn_entity();

    // 4KB component
    let large: [u8; 4096] = [0xFF; 4096];
    let data = Bytes::from_slice(&env, &large);
    world.add_component(e, symbol_short!("big"), data.clone());

    let retrieved = world.get_component(e, &symbol_short!("big")).unwrap();
    assert_eq!(retrieved.len(), 4096);
}

// ---------------------------------------------------------------------------
// Version Counter Edge Cases
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Plugin Edge Cases
// ---------------------------------------------------------------------------

#[test]
fn test_double_plugin_registration() {
    struct MyPlugin;
    impl Plugin for MyPlugin {
        fn name(&self) -> &'static str {
            "my_plugin"
        }
        fn build(&self, app: &mut GameApp) {
            app.add_system("sys", |_world: &mut SimpleWorld, _env: &Env| {});
        }
    }

    let env = Env::default();
    let mut app = GameApp::new(&env);
    app.add_plugin(MyPlugin);
    app.add_plugin(MyPlugin); // duplicate

    assert_eq!(app.plugin_count(), 1);
    assert_eq!(app.system_count(), 1); // not doubled
}

#[test]
fn test_plugin_app_run_with_no_systems() {
    let env = Env::default();
    let mut app = GameApp::new(&env);

    // Should not panic
    app.run(&env).unwrap();
    assert_eq!(app.system_count(), 0);
}

#[test]
fn test_plugin_app_run_with_no_entities() {
    let env = Env::default();
    let mut app = GameApp::new(&env);
    app.add_system("noop", |_world: &mut SimpleWorld, _env: &Env| {});

    // Should not panic even with no entities
    app.run(&env).unwrap();
}

// ---------------------------------------------------------------------------
// Observer Edge Cases
// ---------------------------------------------------------------------------

#[test]
fn test_observer_for_unregistered_component() {
    let env = Env::default();
    let world = SimpleWorld::new(&env);
    let mut observed = ObservedWorld::new(world);

    // Register observer for "vel" only
    observed.observers_mut().on_add(
        symbol_short!("vel"),
        |_event: &ComponentEvent, _world: &SimpleWorld, _env: &Env| {},
    );

    // Add "pos" - observer should NOT fire (and not panic)
    let e = observed.spawn_entity();
    observed.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[1]), &env);

    assert!(observed.has_component(e, &symbol_short!("pos")));
}

#[test]
fn test_remove_nonexistent_component_observed() {
    let env = Env::default();
    let world = SimpleWorld::new(&env);
    let mut observed = ObservedWorld::new(world);

    let e = observed.spawn_entity();

    // Remove component that doesn't exist - should return false, not panic
    let removed = observed.remove_component(e, &symbol_short!("pos"), &env);
    assert!(!removed);
}

// ---------------------------------------------------------------------------
// Command Queue Edge Cases
// ---------------------------------------------------------------------------

#[test]
fn test_empty_command_queue_apply() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let queue = CommandQueue::new();

    assert!(queue.is_empty());
    let spawned = queue.apply(&mut world);
    assert!(spawned.is_empty());
}

#[test]
fn test_despawn_via_queue_then_add_component() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let e = world.spawn_entity();
    world.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[1]));

    let mut queue = CommandQueue::new();
    // Despawn first, then try to add component to same entity
    queue.despawn(e);
    queue.add_component(e, symbol_short!("new"), Bytes::from_array(&env, &[2]));

    queue.apply(&mut world);

    // After despawn, the entity's old components are gone
    assert!(!world.has_component(e, &symbol_short!("pos")));
    // The add_component after despawn will re-add (entity ID is just a number)
    assert!(world.has_component(e, &symbol_short!("new")));
}

// ---------------------------------------------------------------------------
// Query Cache Edge Cases
// ---------------------------------------------------------------------------

#[test]
fn test_query_cache_manual_invalidation() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let mut cache = SimpleQueryCache::new(symbol_short!("pos"), &env);

    let e = world.spawn_entity();
    world.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[1]));

    let results = cache.execute(&world, &env);
    assert_eq!(results.len(), 1);
    assert!(cache.is_valid(world.version()));

    // Manually invalidate
    cache.invalidate();
    assert!(!cache.is_valid(world.version()));

    // Re-execute should work fine
    let results = cache.execute(&world, &env);
    assert_eq!(results.len(), 1);
}

#[test]
fn test_query_cache_for_nonexistent_component() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let mut cache = SimpleQueryCache::new(symbol_short!("xyz"), &env);

    let e = world.spawn_entity();
    world.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[1]));

    // Query for "xyz" which no entity has
    let results = cache.execute(&world, &env);
    assert_eq!(results.len(), 0);
}

// ---------------------------------------------------------------------------
// Sparse Storage Edge Cases
// ---------------------------------------------------------------------------

#[test]
fn test_sparse_to_table_component_switch() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let e = world.spawn_entity();

    // Add as sparse
    world.add_component_with_storage(
        e,
        symbol_short!("comp"),
        Bytes::from_array(&env, &[1]),
        ComponentStorage::Sparse,
    );
    assert_eq!(world.table_component_count(&symbol_short!("comp")), 0);
    assert_eq!(world.component_count(&symbol_short!("comp")), 1);

    // Add again as table (overwrites)
    world.add_component_with_storage(
        e,
        symbol_short!("comp"),
        Bytes::from_array(&env, &[2]),
        ComponentStorage::Table,
    );

    // Now in table storage
    assert_eq!(world.table_component_count(&symbol_short!("comp")), 1);
    assert_eq!(world.component_count(&symbol_short!("comp")), 1);
    assert!(world.has_component(e, &symbol_short!("comp")));
    let val = world.get_component(e, &symbol_short!("comp")).unwrap();
    assert_eq!(val.get(0).unwrap(), 2);
}

#[test]
fn test_multiple_spawns_produce_sequential_ids() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    let ids: alloc::vec::Vec<u32> = (0..20).map(|_| world.spawn_entity()).collect();

    for (i, entity_id) in ids.iter().enumerate().take(20) {
        assert_eq!(*entity_id, (i as u32) + 1);
    }
    assert_eq!(world.next_entity_id(), 21);
}

extern crate alloc;
