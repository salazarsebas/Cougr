//! Integration tests for Phase A - Rich Component Types.
//!
//! Validates `RichComponentTrait`, `impl_rich_component!`, and the
//! `set_rich` / `get_rich` / `remove_rich` methods on `SimpleWorld`.
//!
//! Rich component storage uses `env.storage().instance()` which requires a
//! contract context. Tests run inside `env.as_contract()` to satisfy this.

use cougr_core::impl_rich_component;
use cougr_core::rich_component::RichComponentTrait;
use cougr_core::simple_world::SimpleWorld;
use soroban_sdk::{contract, contractimpl, contracttype, Env, String, Vec};

// ─── Minimal test contract ────────────────────────────────────────────────────

#[contract]
struct WorldContract;

#[contractimpl]
impl WorldContract {}

// ─── Rich component types ─────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerProfile {
    pub name: String,
    pub scores: Vec<u32>,
    pub level: Option<u32>,
}

impl_rich_component!(PlayerProfile, "player_profile");

#[contracttype]
#[derive(Clone, Debug)]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct InventoryItem {
    pub item_id: u32,
    pub rarity: Rarity,
    pub count: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Inventory {
    pub items: Vec<InventoryItem>,
    pub capacity: u32,
}

impl_rich_component!(Inventory, "inventory_component");

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Register a throwaway test contract and return its address.
fn make_contract(env: &Env) -> soroban_sdk::Address {
    env.register(WorldContract, ())
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[test]
fn test_store_and_retrieve_rich_component() {
    let env = Env::default();
    let addr = make_contract(&env);

    env.as_contract(&addr, || {
        let mut world = SimpleWorld::new(&env);
        let entity = world.spawn_entity();

        let profile = PlayerProfile {
            name: String::from_str(&env, "Alice"),
            scores: Vec::from_array(&env, [100u32, 200u32, 300u32]),
            level: Some(42),
        };

        world.set_rich(&env, entity, &profile);
        let retrieved: Option<PlayerProfile> = world.get_rich(&env, entity);
        let retrieved = retrieved.unwrap();

        assert_eq!(retrieved.name, String::from_str(&env, "Alice"));
        assert_eq!(retrieved.level, Some(42));
        assert_eq!(retrieved.scores.len(), 3);
        assert_eq!(retrieved.scores.get(0), Some(100));
        assert_eq!(retrieved.scores.get(2), Some(300));
    });
}

#[test]
fn test_overwrite_rich_component_updates_value() {
    let env = Env::default();
    let addr = make_contract(&env);

    env.as_contract(&addr, || {
        let mut world = SimpleWorld::new(&env);
        let entity = world.spawn_entity();

        world.set_rich(
            &env,
            entity,
            &PlayerProfile {
                name: String::from_str(&env, "Alice"),
                scores: Vec::new(&env),
                level: None,
            },
        );

        world.set_rich(
            &env,
            entity,
            &PlayerProfile {
                name: String::from_str(&env, "Bob"),
                scores: Vec::new(&env),
                level: Some(10),
            },
        );

        let retrieved: Option<PlayerProfile> = world.get_rich(&env, entity);
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, String::from_str(&env, "Bob"));
        assert_eq!(retrieved.level, Some(10));
    });
}

#[test]
fn test_remove_rich_component() {
    let env = Env::default();
    let addr = make_contract(&env);

    env.as_contract(&addr, || {
        let mut world = SimpleWorld::new(&env);
        let entity = world.spawn_entity();

        world.set_rich(
            &env,
            entity,
            &PlayerProfile {
                name: String::from_str(&env, "Alice"),
                scores: Vec::new(&env),
                level: None,
            },
        );

        world.remove_rich::<PlayerProfile>(&env, entity);
        let retrieved: Option<PlayerProfile> = world.get_rich(&env, entity);
        assert!(retrieved.is_none());
    });
}

#[test]
fn test_get_absent_rich_component_returns_none() {
    let env = Env::default();
    let addr = make_contract(&env);

    env.as_contract(&addr, || {
        let world = SimpleWorld::new(&env);
        let result: Option<PlayerProfile> = world.get_rich(&env, 999);
        assert!(result.is_none());
    });
}

