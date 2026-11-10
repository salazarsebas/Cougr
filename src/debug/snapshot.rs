//! State snapshots and diffing.

use crate::simple_world::{EntityId, SimpleWorld};
use alloc::vec::Vec;
use soroban_sdk::{contracttype, Bytes, Env, Map, Symbol};

/// A frozen snapshot of world state for comparison.
///
/// Can optionally be stored on-chain (it's `#[contracttype]`).
#[contracttype]
#[derive(Clone, Debug)]
pub struct WorldSnapshot {
    /// World version at the time of snapshot.
    pub version: u64,
    /// Number of entities.
    pub entity_count: u32,
    /// Map from entity ID to a list of (component_type, data) pairs.
    pub entity_states: Map<u32, soroban_sdk::Vec<(Symbol, Bytes)>>,
}

/// Diff between two snapshots (runtime-only, uses `alloc::vec::Vec`).
pub struct WorldDiff {
    /// Entity IDs present in `after` but not `before`.
    pub added_entities: Vec<EntityId>,
    /// Entity IDs present in `before` but not `after`.
    pub removed_entities: Vec<EntityId>,
    /// (entity_id, component_type) pairs added between snapshots.
    pub added_components: Vec<(EntityId, Symbol)>,
    /// (entity_id, component_type) pairs removed between snapshots.
    pub removed_components: Vec<(EntityId, Symbol)>,
    /// (entity_id, component_type) pairs with changed data.
    pub modified_components: Vec<(EntityId, Symbol)>,
}

/// Take a snapshot of the current world state.
pub fn take_snapshot(world: &SimpleWorld, env: &Env) -> WorldSnapshot {
    let mut entity_states: Map<u32, soroban_sdk::Vec<(Symbol, Bytes)>> = Map::new(env);

    for eid in world.entity_components.keys().iter() {
        if let Some(types) = world.entity_components.get(eid) {
            let mut components: soroban_sdk::Vec<(Symbol, Bytes)> = soroban_sdk::Vec::new(env);
            for i in 0..types.len() {
                if let Some(t) = types.get(i) {
                    if let Some(data) = world.get_component(eid, &t) {
                        components.push_back((t, data));
                    }
                }
            }
            entity_states.set(eid, components);
        }
    }

    WorldSnapshot {
        version: world.version(),
        entity_count: world.entity_components.len(),
        entity_states,
    }
}

