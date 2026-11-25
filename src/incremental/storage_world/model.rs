use crate::component::ComponentStorage;
use crate::simple_world::EntityId;
use soroban_sdk::{contracttype, Bytes, Symbol};

/// Metadata stored as a single persistent entry per world namespace.
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

/// Stored reference to a component on an entity, preserving its storage kind.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredComponentRef {
    pub component_type: Symbol,
    pub storage: ComponentStorage,
}

/// Policy controlling persistent storage TTL extension on reads and writes.
///
/// Use [`TtlPolicy::disabled()`] (or set `extend_to` to `0`) to opt out of
/// automatic TTL extension.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TtlPolicy {
    /// Minimum remaining ledgers before extending TTL.
    pub threshold: u32,
    /// Target TTL in ledgers when extending.
    pub extend_to: u32,
}

impl TtlPolicy {
    /// Default minimum remaining ledgers before extending TTL.
    pub const DEFAULT_THRESHOLD: u32 = 1_000;
    /// Default target TTL in ledgers when extending.
    pub const DEFAULT_EXTEND_TO: u32 = 10_000;

    /// Create a custom TTL extension policy.
    pub const fn new(threshold: u32, extend_to: u32) -> Self {
        Self {
            threshold,
            extend_to,
        }
    }

    /// Opt out of automatic TTL extension.
    pub const fn disabled() -> Self {
        Self {
            threshold: 0,
            extend_to: 0,
        }
    }

    /// Returns `true` if automatic TTL extension is enabled.
    pub const fn is_enabled(&self) -> bool {
        self.extend_to > 0
    }
}

impl Default for TtlPolicy {
    fn default() -> Self {
        Self {
            threshold: Self::DEFAULT_THRESHOLD,
            extend_to: Self::DEFAULT_EXTEND_TO,
        }
    }
}

/// Cached entity data loaded from persistent storage.
pub(super) struct LoadedEntity {
    pub(super) entity_id: EntityId,
    pub(super) components: soroban_sdk::Vec<StoredComponentRef>,
}

/// Cached component data loaded from persistent storage.
pub(super) struct LoadedComponent {
    pub(super) entity_id: EntityId,
    pub(super) component_type: Symbol,
    pub(super) data: Bytes,
    pub(super) storage: ComponentStorage,
}

pub(super) fn component_refs_contains(
    components: &soroban_sdk::Vec<StoredComponentRef>,
    component_type: &Symbol,
) -> bool {
    for i in 0..components.len() {
        if let Some(existing) = components.get(i) {
            if &existing.component_type == component_type {
                return true;
            }
        }
    }
    false
}

pub(super) fn component_refs_upsert(
    components: &mut soroban_sdk::Vec<StoredComponentRef>,
    component_type: &Symbol,
    storage: ComponentStorage,
) -> bool {
    for i in 0..components.len() {
        if let Some(existing) = components.get(i) {
            if &existing.component_type == component_type {
                if existing.storage != storage {
                    components.set(
                        i,
                        StoredComponentRef {
                            component_type: component_type.clone(),
                            storage,
                        },
                    );
                    return true;
                }
                return false;
            }
        }
    }
    components.push_back(StoredComponentRef {
        component_type: component_type.clone(),
        storage,
    });
    true
}

pub(super) fn component_refs_without(
    components: &soroban_sdk::Vec<StoredComponentRef>,
    component_type: &Symbol,
) -> soroban_sdk::Vec<StoredComponentRef> {
    let env = components.env().clone();
    let mut filtered = soroban_sdk::Vec::new(&env);
    for i in 0..components.len() {
        if let Some(existing) = components.get(i) {
            if &existing.component_type != component_type {
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
