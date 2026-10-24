use crate::simple_world::EntityId;
use soroban_sdk::{contracttype, Bytes, Symbol};

/// Metadata stored as a single persistent entry.
///
/// The live entity index is not stored here. It lives in bounded buckets
/// (see `keys::entity_index_key`) so spawn and despawn never rewrite a list
/// proportional to the world size, and it is read back through
/// `StorageWorld::entity_ids_page`.
#[contracttype]
#[derive(Clone, Debug)]
pub struct WorldMetadata {
    /// Next entity ID to assign.
    pub next_entity_id: u32,
    /// Version counter for cache invalidation.
    pub version: u64,
    /// Total number of live entities.
    pub entity_count: u32,
}

/// Cached entity data loaded from persistent storage.
pub(super) struct LoadedEntity {
    pub(super) entity_id: EntityId,
    pub(super) component_types: soroban_sdk::Vec<Symbol>,
}

/// Cached component data loaded from persistent storage.
pub(super) struct LoadedComponent {
    pub(super) entity_id: EntityId,
    pub(super) component_type: Symbol,
    pub(super) data: Bytes,
}

pub(super) fn component_types_contains(
    component_types: &soroban_sdk::Vec<Symbol>,
    component_type: &Symbol,
) -> bool {
    for i in 0..component_types.len() {
        if let Some(existing) = component_types.get(i) {
            if &existing == component_type {
                return true;
            }
        }
    }
    false
}

pub(super) fn component_types_without(
    component_types: &soroban_sdk::Vec<Symbol>,
    component_type: &Symbol,
) -> soroban_sdk::Vec<Symbol> {
    let env = component_types.env().clone();
    let mut filtered = soroban_sdk::Vec::new(&env);
    for i in 0..component_types.len() {
        if let Some(existing) = component_types.get(i) {
            if &existing != component_type {
                filtered.push_back(existing);
            }
        }
    }
    filtered
}

pub(super) fn entity_ids_without(
    entity_ids: &soroban_sdk::Vec<EntityId>,
    entity_id: EntityId,
) -> soroban_sdk::Vec<EntityId> {
    let env = entity_ids.env().clone();
    let mut filtered = soroban_sdk::Vec::new(&env);
    for i in 0..entity_ids.len() {
        if let Some(existing) = entity_ids.get(i) {
            if existing != entity_id {
                filtered.push_back(existing);
            }
        }
    }
    filtered
}

pub(super) fn loaded_component_index(
    loaded_components: &[LoadedComponent],
    entity_id: EntityId,
    component_type: &Symbol,
) -> Option<usize> {
    loaded_components.iter().position(|component| {
        component.entity_id == entity_id && &component.component_type == component_type
    })
}

pub(super) fn loaded_entity_index(
    loaded_entities: &[LoadedEntity],
    entity_id: EntityId,
) -> Option<usize> {
    loaded_entities
        .iter()
        .position(|entity| entity.entity_id == entity_id)
}
