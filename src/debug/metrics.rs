//! Storage metrics and statistics.

use crate::simple_world::SimpleWorld;
use alloc::vec::Vec;
use soroban_sdk::{contracttype, Env, Symbol};

/// Aggregate metrics about the world's storage usage.
#[contracttype]
#[derive(Clone, Debug)]
pub struct StorageMetrics {
    /// Total number of entries in table storage.
    pub total_table_entries: u32,
    /// Total number of entries in sparse storage.
    pub total_sparse_entries: u32,
    /// Total number of entities with at least one component.
    pub total_entities: u32,
    /// Average number of components per entity (integer-truncated).
    pub avg_components_per_entity: u32,
    /// Maximum number of components on any single entity.
    pub max_components_per_entity: u32,
    /// Number of distinct component type symbols in use.
    pub unique_component_types: u32,
}

/// Collect storage metrics from the world.
pub fn collect_metrics(world: &SimpleWorld, env: &Env) -> StorageMetrics {
    let entity_count = world.entity_components.len();
    let total_components = world.components.len() + world.sparse_components.len();

    let mut max_components: u32 = 0;
    for eid in world.entity_components.keys().iter() {
        if let Some(types) = world.entity_components.get(eid) {
            let count = types.len();
            if count > max_components {
                max_components = count;
            }
        }
    }

    let avg = total_components.checked_div(entity_count).unwrap_or(0);

    let unique_types = unique_component_types(world, env);

    StorageMetrics {
        total_table_entries: world.components.len(),
        total_sparse_entries: world.sparse_components.len(),
        total_entities: entity_count,
        avg_components_per_entity: avg,
        max_components_per_entity: max_components,
        unique_component_types: unique_types.len() as u32,
    }
}

/// Get all unique component type symbols in the world.
pub fn unique_component_types(world: &SimpleWorld, _env: &Env) -> Vec<Symbol> {
    let mut types = Vec::new();

    for eid in world.entity_components.keys().iter() {
        if let Some(entity_types) = world.entity_components.get(eid) {
            for i in 0..entity_types.len() {
                if let Some(t) = entity_types.get(i) {
                    if !types.iter().any(|existing: &Symbol| existing == &t) {
                        types.push(t);
                    }
                }
            }
        }
    }

    types
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::ComponentStorage;
    use soroban_sdk::{symbol_short, Bytes};

    #[test]
    fn test_metrics_empty_world() {
        let env = Env::default();
        let world = SimpleWorld::new(&env);
        let metrics = collect_metrics(&world, &env);

        assert_eq!(metrics.total_table_entries, 0);
        assert_eq!(metrics.total_sparse_entries, 0);
        assert_eq!(metrics.total_entities, 0);
        assert_eq!(metrics.avg_components_per_entity, 0);
        assert_eq!(metrics.max_components_per_entity, 0);
        assert_eq!(metrics.unique_component_types, 0);
    }

    #[test]
    fn test_metrics_with_entities() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        let e1 = world.spawn_entity();
        world.add_component(e1, symbol_short!("pos"), Bytes::from_array(&env, &[1]));
        world.add_component(e1, symbol_short!("vel"), Bytes::from_array(&env, &[2]));
        world.add_component(e1, symbol_short!("hp"), Bytes::from_array(&env, &[3]));

        let e2 = world.spawn_entity();
        world.add_component(e2, symbol_short!("pos"), Bytes::from_array(&env, &[4]));

        let metrics = collect_metrics(&world, &env);
        assert_eq!(metrics.total_table_entries, 4);
        assert_eq!(metrics.total_sparse_entries, 0);
        assert_eq!(metrics.total_entities, 2);
        assert_eq!(metrics.avg_components_per_entity, 2); // 4/2 = 2
        assert_eq!(metrics.max_components_per_entity, 3); // e1 has 3
        assert_eq!(metrics.unique_component_types, 3); // pos, vel, hp
    }

    #[test]
    fn test_metrics_with_sparse() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        let e = world.spawn_entity();
        world.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[1]));
        world.add_component_with_storage(
            e,
            symbol_short!("tag"),
            Bytes::from_array(&env, &[2]),
            ComponentStorage::Sparse,
        );

        let metrics = collect_metrics(&world, &env);
        assert_eq!(metrics.total_table_entries, 1);
        assert_eq!(metrics.total_sparse_entries, 1);
    }

    #[test]
    fn test_unique_component_types() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        let e1 = world.spawn_entity();
        let e2 = world.spawn_entity();
        world.add_component(e1, symbol_short!("pos"), Bytes::from_array(&env, &[1]));
        world.add_component(e2, symbol_short!("pos"), Bytes::from_array(&env, &[2])); // duplicate type
        world.add_component(e2, symbol_short!("vel"), Bytes::from_array(&env, &[3]));

        let types = unique_component_types(&world, &env);
        assert_eq!(types.len(), 2); // pos + vel
    }
}
