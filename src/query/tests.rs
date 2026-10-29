use super::*;
use soroban_sdk::{symbol_short, Env};

#[test]
fn test_simple_query_cache() {
    let env = Env::default();
    let mut world = crate::simple_world::SimpleWorld::new(&env);

    let e1 = world.spawn_entity();
    let data = soroban_sdk::Bytes::from_array(&env, &[1, 2, 3, 4]);
    world.add_component(e1, symbol_short!("pos"), data);

    let mut cache = SimpleQueryCache::new(symbol_short!("pos"), &env);

    let results = cache.execute(&world, &env);
    assert_eq!(results.len(), 1);
    assert!(cache.is_valid(world.version()));

    let results2 = cache.execute(&world, &env);
    assert_eq!(results2.len(), 1);

    let e2 = world.spawn_entity();
    let data2 = soroban_sdk::Bytes::from_array(&env, &[5, 6, 7, 8]);
    world.add_component(e2, symbol_short!("pos"), data2);
    assert!(!cache.is_valid(world.version()));

    let results3 = cache.execute(&world, &env);
    assert_eq!(results3.len(), 2);
    assert!(cache.is_valid(world.version()));
}

#[test]
fn test_simple_query_cache_invalidate() {
    let env = Env::default();
    let mut cache = SimpleQueryCache::new(symbol_short!("test"), &env);
    let mut world = crate::simple_world::SimpleWorld::new(&env);
    let entity = world.spawn_entity();
    world.add_component(
        entity,
        symbol_short!("test"),
        soroban_sdk::Bytes::from_array(&env, &[1]),
    );
    let _ = cache.execute(&world, &env);
    assert!(cache.is_valid(world.version()));
    cache.invalidate();
    assert!(!cache.is_valid(world.version()));
}

#[test]
fn test_simple_query_builder_with_sparse_and_any() {
    let env = Env::default();
    let mut world = crate::simple_world::SimpleWorld::new(&env);

    let e1 = world.spawn_entity();
    world.add_component(
        e1,
        symbol_short!("pos"),
        soroban_sdk::Bytes::from_array(&env, &[1]),
    );

    let e2 = world.spawn_entity();
    world.add_component_with_storage(
        e2,
        symbol_short!("tag"),
        soroban_sdk::Bytes::from_array(&env, &[2]),
        crate::component::ComponentStorage::Sparse,
    );

    let query = SimpleQueryBuilder::new(&env)
        .with_any_component(symbol_short!("pos"))
        .with_any_component(symbol_short!("tag"))
        .include_sparse()
        .build();

    let results = query.execute(&world, &env);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_simple_query_state_tracks_world_version() {
    let env = Env::default();
    let mut world = crate::simple_world::SimpleWorld::new(&env);
    let query = SimpleQueryBuilder::new(&env)
        .with_component(symbol_short!("pos"))
        .build();
    let mut state = SimpleQueryState::new(query, &env);

    assert_eq!(state.execute(&world, &env).len(), 0);
    assert!(state.is_valid(world.version()));

    let entity = world.spawn_entity();
    world.add_component(
        entity,
        symbol_short!("pos"),
        soroban_sdk::Bytes::from_array(&env, &[3]),
    );

    assert!(!state.is_valid(world.version()));
    assert_eq!(state.execute(&world, &env).len(), 1);
}

#[test]
fn test_simple_query_bulk_filters_and_candidate_count() {
    let env = Env::default();
    let mut world = crate::simple_world::SimpleWorld::new(&env);

    let e1 = world.spawn_entity();
    world.add_component(
        e1,
        symbol_short!("pos"),
        soroban_sdk::Bytes::from_array(&env, &[1]),
    );
    world.add_component(
        e1,
        symbol_short!("vel"),
        soroban_sdk::Bytes::from_array(&env, &[2]),
    );

    let e2 = world.spawn_entity();
    world.add_component(
        e2,
        symbol_short!("pos"),
        soroban_sdk::Bytes::from_array(&env, &[3]),
    );
    world.add_component_with_storage(
        e2,
        symbol_short!("sleep"),
        soroban_sdk::Bytes::from_array(&env, &[4]),
        crate::component::ComponentStorage::Sparse,
    );

    let query = SimpleQueryBuilder::new(&env)
        .with_components(&[symbol_short!("pos")])
        .without_components(&[symbol_short!("sleep")])
        .with_any_components(&[symbol_short!("vel"), symbol_short!("sleep")])
        .include_sparse()
        .build();

    assert_eq!(query.candidate_count(&world, &env), 2);
    let results = query.execute(&world, &env);
    assert_eq!(results.len(), 1);
    assert_eq!(results.get(0), Some(e1));
}
