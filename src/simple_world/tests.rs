use super::SimpleWorld;
use crate::component::{ComponentStorage, Position};
use soroban_sdk::{symbol_short, Bytes, Env};

#[test]
fn test_simple_world_creation() {
    let env = Env::default();
    let world = SimpleWorld::new(&env);
    assert_eq!(world.next_entity_id, 1);
    assert_eq!(world.components.len(), 0);
    assert_eq!(world.version(), 0);
}

#[test]
fn test_spawn_entity() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let id1 = world.spawn_entity();
    let id2 = world.spawn_entity();
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

#[test]
fn test_add_and_get_component() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let entity_id = world.spawn_entity();

    let data = Bytes::from_array(&env, &[1, 2, 3, 4]);
    world.add_component(entity_id, symbol_short!("test"), data.clone());

    let retrieved = world.get_component(entity_id, &symbol_short!("test"));
    assert_eq!(retrieved, Some(data));
}

#[test]
fn test_has_component() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let entity_id = world.spawn_entity();

    assert!(!world.has_component(entity_id, &symbol_short!("test")));

    let data = Bytes::from_array(&env, &[1]);
    world.add_component(entity_id, symbol_short!("test"), data);

    assert!(world.has_component(entity_id, &symbol_short!("test")));
}

#[test]
fn test_remove_component() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let entity_id = world.spawn_entity();

    let data = Bytes::from_array(&env, &[1]);
    world.add_component(entity_id, symbol_short!("test"), data);

    assert!(world.remove_component(entity_id, &symbol_short!("test")));
    assert!(!world.has_component(entity_id, &symbol_short!("test")));
    assert!(!world.remove_component(entity_id, &symbol_short!("test")));
}

#[test]
fn test_get_entities_with_component() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    let e1 = world.spawn_entity();
    let e2 = world.spawn_entity();
    let e3 = world.spawn_entity();

    let data = Bytes::from_array(&env, &[1]);
    world.add_component(e1, symbol_short!("pos"), data.clone());
    world.add_component(e2, symbol_short!("pos"), data.clone());
    world.add_component(e3, symbol_short!("vel"), data);

    let entities = world.get_entities_with_component(&symbol_short!("pos"), &env);
    assert_eq!(entities.len(), 2);
    assert_eq!(world.table_component_count(&symbol_short!("pos")), 2);
    assert_eq!(world.component_count(&symbol_short!("pos")), 2);
}

#[test]
fn test_despawn_entity() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let entity_id = world.spawn_entity();

    let data = Bytes::from_array(&env, &[1]);
    world.add_component(entity_id, symbol_short!("a"), data.clone());
    world.add_component(entity_id, symbol_short!("b"), data);

    world.despawn_entity(entity_id);
    assert!(!world.has_component(entity_id, &symbol_short!("a")));
    assert!(!world.has_component(entity_id, &symbol_short!("b")));
}

#[test]
fn test_version_increments_on_mutations() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    assert_eq!(world.version(), 0);

    let e1 = world.spawn_entity();
    let data = Bytes::from_array(&env, &[1]);
    world.add_component(e1, symbol_short!("test"), data);
    assert_eq!(world.version(), 1);

    world.remove_component(e1, &symbol_short!("test"));
    assert_eq!(world.version(), 2);

    let data2 = Bytes::from_array(&env, &[2]);
    world.add_component(e1, symbol_short!("a"), data2);
    assert_eq!(world.version(), 3);

    world.despawn_entity(e1);
    assert_eq!(world.version(), 4);
}

#[test]
fn test_sparse_component_storage() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let e1 = world.spawn_entity();

    let data = Bytes::from_array(&env, &[1]);
    world.add_component_with_storage(
        e1,
        symbol_short!("marker"),
        data.clone(),
        ComponentStorage::Sparse,
    );

    assert!(world.has_component(e1, &symbol_short!("marker")));
    assert_eq!(
        world.get_component(e1, &symbol_short!("marker")),
        Some(data)
    );

    assert!(!world.components.contains_key((e1, symbol_short!("marker"))));
    assert!(world
        .sparse_components
        .contains_key((e1, symbol_short!("marker"))));
    assert_eq!(world.table_component_count(&symbol_short!("marker")), 0);
    assert_eq!(world.component_count(&symbol_short!("marker")), 1);
}

