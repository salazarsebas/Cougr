use super::ArchetypeWorld;
use crate::component::Position;
use crate::simple_world::SimpleWorld;
use soroban_sdk::{symbol_short, Bytes, Env};

#[test]
fn test_new_world() {
    let env = Env::default();
    let world = ArchetypeWorld::new(&env);
    assert_eq!(world.next_entity_id, 1);
    assert_eq!(world.version(), 0);
}

#[test]
fn test_spawn_entity() {
    let env = Env::default();
    let mut world = ArchetypeWorld::new(&env);
    let e1 = world.spawn_entity();
    let e2 = world.spawn_entity();
    assert_eq!(e1, 1);
    assert_eq!(e2, 2);
}

#[test]
fn test_add_single_component() {
    let env = Env::default();
    let mut world = ArchetypeWorld::new(&env);
    let e1 = world.spawn_entity();

    world.add_component(
        e1,
        symbol_short!("pos"),
        Bytes::from_array(&env, &[1, 2]),
        &env,
    );

    assert!(world.has_component(e1, &symbol_short!("pos")));
    assert_eq!(
        world.get_component(e1, &symbol_short!("pos")),
        Some(Bytes::from_array(&env, &[1, 2]))
    );
}

#[test]
fn test_add_multiple_components() {
    let env = Env::default();
    let mut world = ArchetypeWorld::new(&env);
    let e1 = world.spawn_entity();

    world.add_component(
        e1,
        symbol_short!("pos"),
        Bytes::from_array(&env, &[1]),
        &env,
    );
    world.add_component(
        e1,
        symbol_short!("vel"),
        Bytes::from_array(&env, &[2]),
        &env,
    );

    assert!(world.has_component(e1, &symbol_short!("pos")));
    assert!(world.has_component(e1, &symbol_short!("vel")));
}

#[test]
fn test_entities_share_archetype() {
    let env = Env::default();
    let mut world = ArchetypeWorld::new(&env);
    let e1 = world.spawn_entity();
    let e2 = world.spawn_entity();

    world.add_component(
        e1,
        symbol_short!("pos"),
        Bytes::from_array(&env, &[1]),
        &env,
    );
    world.add_component(
        e1,
        symbol_short!("vel"),
        Bytes::from_array(&env, &[2]),
        &env,
    );
    world.add_component(
        e2,
        symbol_short!("pos"),
        Bytes::from_array(&env, &[3]),
        &env,
    );
    world.add_component(
        e2,
        symbol_short!("vel"),
        Bytes::from_array(&env, &[4]),
        &env,
    );

    let a1 = world.entity_archetype.get(e1).unwrap();
    let a2 = world.entity_archetype.get(e2).unwrap();
    assert_eq!(a1, a2);
}

#[test]
fn test_query() {
    let env = Env::default();
    let mut world = ArchetypeWorld::new(&env);

    let e1 = world.spawn_entity();
    let e2 = world.spawn_entity();
    let e3 = world.spawn_entity();

    world.add_component(
        e1,
        symbol_short!("pos"),
        Bytes::from_array(&env, &[1]),
        &env,
    );
    world.add_component(
        e1,
        symbol_short!("vel"),
        Bytes::from_array(&env, &[2]),
        &env,
    );
    world.add_component(
        e2,
        symbol_short!("pos"),
        Bytes::from_array(&env, &[3]),
        &env,
    );
    world.add_component(
        e3,
        symbol_short!("vel"),
        Bytes::from_array(&env, &[4]),
        &env,
    );

    let with_pos = world.query(&[symbol_short!("pos")], &env);
    assert_eq!(with_pos.len(), 2);

    let with_vel = world.query(&[symbol_short!("vel")], &env);
    assert_eq!(with_vel.len(), 2);

    let with_both = world.query(&[symbol_short!("pos"), symbol_short!("vel")], &env);
    assert_eq!(with_both.len(), 1);
    assert_eq!(with_both.get(0), Some(e1));
}

#[test]
fn test_remove_component() {
    let env = Env::default();
    let mut world = ArchetypeWorld::new(&env);
    let e1 = world.spawn_entity();

    world.add_component(
        e1,
        symbol_short!("pos"),
        Bytes::from_array(&env, &[1]),
        &env,
    );
    world.add_component(
        e1,
        symbol_short!("vel"),
        Bytes::from_array(&env, &[2]),
        &env,
    );

    assert!(world.remove_component(e1, &symbol_short!("vel"), &env));
    assert!(!world.has_component(e1, &symbol_short!("vel")));
    assert!(world.has_component(e1, &symbol_short!("pos")));
    assert_eq!(
        world.get_component(e1, &symbol_short!("pos")),
        Some(Bytes::from_array(&env, &[1]))
    );
}

#[test]
fn test_remove_last_component() {
    let env = Env::default();
    let mut world = ArchetypeWorld::new(&env);
    let e1 = world.spawn_entity();

    world.add_component(
        e1,
        symbol_short!("pos"),
        Bytes::from_array(&env, &[1]),
        &env,
    );
    assert!(world.remove_component(e1, &symbol_short!("pos"), &env));

    assert!(world.entity_archetype.get(e1).is_none());
    assert!(!world.has_component(e1, &symbol_short!("pos")));
}

#[test]
fn test_remove_nonexistent_component() {
    let env = Env::default();
    let mut world = ArchetypeWorld::new(&env);
    let e1 = world.spawn_entity();

    assert!(!world.remove_component(e1, &symbol_short!("pos"), &env));
}