#[test]
fn test_nested_struct_with_enum_roundtrip() {
    let env = Env::default();
    let addr = make_contract(&env);

    env.as_contract(&addr, || {
        let mut world = SimpleWorld::new(&env);
        let entity = world.spawn_entity();

        let mut items: Vec<InventoryItem> = Vec::new(&env);
        items.push_back(InventoryItem {
            item_id: 1,
            rarity: Rarity::Legendary,
            count: 1,
        });
        items.push_back(InventoryItem {
            item_id: 2,
            rarity: Rarity::Common,
            count: 99,
        });

        let inv = Inventory {
            items,
            capacity: 100,
        };

        world.set_rich(&env, entity, &inv);
        let retrieved: Option<Inventory> = world.get_rich(&env, entity);
        let retrieved = retrieved.unwrap();

        assert_eq!(retrieved.capacity, 100);
        assert_eq!(retrieved.items.len(), 2);
        assert_eq!(retrieved.items.get(0).unwrap().item_id, 1);
        assert_eq!(retrieved.items.get(1).unwrap().count, 99);
    });
}

#[test]
fn test_rich_and_primitive_components_on_same_entity() {
    use cougr_core::component::Position;

    let env = Env::default();
    let addr = make_contract(&env);

    env.as_contract(&addr, || {
        let mut world = SimpleWorld::new(&env);
        let entity = world.spawn_entity();

        // primitive component via the standard ECS path
        world.set_typed(&env, entity, &Position::new(10, 20));

        // rich component on the same entity
        world.set_rich(
            &env,
            entity,
            &PlayerProfile {
                name: String::from_str(&env, "Tester"),
                scores: Vec::new(&env),
                level: Some(1),
            },
        );

        // both coexist independently
        let pos: Option<Position> = world.get_typed(&env, entity);
        assert_eq!(pos.unwrap().x, 10);

        let profile: Option<PlayerProfile> = world.get_rich(&env, entity);
        assert_eq!(profile.unwrap().name, String::from_str(&env, "Tester"));
    });
}

#[test]
fn test_different_entities_have_independent_rich_components() {
    let env = Env::default();
    let addr = make_contract(&env);

    env.as_contract(&addr, || {
        let mut world = SimpleWorld::new(&env);
        let e1 = world.spawn_entity();
        let e2 = world.spawn_entity();

        world.set_rich(
            &env,
            e1,
            &PlayerProfile {
                name: String::from_str(&env, "Alice"),
                scores: Vec::new(&env),
                level: Some(5),
            },
        );
        world.set_rich(
            &env,
            e2,
            &PlayerProfile {
                name: String::from_str(&env, "Bob"),
                scores: Vec::new(&env),
                level: Some(10),
            },
        );

        let p1: Option<PlayerProfile> = world.get_rich(&env, e1);
        let p2: Option<PlayerProfile> = world.get_rich(&env, e2);

        assert_eq!(p1.unwrap().name, String::from_str(&env, "Alice"));
        assert_eq!(p2.unwrap().name, String::from_str(&env, "Bob"));
    });
}

#[test]
fn test_option_none_stored_correctly() {
    let env = Env::default();
    let addr = make_contract(&env);

    env.as_contract(&addr, || {
        let mut world = SimpleWorld::new(&env);
        let entity = world.spawn_entity();

        world.set_rich(
            &env,
            entity,
            &PlayerProfile {
                name: String::from_str(&env, "Ghost"),
                scores: Vec::new(&env),
                level: None,
            },
        );

        let retrieved: Option<PlayerProfile> = world.get_rich(&env, entity);
        let retrieved = retrieved.unwrap();
        assert!(retrieved.level.is_none());
        assert_eq!(retrieved.name, String::from_str(&env, "Ghost"));
    });
}

#[test]
fn test_long_component_name_no_9_char_limit() {
    // impl_rich_component! uses Symbol::new (no 9-char limit).
    // "player_profile" is 14 chars - would panic in symbol_short!
    let env = Env::default();
    let sym = PlayerProfile::component_type(&env);
    assert_eq!(sym, soroban_sdk::Symbol::new(&env, "player_profile"));
}

#[test]
fn test_large_scores_vec_roundtrip() {
    let env = Env::default();
    let addr = make_contract(&env);

    env.as_contract(&addr, || {
        let mut world = SimpleWorld::new(&env);
        let entity = world.spawn_entity();

        let mut scores: Vec<u32> = Vec::new(&env);
        for i in 0..20u32 {
            scores.push_back(i * 10);
        }

        world.set_rich(
            &env,
            entity,
            &PlayerProfile {
                name: String::from_str(&env, "BigPlayer"),
                scores,
                level: Some(99),
            },
        );

        let retrieved: Option<PlayerProfile> = world.get_rich(&env, entity);
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.scores.len(), 20);
        assert_eq!(retrieved.scores.get(19), Some(190));
    });
}
