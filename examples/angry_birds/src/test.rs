use crate::{BirdType, Material};
use cougr_core::component::ComponentTrait;
use soroban_sdk::Env;

#[test]
fn test_material_damage_resistance() {
    let env = Env::default();

    // Test material serialization/deserialization
    let wood_material = crate::MaterialComponent {
        kind: Material::Wood,
    };
    let glass_material = crate::MaterialComponent {
        kind: Material::Glass,
    };
    let stone_material = crate::MaterialComponent {
        kind: Material::Stone,
    };

    // Serialize and deserialize wood
    let wood_bytes = wood_material.serialize(&env);
    let wood_deserialized = crate::MaterialComponent::deserialize(&env, &wood_bytes).unwrap();
    assert_eq!(wood_material.kind, wood_deserialized.kind);

    // Serialize and deserialize glass
    let glass_bytes = glass_material.serialize(&env);
    let glass_deserialized = crate::MaterialComponent::deserialize(&env, &glass_bytes).unwrap();
    assert_eq!(glass_material.kind, glass_deserialized.kind);

    // Serialize and deserialize stone
    let stone_bytes = stone_material.serialize(&env);
    let stone_deserialized = crate::MaterialComponent::deserialize(&env, &stone_bytes).unwrap();
    assert_eq!(stone_material.kind, stone_deserialized.kind);
}

#[test]
fn test_projectile_component() {
    let env = Env::default();

    // Test projectile serialization/deserialization
    let projectile = crate::ProjectileComponent {
        bird_type: BirdType::Yellow,
        angle: 45000,  // 45 degrees scaled by 1000
        power: 750000, // 0.75 scaled by 1000
        active: true,
    };

    // Serialize and deserialize
    let bytes = projectile.serialize(&env);
    let deserialized = crate::ProjectileComponent::deserialize(&env, &bytes).unwrap();

    assert_eq!(projectile.bird_type, deserialized.bird_type);
    assert_eq!(projectile.angle, deserialized.angle);
    assert_eq!(projectile.power, deserialized.power);
    assert_eq!(projectile.active, deserialized.active);
}

#[test]
fn test_health_component() {
    let env = Env::default();

    // Test health serialization/deserialization
    let health = crate::HealthComponent {
        hp: 75,
        max_hp: 100,
    };

    // Serialize and deserialize
    let bytes = health.serialize(&env);
    let deserialized = crate::HealthComponent::deserialize(&env, &bytes).unwrap();

    assert_eq!(health.hp, deserialized.hp);
    assert_eq!(health.max_hp, deserialized.max_hp);
}

#[test]
fn test_position_component() {
    let env = Env::default();

    // Test position serialization/deserialization
    let position = crate::PositionComponent {
        x: 500000, // 0.5 scaled by 1000
        y: 750000, // 0.75 scaled by 1000
    };

    // Serialize and deserialize
    let bytes = position.serialize(&env);
    let deserialized = crate::PositionComponent::deserialize(&env, &bytes).unwrap();

    assert_eq!(position.x, deserialized.x);
    assert_eq!(position.y, deserialized.y);
}