#[test]
fn test_despawn_entity() {
    let env = Env::default();
    let mut world = ArchetypeWorld::new(&env);
    let e1 = world.spawn_entity();

    world.add_component(
        e1,
        symbol_short!("pos"),
        Bytes::from_array(&env, &[1]),
        &env,
    );
    world.add_component(
        e1,
        symbol_short!("vel"),
        Bytes::from_array(&env, &[2]),
        &env,
    );

    world.despawn_entity(e1, &env);
    assert!(!world.has_component(e1, &symbol_short!("pos")));
    assert!(!world.has_component(e1, &symbol_short!("vel")));
    assert!(world.entity_archetype.get(e1).is_none());
}

#[test]
fn test_update_existing_component() {
    let env = Env::default();
    let mut world = ArchetypeWorld::new(&env);
    let e1 = world.spawn_entity();

    world.add_component(
        e1,
        symbol_short!("pos"),
        Bytes::from_array(&env, &[1]),
        &env,
    );
    world.add_component(
        e1,
        symbol_short!("pos"),
        Bytes::from_array(&env, &[99]),
        &env,
    );

    assert_eq!(
        world.get_component(e1, &symbol_short!("pos")),
        Some(Bytes::from_array(&env, &[99]))
    );
}

#[test]
fn test_version_tracking() {
    let env = Env::default();
    let mut world = ArchetypeWorld::new(&env);
    assert_eq!(world.version(), 0);

    let e1 = world.spawn_entity();
    world.add_component(
        e1,
        symbol_short!("pos"),
        Bytes::from_array(&env, &[1]),
        &env,
    );
    let v1 = world.version();
    assert!(v1 > 0);

    world.remove_component(e1, &symbol_short!("pos"), &env);
    assert!(world.version() > v1);
}

#[test]
fn test_to_simple_world() {
    let env = Env::default();
    let mut world = ArchetypeWorld::new(&env);
    let e1 = world.spawn_entity();
    let e2 = world.spawn_entity();

    world.add_component(
        e1,
        symbol_short!("pos"),
        Bytes::from_array(&env, &[1]),
        &env,
    );
    world.add_component(
        e2,
        symbol_short!("vel"),
        Bytes::from_array(&env, &[2]),
        &env,
    );

    let simple = world.to_simple_world(&env);
    assert!(simple.has_component(e1, &symbol_short!("pos")));
    assert!(simple.has_component(e2, &symbol_short!("vel")));
    assert!(!simple.has_component(e1, &symbol_short!("vel")));
}

#[test]
fn test_from_simple_world() {
    let env = Env::default();
    let mut simple = SimpleWorld::new(&env);
    let e1 = simple.spawn_entity();
    let e2 = simple.spawn_entity();

    simple.add_component(e1, symbol_short!("pos"), Bytes::from_array(&env, &[10]));
    simple.add_component(e1, symbol_short!("vel"), Bytes::from_array(&env, &[20]));
    simple.add_component(e2, symbol_short!("pos"), Bytes::from_array(&env, &[30]));

    let arch_world = ArchetypeWorld::from_simple_world(&simple, &env);
    assert!(arch_world.has_component(e1, &symbol_short!("pos")));
    assert!(arch_world.has_component(e1, &symbol_short!("vel")));
    assert!(arch_world.has_component(e2, &symbol_short!("pos")));
    assert!(!arch_world.has_component(e2, &symbol_short!("vel")));

    let with_both = arch_world.query(&[symbol_short!("pos"), symbol_short!("vel")], &env);
    assert_eq!(with_both.len(), 1);
}

#[test]
fn test_many_entities_same_archetype() {
    let env = Env::default();
    let mut world = ArchetypeWorld::new(&env);

    for _ in 0..20 {
        let eid = world.spawn_entity();
        world.add_component(
            eid,
            symbol_short!("pos"),
            Bytes::from_array(&env, &[1]),
            &env,
        );
        world.add_component(
            eid,
            symbol_short!("vel"),
            Bytes::from_array(&env, &[2]),
            &env,
        );
    }

    let results = world.query(&[symbol_short!("pos"), symbol_short!("vel")], &env);
    assert_eq!(results.len(), 20);
}

#[test]
fn test_set_and_get_typed() {
    let env = Env::default();
    let mut world = ArchetypeWorld::new(&env);
    let e = world.spawn_entity();

    world.set_typed(&env, e, &Position::new(10, 20));

    let retrieved: Option<Position> = world.get_typed(&env, e);
    assert!(retrieved.is_some());
    let r = retrieved.unwrap();
    assert_eq!(r.x, 10);
    assert_eq!(r.y, 20);
}

#[test]
fn test_has_typed() {
    let env = Env::default();
    let mut world = ArchetypeWorld::new(&env);
    let e = world.spawn_entity();

    assert!(!world.has_typed::<Position>(e));
    world.set_typed(&env, e, &Position::new(1, 2));
    assert!(world.has_typed::<Position>(e));
}

#[test]
fn test_remove_typed() {
    let env = Env::default();
    let mut world = ArchetypeWorld::new(&env);
    let e = world.spawn_entity();

    world.set_typed(&env, e, &Position::new(1, 2));
    assert!(world.remove_typed::<Position>(&env, e));
    assert!(!world.has_typed::<Position>(e));
}
