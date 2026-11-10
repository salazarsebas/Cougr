//! Tracks which entities and components need flushing to persistent storage.
//!
//! Only dirty (modified) entries are written on `flush()`, avoiding
//! redundant storage writes and reducing gas costs.

use crate::simple_world::EntityId;
use alloc::vec::Vec;
use soroban_sdk::Symbol;

/// Tracks which entities/components have been modified since the last flush.
///
/// Follows the same pattern as `ChangeTracker` from `src/change_tracker.rs`.
pub struct DirtyTracker {
    dirty_entities: Vec<EntityId>,
    dirty_components: Vec<(EntityId, Symbol)>,
    despawned: Vec<EntityId>,
    new_entities: Vec<EntityId>,
    meta_dirty: bool,
}

impl DirtyTracker {
    /// Create a new empty tracker.
    pub fn new() -> Self {
        Self {
            dirty_entities: Vec::new(),
            dirty_components: Vec::new(),
            despawned: Vec::new(),
            new_entities: Vec::new(),
            meta_dirty: false,
        }
    }

    /// Mark an entity's component list as needing a flush.
    pub fn mark_entity_dirty(&mut self, entity_id: EntityId) {
        if !self.dirty_entities.contains(&entity_id) {
            self.dirty_entities.push(entity_id);
        }
    }

    /// Mark a specific component on an entity as needing a flush.
    pub fn mark_component_dirty(&mut self, entity_id: EntityId, component_type: Symbol) {
        let entry = (entity_id, component_type);
        if !self.dirty_components.contains(&entry) {
            self.dirty_components.push(entry);
        }
    }

    /// Mark an entity as despawned (needs all storage entries removed).
    pub fn mark_despawned(&mut self, entity_id: EntityId) {
        if !self.despawned.contains(&entity_id) {
            self.despawned.push(entity_id);
        }
    }

    /// Mark a newly spawned entity.
    pub fn mark_new_entity(&mut self, entity_id: EntityId) {
        if !self.new_entities.contains(&entity_id) {
            self.new_entities.push(entity_id);
        }
        self.meta_dirty = true;
    }

    /// Mark metadata as needing a flush.
    pub fn mark_meta_dirty(&mut self) {
        self.meta_dirty = true;
    }

    /// Returns true if any data needs flushing.
    pub fn is_dirty(&self) -> bool {
        self.meta_dirty
            || !self.dirty_entities.is_empty()
            || !self.dirty_components.is_empty()
            || !self.despawned.is_empty()
            || !self.new_entities.is_empty()
    }

    /// Get entities whose component lists need writing.
    pub fn dirty_entities(&self) -> &[EntityId] {
        &self.dirty_entities
    }

    /// Get specific components that need writing.
    pub fn dirty_components(&self) -> &[(EntityId, Symbol)] {
        &self.dirty_components
    }

    /// Get despawned entities.
    pub fn despawned(&self) -> &[EntityId] {
        &self.despawned
    }

    /// Get despawned entities (alias used by storage_world flush).
    pub fn despawned_entities(&self) -> &[EntityId] {
        &self.despawned
    }

    /// Get newly spawned entities.
    #[cfg(test)]
    pub fn new_entities(&self) -> &[EntityId] {
        &self.new_entities
    }

    /// Whether metadata needs flushing.
    pub fn is_meta_dirty(&self) -> bool {
        self.meta_dirty
    }

    /// Reset all tracking state after a flush.
    pub fn clear(&mut self) {
        self.dirty_entities.clear();
        self.dirty_components.clear();
        self.despawned.clear();
        self.new_entities.clear();
        self.meta_dirty = false;
    }
}

impl Default for DirtyTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, Env};

    #[test]
    fn test_new_tracker_not_dirty() {
        let tracker = DirtyTracker::new();
        assert!(!tracker.is_dirty());
        assert!(tracker.dirty_entities().is_empty());
        assert!(tracker.dirty_components().is_empty());
        assert!(tracker.despawned().is_empty());
    }

    #[test]
    fn test_mark_entity_dirty() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_entity_dirty(1);
        assert!(tracker.is_dirty());
        assert_eq!(tracker.dirty_entities(), &[1]);
        // Duplicate should be ignored
        tracker.mark_entity_dirty(1);
        assert_eq!(tracker.dirty_entities().len(), 1);
    }

    #[test]
    fn test_mark_component_dirty() {
        let _env = Env::default();
        let mut tracker = DirtyTracker::new();
        tracker.mark_component_dirty(1, symbol_short!("pos"));
        assert!(tracker.is_dirty());
        assert_eq!(tracker.dirty_components().len(), 1);
    }

    #[test]
    fn test_mark_despawned() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_despawned(5);
        assert!(tracker.is_dirty());
        assert_eq!(tracker.despawned(), &[5]);
    }

    #[test]
    fn test_mark_new_entity() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_new_entity(10);
        assert!(tracker.is_dirty());
        assert!(tracker.is_meta_dirty());
        assert_eq!(tracker.new_entities(), &[10]);
    }

    #[test]
    fn test_clear() {
        let _env = Env::default();
        let mut tracker = DirtyTracker::new();
        tracker.mark_entity_dirty(1);
        tracker.mark_component_dirty(1, symbol_short!("pos"));
        tracker.mark_despawned(2);
        tracker.mark_new_entity(3);
        assert!(tracker.is_dirty());

        tracker.clear();
        assert!(!tracker.is_dirty());
        assert!(tracker.dirty_entities().is_empty());
        assert!(tracker.dirty_components().is_empty());
        assert!(tracker.despawned().is_empty());
        assert!(tracker.new_entities().is_empty());
        assert!(!tracker.is_meta_dirty());
    }
}
