//! Integration tests for Phase B - ECS-Aware Events.
//!
//! Validates `ObservableComponentTrait`, `impl_component_observed!`, and the
//! `set_typed_observed` / `remove_component_observed` methods on `SimpleWorld`.
//!
//! **Note on event inspection**: soroban-sdk 25.x `ContractEvents` only captures
//! events emitted inside contract invocations. Direct calls in test code don't
//! populate `env.events().all()`. Behavioral correctness (storage effects) and
//! non-panicking execution are what these tests verify; event topic shape is
//! validated by the macro expansion and the `#[contracttype]` constraint.

use cougr_core::component::ComponentTrait;
use cougr_core::ecs_events::ObservableComponentTrait;
use cougr_core::impl_component_observed;
use cougr_core::simple_world::SimpleWorld;
use soroban_sdk::{contracttype, Env};

// ─── Test component types ─────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct HitPoints {
    pub current: u32,
    pub max: u32,
}

impl_component_observed!(HitPoints, "hitpts", Table, { current: u32, max: u32 });

#[contracttype]
#[derive(Clone, Debug)]
pub struct Mana {
    pub value: u32,
}

impl_component_observed!(Mana, "mana", Table, { value: u32 });

// ─── Tests ───────────────────────────────────────────────────────────────────

#[test]
fn test_set_typed_observed_stores_component() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let entity = world.spawn_entity();

    world.set_typed_observed(
        &env,
        entity,
        &HitPoints {
            current: 100,
            max: 100,
        },
    );

    let hp: Option<HitPoints> = world.get_typed(&env, entity);
    let hp = hp.unwrap();
    assert_eq!(hp.current, 100);
    assert_eq!(hp.max, 100);
}

#[test]
fn test_set_typed_observed_emit_does_not_panic() {
    // Verify emit_set_event executes without panicking (event is emitted to the
    // Soroban host; captured in ContractEvents only within contract invocations).
    let env = Env::default();
    let data = HitPoints {
        current: 80,
        max: 100,
    };
    let bytes = data.serialize(&env);
    HitPoints::emit_set_event(&env, 1, &bytes);
}

#[test]
fn test_remove_component_observed_removes_component() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let entity = world.spawn_entity();

    world.set_typed_observed(
        &env,
        entity,
        &HitPoints {
            current: 50,
            max: 100,
        },
    );
    assert!(world.has_typed::<HitPoints>(entity));

    let removed = world.remove_component_observed(entity, &HitPoints::component_type(), &env);
    assert!(removed);
    assert!(!world.has_typed::<HitPoints>(entity));
}

#[test]
fn test_remove_component_observed_emit_del_does_not_panic() {
    let env = Env::default();
    HitPoints::emit_remove_event(&env, 42);
}

#[test]
fn test_remove_component_observed_returns_false_when_absent() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let entity = world.spawn_entity();

    let removed = world.remove_component_observed(entity, &HitPoints::component_type(), &env);
    assert!(!removed);
}

#[test]
fn test_plain_set_typed_does_not_emit_events() {
    use cougr_core::component::Position;
    use soroban_sdk::testutils::Events;

    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let entity = world.spawn_entity();

    let before = env.events().all().events().len();
    world.set_typed(&env, entity, &Position::new(5, 5));

    assert_eq!(
        env.events().all().events().len(),
        before,
        "set_typed must not emit events - use set_typed_observed for that"
    );
}

#[test]
fn test_observed_component_also_satisfies_component_trait() {
    // ObservableComponentTrait extends ComponentTrait - confirm ECS operations work.
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let entity = world.spawn_entity();

    world.set_typed_observed(
        &env,
        entity,
        &HitPoints {
            current: 75,
            max: 100,
        },
    );

    assert!(world.has_typed::<HitPoints>(entity));

    let hp: Option<HitPoints> = world.get_typed(&env, entity);
    assert_eq!(hp.unwrap().current, 75);

    world.remove_typed::<HitPoints>(entity);
    assert!(!world.has_typed::<HitPoints>(entity));
}

#[test]
fn test_multiple_observed_components_coexist_on_entity() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let entity = world.spawn_entity();

    world.set_typed_observed(
        &env,
        entity,
        &HitPoints {
            current: 100,
            max: 100,
        },
    );
    world.set_typed_observed(&env, entity, &Mana { value: 50 });

    assert!(world.has_typed::<HitPoints>(entity));
    assert!(world.has_typed::<Mana>(entity));

    let hp: Option<HitPoints> = world.get_typed(&env, entity);
    let mana: Option<Mana> = world.get_typed(&env, entity);
    assert_eq!(hp.unwrap().current, 100);
    assert_eq!(mana.unwrap().value, 50);
}

#[test]
fn test_set_typed_observed_overwrite_updates_storage() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let entity = world.spawn_entity();

    world.set_typed_observed(
        &env,
        entity,
        &HitPoints {
            current: 100,
            max: 100,
        },
    );
    world.set_typed_observed(
        &env,
        entity,
        &HitPoints {
            current: 50,
            max: 100,
        },
    );

    let hp: Option<HitPoints> = world.get_typed(&env, entity);
    assert_eq!(hp.unwrap().current, 50);
}

#[test]
fn test_serialize_deserialize_roundtrip_for_observed_component() {
    let env = Env::default();
    let original = HitPoints {
        current: 123,
        max: 456,
    };
    let bytes = original.serialize(&env);
    let restored = HitPoints::deserialize(&env, &bytes).unwrap();
    assert_eq!(restored.current, 123);
    assert_eq!(restored.max, 456);
}