/// Compute the diff between two snapshots.
pub fn diff_snapshots(before: &WorldSnapshot, after: &WorldSnapshot, _env: &Env) -> WorldDiff {
    let mut added_entities = Vec::new();
    let mut removed_entities = Vec::new();
    let mut added_components = Vec::new();
    let mut removed_components = Vec::new();
    let mut modified_components = Vec::new();

    // Find added entities (in after, not in before)
    for eid in after.entity_states.keys().iter() {
        if !before.entity_states.contains_key(eid) {
            added_entities.push(eid);
        }
    }

    // Find removed entities (in before, not in after)
    for eid in before.entity_states.keys().iter() {
        if !after.entity_states.contains_key(eid) {
            removed_entities.push(eid);
        }
    }

    // Find component-level changes for entities present in both
    for eid in after.entity_states.keys().iter() {
        if let (Some(after_comps), Some(before_comps)) =
            (after.entity_states.get(eid), before.entity_states.get(eid))
        {
            // Check for added and modified components
            for i in 0..after_comps.len() {
                if let Some((after_type, after_data)) = after_comps.get(i) {
                    let mut found_in_before = false;
                    for j in 0..before_comps.len() {
                        if let Some((before_type, before_data)) = before_comps.get(j) {
                            if after_type == before_type {
                                found_in_before = true;
                                if after_data != before_data {
                                    modified_components.push((eid, after_type.clone()));
                                }
                                break;
                            }
                        }
                    }
                    if !found_in_before {
                        added_components.push((eid, after_type));
                    }
                }
            }

            // Check for removed components
            for j in 0..before_comps.len() {
                if let Some((before_type, _)) = before_comps.get(j) {
                    let mut found_in_after = false;
                    for i in 0..after_comps.len() {
                        if let Some((after_type, _)) = after_comps.get(i) {
                            if before_type == after_type {
                                found_in_after = true;
                                break;
                            }
                        }
                    }
                    if !found_in_after {
                        removed_components.push((eid, before_type));
                    }
                }
            }
        } else if after.entity_states.contains_key(eid) && !before.entity_states.contains_key(eid) {
            // All components of added entity are added components
            if let Some(comps) = after.entity_states.get(eid) {
                for i in 0..comps.len() {
                    if let Some((t, _)) = comps.get(i) {
                        added_components.push((eid, t));
                    }
                }
            }
        }
    }

    WorldDiff {
        added_entities,
        removed_entities,
        added_components,
        removed_components,
        modified_components,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::ComponentStorage;
    use soroban_sdk::{symbol_short, Bytes};

    #[test]
    fn test_snapshot_empty_world() {
        let env = Env::default();
        let world = SimpleWorld::new(&env);
        let snap = take_snapshot(&world, &env);

        assert_eq!(snap.version, 0);
        assert_eq!(snap.entity_count, 0);
        assert_eq!(snap.entity_states.len(), 0);
    }

    #[test]
    fn test_snapshot_captures_state() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        let e1 = world.spawn_entity();
        world.add_component(e1, symbol_short!("pos"), Bytes::from_array(&env, &[10]));
        world.add_component(e1, symbol_short!("vel"), Bytes::from_array(&env, &[5]));

        let snap = take_snapshot(&world, &env);
        assert_eq!(snap.version, world.version());
        assert_eq!(snap.entity_count, 1);

        let comps = snap.entity_states.get(e1).unwrap();
        assert_eq!(comps.len(), 2);
    }

    #[test]
    fn test_snapshot_includes_sparse() {
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

        let snap = take_snapshot(&world, &env);
        let comps = snap.entity_states.get(e).unwrap();
        assert_eq!(comps.len(), 2); // both table and sparse
    }

    #[test]
    fn test_diff_no_changes() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let e = world.spawn_entity();
        world.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[1]));

        let snap1 = take_snapshot(&world, &env);
        let snap2 = take_snapshot(&world, &env);
        let diff = diff_snapshots(&snap1, &snap2, &env);

        assert!(diff.added_entities.is_empty());
        assert!(diff.removed_entities.is_empty());
        assert!(diff.added_components.is_empty());
        assert!(diff.removed_components.is_empty());
        assert!(diff.modified_components.is_empty());
    }

    #[test]
    fn test_diff_added_entity() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        let snap_before = take_snapshot(&world, &env);

        let e = world.spawn_entity();
        world.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[1]));

        let snap_after = take_snapshot(&world, &env);
        let diff = diff_snapshots(&snap_before, &snap_after, &env);

        assert_eq!(diff.added_entities.len(), 1);
        assert_eq!(diff.added_entities[0], e);
        assert!(diff.removed_entities.is_empty());
    }

    #[test]
    fn test_diff_removed_entity() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        let e = world.spawn_entity();
        world.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[1]));
        let snap_before = take_snapshot(&world, &env);

        world.despawn_entity(e);
        let snap_after = take_snapshot(&world, &env);
        let diff = diff_snapshots(&snap_before, &snap_after, &env);

        assert!(diff.added_entities.is_empty());
        assert_eq!(diff.removed_entities.len(), 1);
        assert_eq!(diff.removed_entities[0], e);
    }

    #[test]
    fn test_diff_modified_component() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        let e = world.spawn_entity();
        world.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[10]));
        let snap_before = take_snapshot(&world, &env);

        world.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[20]));
        let snap_after = take_snapshot(&world, &env);
        let diff = diff_snapshots(&snap_before, &snap_after, &env);

        assert!(diff.added_entities.is_empty());
        assert!(diff.removed_entities.is_empty());
        assert_eq!(diff.modified_components.len(), 1);
        assert_eq!(diff.modified_components[0].0, e);
    }

    #[test]
    fn test_diff_added_component() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        let e = world.spawn_entity();
        world.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[1]));
        let snap_before = take_snapshot(&world, &env);

        world.add_component(e, symbol_short!("vel"), Bytes::from_array(&env, &[2]));
        let snap_after = take_snapshot(&world, &env);
        let diff = diff_snapshots(&snap_before, &snap_after, &env);

        assert_eq!(diff.added_components.len(), 1);
        assert_eq!(diff.added_components[0].0, e);
    }

    #[test]
    fn test_diff_removed_component() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        let e = world.spawn_entity();
        world.add_component(e, symbol_short!("pos"), Bytes::from_array(&env, &[1]));
        world.add_component(e, symbol_short!("vel"), Bytes::from_array(&env, &[2]));
        let snap_before = take_snapshot(&world, &env);

        world.remove_component(e, &symbol_short!("vel"));
        let snap_after = take_snapshot(&world, &env);
        let diff = diff_snapshots(&snap_before, &snap_after, &env);

        assert_eq!(diff.removed_components.len(), 1);
        assert_eq!(diff.removed_components[0].0, e);
    }
}
