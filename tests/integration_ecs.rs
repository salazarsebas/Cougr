//! Integration tests for the core ECS subsystem.
//!
//! Tests full game-loop patterns: spawn -> add components -> run systems -> query -> verify.

use cougr_core::scheduler::SimpleScheduler;
use cougr_core::{
    query::SimpleQueryCache, CommandQueue, ComponentStorage, GameApp, Plugin, SimpleWorld,
};
use soroban_sdk::{symbol_short, Bytes, Env};

// ---------------------------------------------------------------------------
// Helper system functions
// ---------------------------------------------------------------------------

/// A physics system that reads "pos" + "vel" and updates "pos".
fn physics_system(world: &mut SimpleWorld, env: &Env) {
    let entities = world.get_entities_with_component(&symbol_short!("pos"), env);
    for i in 0..entities.len() {
        let eid = entities.get(i).unwrap();
        if let (Some(pos_data), Some(vel_data)) = (
            world.get_component(eid, &symbol_short!("pos")),
            world.get_component(eid, &symbol_short!("vel")),
        ) {
            // pos = pos + vel (byte-level add for simplicity)
            let px = pos_data.get(0).unwrap_or(0);
            let vx = vel_data.get(0).unwrap_or(0);
            let new_pos = Bytes::from_array(env, &[px.wrapping_add(vx)]);
            world.add_component(eid, symbol_short!("pos"), new_pos);
        }
    }
}

/// A scoring system that adds a "score" component to entities with "scored" marker.
fn scoring_system(world: &mut SimpleWorld, env: &Env) {
    let entities = world.get_entities_with_component(&symbol_short!("scored"), env);
    for i in 0..entities.len() {
        let eid = entities.get(i).unwrap();
        let current = world
            .get_component(eid, &symbol_short!("score"))
            .map(|b| b.get(0).unwrap_or(0))
            .unwrap_or(0);
        let new_score = Bytes::from_array(env, &[current + 1]);
        world.add_component(eid, symbol_short!("score"), new_score);
    }
}

/// A cleanup system that removes "dead" entities.
fn cleanup_system(world: &mut SimpleWorld, env: &Env) {
    let entities = world.get_entities_with_component(&symbol_short!("dead"), env);
    for i in 0..entities.len() {
        let eid = entities.get(i).unwrap();
        world.despawn_entity(eid);
    }
}

/// A spawn system that creates new entities via CommandQueue.
fn spawn_bullets_system(world: &mut SimpleWorld, env: &Env) {
    let mut cmds = CommandQueue::new();
    cmds.spawn();
    let spawned = cmds.apply(world);
    for id in &spawned {
        let data = Bytes::from_array(env, &[0xFF]);
        world.add_component(*id, symbol_short!("bullet"), data);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_full_game_tick_cycle() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    // Spawn entities with position + velocity
    let e1 = world.spawn_entity();
    world.add_component(e1, symbol_short!("pos"), Bytes::from_array(&env, &[10]));
    world.add_component(e1, symbol_short!("vel"), Bytes::from_array(&env, &[5]));

    let e2 = world.spawn_entity();
    world.add_component(e2, symbol_short!("pos"), Bytes::from_array(&env, &[20]));
    world.add_component(e2, symbol_short!("vel"), Bytes::from_array(&env, &[3]));

    // Run physics
    physics_system(&mut world, &env);

    // Verify positions updated: 10+5=15, 20+3=23
    let pos1 = world.get_component(e1, &symbol_short!("pos")).unwrap();
    assert_eq!(pos1.get(0).unwrap(), 15);

    let pos2 = world.get_component(e2, &symbol_short!("pos")).unwrap();
    assert_eq!(pos2.get(0).unwrap(), 23);

    // Query: both entities still have "pos"
    let with_pos = world.get_entities_with_component(&symbol_short!("pos"), &env);
    assert_eq!(with_pos.len(), 2);
}

#[test]
fn test_plugin_app_lifecycle() {
    struct PhysicsPlugin;
    impl Plugin for PhysicsPlugin {
        fn name(&self) -> &'static str {
            "physics"
        }
        fn build(&self, app: &mut GameApp) {
            app.add_system("physics", physics_system);
        }
    }

    struct ScoringPlugin;
    impl Plugin for ScoringPlugin {
        fn name(&self) -> &'static str {
            "scoring"
        }
        fn build(&self, app: &mut GameApp) {
            app.add_system("scoring", scoring_system);
        }
    }

    let env = Env::default();
    let mut app = GameApp::new(&env);
    app.add_plugin(PhysicsPlugin);
    app.add_plugin(ScoringPlugin);

    assert_eq!(app.plugin_count(), 2);
    assert_eq!(app.system_count(), 2);

    // Set up entities
    let e1 = app.world_mut().spawn_entity();
    app.world_mut()
        .add_component(e1, symbol_short!("pos"), Bytes::from_array(&env, &[10]));
    app.world_mut()
        .add_component(e1, symbol_short!("vel"), Bytes::from_array(&env, &[5]));
    app.world_mut()
        .add_component(e1, symbol_short!("scored"), Bytes::from_array(&env, &[1]));

    // Run all systems
    app.run(&env).unwrap();

    // Physics updated pos: 10+5=15
    let pos = app
        .world()
        .get_component(e1, &symbol_short!("pos"))
        .unwrap();
    assert_eq!(pos.get(0).unwrap(), 15);

    // Scoring added score: 0+1=1
    let score = app
        .world()
        .get_component(e1, &symbol_short!("score"))
        .unwrap();
    assert_eq!(score.get(0).unwrap(), 1);

    // Extract world
    let world = app.into_world();
    assert!(world.has_component(e1, &symbol_short!("pos")));
    assert!(world.has_component(e1, &symbol_short!("score")));
}

