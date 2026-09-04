//! Cross-module integration tests.
//!
//! Tests interactions between multiple ECS subsystems working together:
//! GameApp + ObservedWorld, TrackedWorld + CommandQueue,
//! SimpleQueryCache + TrackedWorld, etc.

use core::sync::atomic::{AtomicU32, Ordering};
use cougr_core::scheduler::SimpleScheduler;
use cougr_core::{
    query::SimpleQueryCache,
    runtime::{ComponentEvent, ObservedWorld, TrackedWorld},
    CommandQueue, GameApp, Plugin, SimpleWorld,
};
use soroban_sdk::{symbol_short, Bytes, Env};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

static OBSERVER_FIRE_COUNT: AtomicU32 = AtomicU32::new(0);

fn counting_observer(_event: &ComponentEvent, _world: &SimpleWorld, _env: &Env) {
    OBSERVER_FIRE_COUNT.fetch_add(1, Ordering::Relaxed);
}

fn setup_physics_system(world: &mut SimpleWorld, env: &Env) {
    let entities = world.get_entities_with_component(&symbol_short!("pos"), env);
    for i in 0..entities.len() {
        let eid = entities.get(i).unwrap();
        if let (Some(pos_data), Some(vel_data)) = (
            world.get_component(eid, &symbol_short!("pos")),
            world.get_component(eid, &symbol_short!("vel")),
        ) {
            let px = pos_data.get(0).unwrap_or(0);
            let vx = vel_data.get(0).unwrap_or(0);
            let new_pos = Bytes::from_array(env, &[px.wrapping_add(vx)]);
            world.add_component(eid, symbol_short!("pos"), new_pos);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_tracked_world_records_command_queue_changes() {
    let env = Env::default();
    let world = SimpleWorld::new(&env);
    let mut tracked = TrackedWorld::new(world);

    // Spawn entity and add component via TrackedWorld
    let e1 = tracked.spawn_entity();
    tracked.add_component(e1, symbol_short!("pos"), Bytes::from_array(&env, &[1]));

    // Now use CommandQueue on the underlying world
    let mut queue = CommandQueue::new();
    queue.spawn();
    queue.add_component(e1, symbol_short!("hp"), Bytes::from_array(&env, &[100]));
    let spawned = queue.apply(tracked.world_mut());

    assert_eq!(spawned.len(), 1);
    let _e2 = spawned[0];

    // TrackedWorld recorded the direct add
    assert!(tracked.tracker().was_added(e1, &symbol_short!("pos")));
    // But CommandQueue bypasses tracking (applied directly to world)
    assert!(!tracked.tracker().was_added(e1, &symbol_short!("hp")));

    // Both components exist on the world
    assert!(tracked.has_component(e1, &symbol_short!("pos")));
    assert!(tracked.has_component(e1, &symbol_short!("hp")));
}

#[test]
fn test_query_cache_invalidated_by_tracked_mutations() {
    let env = Env::default();
    let world = SimpleWorld::new(&env);
    let mut tracked = TrackedWorld::new(world);
    let mut cache = SimpleQueryCache::new(symbol_short!("pos"), &env);

    // Initial query: empty
    let results = cache.execute(tracked.world(), &env);
    assert_eq!(results.len(), 0);

    // Add component through tracked world
    let e1 = tracked.spawn_entity();
    tracked.add_component(e1, symbol_short!("pos"), Bytes::from_array(&env, &[1]));

    // Tracked mutation changes world version → cache is stale
    assert!(!cache.is_valid(tracked.world().version()));

    let results = cache.execute(tracked.world(), &env);
    assert_eq!(results.len(), 1);

    // Remove through tracked world
    tracked.remove_component(e1, &symbol_short!("pos"));
    let results = cache.execute(tracked.world(), &env);
    assert_eq!(results.len(), 0);
}

#[test]
fn test_observed_world_into_plugin_app() {
    let env = Env::default();
    let before = OBSERVER_FIRE_COUNT.load(Ordering::Relaxed);

    // Build an ObservedWorld with an observer
    let world = SimpleWorld::new(&env);
    let mut observed = ObservedWorld::new(world);
    observed
        .observers_mut()
        .on_add(symbol_short!("pos"), counting_observer);

    // Add component through observed world (fires observer)
    let e1 = observed.spawn_entity();
    observed.add_component(
        e1,
        symbol_short!("pos"),
        Bytes::from_array(&env, &[10]),
        &env,
    );
    observed.add_component(
        e1,
        symbol_short!("vel"),
        Bytes::from_array(&env, &[5]),
        &env,
    );

    let after_add = OBSERVER_FIRE_COUNT.load(Ordering::Relaxed);
    assert_eq!(after_add - before, 1); // only "pos" observer fires

    // Extract world and feed into GameApp
    let inner_world = observed.into_inner();
    let mut app = GameApp::with_world(inner_world);
    app.add_system("physics", setup_physics_system);
    app.run(&env).unwrap();

    // Physics applied: 10+5=15
    let pos = app
        .world()
        .get_component(e1, &symbol_short!("pos"))
        .unwrap();
    assert_eq!(pos.get(0).unwrap(), 15);
}

#[test]
fn test_change_tracker_and_query_cache_cycle() {
    let env = Env::default();
    let world = SimpleWorld::new(&env);
    let mut tracked = TrackedWorld::new(world);
    let mut cache = SimpleQueryCache::new(symbol_short!("pos"), &env);

    // Tick 1: spawn + add
    let e1 = tracked.spawn_entity();
    tracked.add_component(e1, symbol_short!("pos"), Bytes::from_array(&env, &[10]));
    let e2 = tracked.spawn_entity();
    tracked.add_component(e2, symbol_short!("pos"), Bytes::from_array(&env, &[20]));

    let results = cache.execute(tracked.world(), &env);
    assert_eq!(results.len(), 2);
    assert!(tracked.tracker().was_added(e1, &symbol_short!("pos")));
    assert!(tracked.tracker().was_added(e2, &symbol_short!("pos")));

    // Clear tracker at end of tick
    tracked.tracker_mut().clear();
    tracked.tracker_mut().advance_tick();
    assert_eq!(tracked.tracker().tick(), 1);

    // Tick 2: modify e1
    tracked.add_component(e1, symbol_short!("pos"), Bytes::from_array(&env, &[15]));
    assert!(tracked.tracker().was_modified(e1, &symbol_short!("pos")));
    assert!(!tracked.tracker().was_added(e1, &symbol_short!("pos")));

    // Cache still valid length-wise after modify (version changed though)
    let results = cache.execute(tracked.world(), &env);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_multiple_plugins_sharing_world() {
    struct PluginA;
    impl Plugin for PluginA {
        fn name(&self) -> &'static str {
            "plugin_a"
        }
        fn build(&self, app: &mut GameApp) {
            // Plugin A sets up initial entities
            let e = app.world_mut().spawn_entity();
            let env = app.world().env().clone();
            app.world_mut().add_component(
                e,
                symbol_short!("a_comp"),
                Bytes::from_array(&env, &[42]),
            );
        }
    }

    struct PluginB;
    impl Plugin for PluginB {
        fn name(&self) -> &'static str {
            "plugin_b"
        }
        fn build(&self, app: &mut GameApp) {
            app.add_system("b_system", |world: &mut SimpleWorld, env: &Env| {
                // Plugin B's system reads entities from Plugin A
                let entities = world.get_entities_with_component(&symbol_short!("a_comp"), env);
                for i in 0..entities.len() {
                    let eid = entities.get(i).unwrap();
                    let data = Bytes::from_array(env, &[99]);
                    world.add_component(eid, symbol_short!("b_comp"), data);
                }
            });
        }
    }

    let env = Env::default();
    let mut app = GameApp::new(&env);
    app.add_plugin(PluginA);
    app.add_plugin(PluginB);

    // Entity created by PluginA during build
    assert!(app.world().has_component(1, &symbol_short!("a_comp")));

    // Run PluginB's system
    app.run(&env).unwrap();

    // PluginB's system added its component
    assert!(app.world().has_component(1, &symbol_short!("b_comp")));
}

#[test]
fn test_tracked_world_despawn_with_query_cache() {
    let env = Env::default();
    let world = SimpleWorld::new(&env);
    let mut tracked = TrackedWorld::new(world);
    let mut cache = SimpleQueryCache::new(symbol_short!("pos"), &env);

    // Create 3 entities
    let e1 = tracked.spawn_entity();
    tracked.add_component(e1, symbol_short!("pos"), Bytes::from_array(&env, &[1]));
    let e2 = tracked.spawn_entity();
    tracked.add_component(e2, symbol_short!("pos"), Bytes::from_array(&env, &[2]));
    let e3 = tracked.spawn_entity();
    tracked.add_component(e3, symbol_short!("pos"), Bytes::from_array(&env, &[3]));

    let results = cache.execute(tracked.world(), &env);
    assert_eq!(results.len(), 3);

    // Despawn e2 through tracked world
    tracked.tracker_mut().clear();
    tracked.despawn_entity(e2);
    assert!(tracked.tracker().was_removed(e2, &symbol_short!("pos")));

    // Cache should be stale and return 2
    let results = cache.execute(tracked.world(), &env);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_command_queue_sparse_storage_integration() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    let e1 = world.spawn_entity();

    let mut queue = CommandQueue::new();
    queue.add_component(e1, symbol_short!("pos"), Bytes::from_array(&env, &[1]));
    queue.add_sparse_component(e1, symbol_short!("tag"), Bytes::from_array(&env, &[2]));
    queue.apply(&mut world);

    assert_eq!(world.table_component_count(&symbol_short!("pos")), 1);
    assert_eq!(world.table_component_count(&symbol_short!("tag")), 0);
    assert_eq!(world.component_count(&symbol_short!("tag")), 1);
    assert!(world.has_component(e1, &symbol_short!("pos")));
    assert!(world.has_component(e1, &symbol_short!("tag")));
}

#[test]
fn test_scheduler_with_command_queue_pattern() {
    /// System that uses CommandQueue internally.
    fn spawner_system(world: &mut SimpleWorld, env: &Env) {
        let mut cmds = CommandQueue::new();
        cmds.spawn();
        let spawned = cmds.apply(world);
        for id in &spawned {
            world.add_component(*id, symbol_short!("spawned"), Bytes::from_array(env, &[1]));
        }
    }

    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let mut scheduler = SimpleScheduler::new();
    scheduler.add_system("spawner", spawner_system);

    // Run 3 ticks - each spawns one entity
    scheduler.run_all(&mut world, &env).unwrap();
    scheduler.run_all(&mut world, &env).unwrap();
    scheduler.run_all(&mut world, &env).unwrap();

    let spawned_entities = world.get_entities_with_component(&symbol_short!("spawned"), &env);
    assert_eq!(spawned_entities.len(), 3);
}
