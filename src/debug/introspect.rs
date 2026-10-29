//! World and entity inspection utilities.

use crate::simple_world::{EntityId, SimpleWorld};
use alloc::vec::Vec;
use soroban_sdk::{contracttype, Env, Symbol};

/// Summary of a single entity's state.
#[contracttype]
#[derive(Clone, Debug)]
pub struct EntitySummary {
    /// The entity's ID.
    pub entity_id: u32,
    /// Total number of components (table + sparse).
    pub component_count: u32,
    /// List of component type symbols.
    pub component_types: soroban_sdk::Vec<Symbol>,
    /// Number of components in Table storage.
    pub table_count: u32,
    /// Number of components in Sparse storage.
    pub sparse_count: u32,
}

/// Summary of the entire world state.
#[contracttype]
#[derive(Clone, Debug)]
pub struct WorldSummary {
    /// Total number of entities with components.
    pub entity_count: u32,
    /// Total number of component entries across all entities.
    pub total_components: u32,
    /// Number of entries in the table storage map.
    pub table_entries: u32,
    /// Number of entries in the sparse storage map.
    pub sparse_entries: u32,
    /// Current world version.
    pub version: u64,
    /// Next entity ID that will be assigned.
    pub next_entity_id: u32,
}

/// Inspect a single entity, returning a summary of its components.
///
/// Returns `None` if the entity has no components registered.
pub fn inspect_entity(
    world: &SimpleWorld,
    entity_id: EntityId,
    _env: &Env,
) -> Option<EntitySummary> {
    let types = world.entity_components.get(entity_id)?;

    let mut table_count: u32 = 0;
    let mut sparse_count: u32 = 0;

    for i in 0..types.len() {
        if let Some(t) = types.get(i) {
            if world.components.contains_key((entity_id, t.clone())) {
                table_count += 1;
            } else if world.sparse_components.contains_key((entity_id, t.clone())) {
                sparse_count += 1;
            }
        }
    }

    Some(EntitySummary {
        entity_id,
        component_count: table_count + sparse_count,
        component_types: types,
        table_count,
        sparse_count,
    })
}

/// Inspect the entire world, returning a high-level summary.
pub fn inspect_world(world: &SimpleWorld, _env: &Env) -> WorldSummary {
    WorldSummary {
        entity_count: world.entity_components.len(),
        total_components: world.components.len() + world.sparse_components.len(),
        table_entries: world.components.len(),
        sparse_entries: world.sparse_components.len(),
        version: world.version(),
        next_entity_id: world.next_entity_id,
    }
}

/// List summaries for all entities that have components.
pub fn list_entities(world: &SimpleWorld, env: &Env) -> Vec<EntitySummary> {
    let mut summaries = Vec::new();

    for eid in world.entity_components.keys().iter() {
        if let Some(summary) = inspect_entity(world, eid, env) {
            summaries.push(summary);
        }
    }

    summaries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::ComponentStorage;
    use soroban_sdk::{symbol_short, Bytes};

    #[test]
    fn test_inspect_empty_world() {
        let env = Env::default();
        let world = SimpleWorld::new(&env);
        let summary = inspect_world(&world, &env);

        assert_eq!(summary.entity_count, 0);
        assert_eq!(summary.total_components, 0);
        assert_eq!(summary.table_entries, 0);
        assert_eq!(summary.sparse_entries, 0);
        assert_eq!(summary.version, 0);
        assert_eq!(summary.next_entity_id, 1);
    }

    #[test]
    fn test_inspect_entity_not_found() {
        let env = Env::default();
        let world = SimpleWorld::new(&env);

        assert!(inspect_entity(&world, 999, &env).is_none());
    }

    #[test]
    fn test_inspect_entity_with_components() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let e = world.spawn_entity();

        world.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[1]));
        world.add_component(e, symbol_short!("vel"), Bytes::from_array(&env, &[2]));
        world.add_component_with_storage(
            e,
            symbol_short!("tag"),
            Bytes::from_array(&env, &[3]),
            ComponentStorage::Sparse,
        );

        let summary = inspect_entity(&world, e, &env).unwrap();
        assert_eq!(summary.entity_id, e);
        assert_eq!(summary.component_count, 3);
        assert_eq!(summary.table_count, 2);
        assert_eq!(summary.sparse_count, 1);
        assert_eq!(summary.component_types.len(), 3);
    }

    #[test]
    fn test_inspect_world_with_entities() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        let e1 = world.spawn_entity();
        world.add_component(e1, symbol_short!("pos"), Bytes::from_array(&env, &[1]));
        let e2 = world.spawn_entity();
        world.add_component(e2, symbol_short!("pos"), Bytes::from_array(&env, &[2]));
        world.add_component_with_storage(
            e2,
            symbol_short!("tag"),
            Bytes::from_array(&env, &[3]),
            ComponentStorage::Sparse,
        );

        let summary = inspect_world(&world, &env);
        assert_eq!(summary.entity_count, 2);
        assert_eq!(summary.total_components, 3);
        assert_eq!(summary.table_entries, 2);
        assert_eq!(summary.sparse_entries, 1);
        assert_eq!(summary.next_entity_id, 3);
    }

    #[test]
    fn test_list_entities() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        let e1 = world.spawn_entity();
        world.add_component(e1, symbol_short!("pos"), Bytes::from_array(&env, &[1]));
        let e2 = world.spawn_entity();
        world.add_component(e2, symbol_short!("hp"), Bytes::from_array(&env, &[2]));

        let list = list_entities(&world, &env);
        assert_eq!(list.len(), 2);
    }
}