#[test]
fn test_command_queue_within_system() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    // Spawn a shooter entity
    let shooter = world.spawn_entity();
    world.add_component(
        shooter,
        symbol_short!("pos"),
        Bytes::from_array(&env, &[50]),
    );

    // Run the system that spawns bullets
    spawn_bullets_system(&mut world, &env);

    // Verify bullet was spawned (entity 2)
    assert!(world.has_component(2, &symbol_short!("bullet")));
    // Shooter still exists
    assert!(world.has_component(shooter, &symbol_short!("pos")));
}

#[test]
fn test_multi_system_pipeline() {
    let env = Env::default();
    let mut scheduler = SimpleScheduler::new();
    scheduler.add_system("physics", physics_system);
    scheduler.add_system("scoring", scoring_system);
    scheduler.add_system("cleanup", cleanup_system);

    let mut world = SimpleWorld::new(&env);

    // Entity 1: alive with pos+vel+scored
    let e1 = world.spawn_entity();
    world.add_component(e1, symbol_short!("pos"), Bytes::from_array(&env, &[10]));
    world.add_component(e1, symbol_short!("vel"), Bytes::from_array(&env, &[5]));
    world.add_component(e1, symbol_short!("scored"), Bytes::from_array(&env, &[1]));

    // Entity 2: dead
    let e2 = world.spawn_entity();
    world.add_component(e2, symbol_short!("pos"), Bytes::from_array(&env, &[99]));
    world.add_component(e2, symbol_short!("dead"), Bytes::from_array(&env, &[1]));

    scheduler.run_all(&mut world, &env).unwrap();

    // e1 physics applied: 10+5=15
    let pos = world.get_component(e1, &symbol_short!("pos")).unwrap();
    assert_eq!(pos.get(0).unwrap(), 15);

    // e1 scoring applied
    assert!(world.has_component(e1, &symbol_short!("score")));

    // e2 cleaned up
    assert!(!world.has_component(e2, &symbol_short!("pos")));
    assert!(!world.has_component(e2, &symbol_short!("dead")));
}

#[test]
fn test_table_vs_sparse_mixed_storage() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    let e1 = world.spawn_entity();
    let e2 = world.spawn_entity();
    let e3 = world.spawn_entity();

    let data = Bytes::from_array(&env, &[1]);

    // e1: Table "pos" + Sparse "tag"
    world.add_component(e1, symbol_short!("pos"), data.clone());
    world.add_component_with_storage(
        e1,
        symbol_short!("tag"),
        data.clone(),
        ComponentStorage::Sparse,
    );

    // e2: Only Table "pos"
    world.add_component(e2, symbol_short!("pos"), data.clone());

    // e3: Only Sparse "tag"
    world.add_component_with_storage(
        e3,
        symbol_short!("tag"),
        data.clone(),
        ComponentStorage::Sparse,
    );

    // Table query for "pos": e1, e2
    let table_pos = world.get_table_entities_with_component(&symbol_short!("pos"), &env);
    assert_eq!(table_pos.len(), 2);

    // All query for "tag": e1, e3
    let all_tag = world.get_all_entities_with_component(&symbol_short!("tag"), &env);
    assert_eq!(all_tag.len(), 2);

    // Table-only query for "tag": empty (both are sparse)
    let table_tag = world.get_table_entities_with_component(&symbol_short!("tag"), &env);
    assert_eq!(table_tag.len(), 0);

    // has_component works transparently
    assert!(world.has_component(e1, &symbol_short!("pos")));
    assert!(world.has_component(e1, &symbol_short!("tag")));
    assert!(world.has_component(e3, &symbol_short!("tag")));
    assert!(!world.has_component(e3, &symbol_short!("pos")));
}