#[test]
fn test_indices_follow_storage_migration() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let entity = world.spawn_entity();
    let data = Bytes::from_array(&env, &[7]);

    world.add_component(entity, symbol_short!("state"), data.clone());
    assert_eq!(world.table_component_count(&symbol_short!("state")), 1);
    assert_eq!(world.component_count(&symbol_short!("state")), 1);

    world.add_component_with_storage(
        entity,
        symbol_short!("state"),
        data,
        ComponentStorage::Sparse,
    );
    assert_eq!(world.table_component_count(&symbol_short!("state")), 0);
    assert_eq!(world.component_count(&symbol_short!("state")), 1);
}

#[test]
fn test_sparse_vs_table_queries() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);

    let e1 = world.spawn_entity();
    let e2 = world.spawn_entity();

    let data = Bytes::from_array(&env, &[1]);
    world.add_component(e1, symbol_short!("pos"), data.clone());
    world.add_component_with_storage(
        e1,
        symbol_short!("tag"),
        data.clone(),
        ComponentStorage::Sparse,
    );
    world.add_component_with_storage(e2, symbol_short!("tag"), data, ComponentStorage::Sparse);

    let table_only = world.get_table_entities_with_component(&symbol_short!("pos"), &env);
    assert_eq!(table_only.len(), 1);

    let all_pos = world.get_entities_with_component(&symbol_short!("pos"), &env);
    assert_eq!(all_pos.len(), 1);

    let all_tag = world.get_all_entities_with_component(&symbol_short!("tag"), &env);
    assert_eq!(all_tag.len(), 2);

    let table_tag = world.get_table_entities_with_component(&symbol_short!("tag"), &env);
    assert_eq!(table_tag.len(), 0);
}

#[test]
fn test_remove_sparse_component() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let e1 = world.spawn_entity();

    let data = Bytes::from_array(&env, &[1]);
    world.add_component_with_storage(e1, symbol_short!("tag"), data, ComponentStorage::Sparse);
    assert!(world.has_component(e1, &symbol_short!("tag")));

    assert!(world.remove_component(e1, &symbol_short!("tag")));
    assert!(!world.has_component(e1, &symbol_short!("tag")));
}

#[test]
fn test_despawn_clears_both_maps() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let e1 = world.spawn_entity();

    let data = Bytes::from_array(&env, &[1]);
    world.add_component(e1, symbol_short!("pos"), data.clone());
    world.add_component_with_storage(e1, symbol_short!("tag"), data, ComponentStorage::Sparse);

    world.despawn_entity(e1);
    assert!(!world.has_component(e1, &symbol_short!("pos")));
    assert!(!world.has_component(e1, &symbol_short!("tag")));
}

#[test]
fn test_add_component_replaces_existing() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let entity_id = world.spawn_entity();

    let data1 = Bytes::from_array(&env, &[1]);
    let data2 = Bytes::from_array(&env, &[2]);

    world.add_component(entity_id, symbol_short!("test"), data1);
    world.add_component(entity_id, symbol_short!("test"), data2.clone());

    let retrieved = world.get_component(entity_id, &symbol_short!("test"));
    assert_eq!(retrieved, Some(data2));
}

#[test]
fn test_set_and_get_typed() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let e = world.spawn_entity();

    let pos = Position::new(10, 20);
    world.set_typed(&env, e, &pos);

    let retrieved: Option<Position> = world.get_typed(&env, e);
    assert!(retrieved.is_some());
    let r = retrieved.unwrap();
    assert_eq!(r.x, 10);
    assert_eq!(r.y, 20);
}

#[test]
fn test_has_typed() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let e = world.spawn_entity();

    assert!(!world.has_typed::<Position>(e));
    world.set_typed(&env, e, &Position::new(1, 2));
    assert!(world.has_typed::<Position>(e));
}

#[test]
fn test_remove_typed() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let e = world.spawn_entity();

    world.set_typed(&env, e, &Position::new(1, 2));
    assert!(world.remove_typed::<Position>(e));
    assert!(!world.has_typed::<Position>(e));
    assert!(!world.remove_typed::<Position>(e));
}

#[test]
fn test_get_typed_nonexistent() {
    let env = Env::default();
    let world = SimpleWorld::new(&env);
    let result: Option<Position> = world.get_typed(&env, 999);
    assert!(result.is_none());
}

#[test]
fn test_set_typed_overwrites() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let e = world.spawn_entity();

    world.set_typed(&env, e, &Position::new(1, 2));
    world.set_typed(&env, e, &Position::new(50, 60));

    let pos: Position = world.get_typed(&env, e).unwrap();
    assert_eq!(pos.x, 50);
    assert_eq!(pos.y, 60);
}