#[test]
fn test_entity_complete_lifecycle() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    // 1. Spawn
    let e = world.spawn_entity();
    assert_eq!(e, 1);

    // 2. Add components
    world.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[10, 20]));
    world.add_component(e, symbol_short!("hp"), Bytes::from_array(&env, &[100]));
    world.add_component(e, symbol_short!("name"), Bytes::from_array(&env, &[65, 66]));

    assert!(world.has_component(e, &symbol_short!("pos")));
    assert!(world.has_component(e, &symbol_short!("hp")));
    assert!(world.has_component(e, &symbol_short!("name")));

    // 3. Remove some
    world.remove_component(e, &symbol_short!("name"));
    assert!(!world.has_component(e, &symbol_short!("name")));
    assert!(world.has_component(e, &symbol_short!("pos")));

    // 4. Modify existing
    world.add_component(e, symbol_short!("hp"), Bytes::from_array(&env, &[50]));
    let hp = world.get_component(e, &symbol_short!("hp")).unwrap();
    assert_eq!(hp.get(0).unwrap(), 50);

    // 5. Despawn
    world.despawn_entity(e);
    assert!(!world.has_component(e, &symbol_short!("pos")));
    assert!(!world.has_component(e, &symbol_short!("hp")));
}

#[test]
fn test_query_cache_with_mutations() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let mut cache = SimpleQueryCache::new(symbol_short!("pos"), &env);

    // Empty world
    let results = cache.execute(&world, &env);
    assert_eq!(results.len(), 0);

    // Add entity with "pos"
    let e1 = world.spawn_entity();
    world.add_component(e1, symbol_short!("pos"), Bytes::from_array(&env, &[1]));

    // Cache should be stale
    assert!(!cache.is_valid(world.version()));

    let results = cache.execute(&world, &env);
    assert_eq!(results.len(), 1);
    assert!(cache.is_valid(world.version()));

    // Second call without mutations returns cached
    let results2 = cache.execute(&world, &env);
    assert_eq!(results2.len(), 1);

    // Add another entity
    let e2 = world.spawn_entity();
    world.add_component(e2, symbol_short!("pos"), Bytes::from_array(&env, &[2]));
    let results3 = cache.execute(&world, &env);
    assert_eq!(results3.len(), 2);

    // Remove entity
    world.despawn_entity(e1);
    let results4 = cache.execute(&world, &env);
    assert_eq!(results4.len(), 1);
}

#[test]
fn test_scheduler_system_ordering() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    // Spawn entity with pos=10, vel=5
    let e = world.spawn_entity();
    world.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[10]));
    world.add_component(e, symbol_short!("vel"), Bytes::from_array(&env, &[5]));
    world.add_component(e, symbol_short!("scored"), Bytes::from_array(&env, &[1]));

    let mut scheduler = SimpleScheduler::new();
    // Systems run in insertion order
    scheduler.add_system("physics", physics_system);
    scheduler.add_system("scoring", scoring_system);

    // Tick 1
    scheduler.run_all(&mut world, &env).unwrap();
    let pos = world.get_component(e, &symbol_short!("pos")).unwrap();
    assert_eq!(pos.get(0).unwrap(), 15);
    let score = world.get_component(e, &symbol_short!("score")).unwrap();
    assert_eq!(score.get(0).unwrap(), 1);

    // Tick 2
    scheduler.run_all(&mut world, &env).unwrap();
    let pos = world.get_component(e, &symbol_short!("pos")).unwrap();
    assert_eq!(pos.get(0).unwrap(), 20); // 15 + 5
    let score = world.get_component(e, &symbol_short!("score")).unwrap();
    assert_eq!(score.get(0).unwrap(), 2);
}

#[test]
fn test_command_queue_mixed_operations() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    let e1 = world.spawn_entity();
    world.add_component(e1, symbol_short!("old"), Bytes::from_array(&env, &[1]));

    let mut queue = CommandQueue::new();
    queue.spawn(); // will become e2
    queue.add_component(e1, symbol_short!("new"), Bytes::from_array(&env, &[2]));
    queue.remove_component(e1, symbol_short!("old"));
    queue.spawn(); // will become e3

    let spawned = queue.apply(&mut world);
    assert_eq!(spawned.len(), 2);
    assert_eq!(spawned[0], 2);
    assert_eq!(spawned[1], 3);

    assert!(world.has_component(e1, &symbol_short!("new")));
    assert!(!world.has_component(e1, &symbol_short!("old")));
}
