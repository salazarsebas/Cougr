use crate::component::ComponentStorage;
use crate::simple_world::{EntityId, SimpleWorld};
use alloc::vec::Vec;
use soroban_sdk::{Bytes, Symbol};

/// Tracks which components were added, removed, or modified within a tick.
///
/// This is a runtime-only structure (not `#[contracttype]`) that records
/// component mutations as they happen. Systems can query the tracker to
/// skip unchanged entities, improving performance.
///
/// # Example
/// ```
/// use cougr_core::runtime::ChangeTracker;
/// use soroban_sdk::symbol_short;
///
/// let entity_id = 1;
/// let mut tracker = ChangeTracker::new();
/// tracker.record_add(entity_id, symbol_short!("pos"));
///
/// if tracker.was_added(entity_id, &symbol_short!("pos")) {
///     assert_eq!(tracker.change_count(), 1);
/// }
///
/// tracker.clear();
/// tracker.advance_tick();
/// assert_eq!(tracker.tick(), 1);
/// ```
pub struct ChangeTracker {
    added: Vec<(EntityId, Symbol)>,
    removed: Vec<(EntityId, Symbol)>,
    modified: Vec<(EntityId, Symbol)>,
    tick: u64,
}

impl ChangeTracker {
    /// Create a new empty change tracker at tick 0.
    pub fn new() -> Self {
        Self {
            added: Vec::new(),
            removed: Vec::new(),
            modified: Vec::new(),
            tick: 0,
        }
    }

    /// Record that a component was added to an entity.
    pub fn record_add(&mut self, entity_id: EntityId, component_type: Symbol) {
        self.added.push((entity_id, component_type));
    }

    /// Record that a component was removed from an entity.
    pub fn record_remove(&mut self, entity_id: EntityId, component_type: Symbol) {
        self.removed.push((entity_id, component_type));
    }

    /// Record that a component was modified on an entity.
    pub fn record_modify(&mut self, entity_id: EntityId, component_type: Symbol) {
        self.modified.push((entity_id, component_type));
    }

    /// Check if a specific component was added to an entity this tick.
    pub fn was_added(&self, entity_id: EntityId, component_type: &Symbol) -> bool {
        self.added
            .iter()
            .any(|(eid, ct)| *eid == entity_id && ct == component_type)
    }

    /// Check if a specific component was removed from an entity this tick.
    pub fn was_removed(&self, entity_id: EntityId, component_type: &Symbol) -> bool {
        self.removed
            .iter()
            .any(|(eid, ct)| *eid == entity_id && ct == component_type)
    }

    /// Check if a specific component was modified on an entity this tick.
    pub fn was_modified(&self, entity_id: EntityId, component_type: &Symbol) -> bool {
        self.modified
            .iter()
            .any(|(eid, ct)| *eid == entity_id && ct == component_type)
    }

    /// Get all entity IDs that had the given component added this tick.
    pub fn added_entities_with(&self, component_type: &Symbol) -> Vec<EntityId> {
        self.added
            .iter()
            .filter(|(_, ct)| ct == component_type)
            .map(|(eid, _)| *eid)
            .collect()
    }

    /// Get all entity IDs that had the given component modified this tick.
    pub fn modified_entities_with(&self, component_type: &Symbol) -> Vec<EntityId> {
        self.modified
            .iter()
            .filter(|(_, ct)| ct == component_type)
            .map(|(eid, _)| *eid)
            .collect()
    }

    /// Get all entity IDs that had the given component removed this tick.
    pub fn removed_entities_with(&self, component_type: &Symbol) -> Vec<EntityId> {
        self.removed
            .iter()
            .filter(|(_, ct)| ct == component_type)
            .map(|(eid, _)| *eid)
            .collect()
    }

    /// Clear all recorded changes. Call this between ticks.
    pub fn clear(&mut self) {
        self.added.clear();
        self.removed.clear();
        self.modified.clear();
    }

    /// Returns the current tick number.
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Advance the tick counter by one.
    pub fn advance_tick(&mut self) {
        self.tick += 1;
    }

    /// Returns the total number of recorded changes.
    pub fn change_count(&self) -> usize {
        self.added.len() + self.removed.len() + self.modified.len()
    }
}

impl Default for ChangeTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// A wrapper around `SimpleWorld` that automatically records component changes.
///
/// Similar to `HookedWorld`, but instead of firing callbacks, it records changes
/// in a `ChangeTracker` for later querying by systems.
///
/// # Example
/// ```
/// use cougr_core::{SimpleWorld, runtime::TrackedWorld};
/// use soroban_sdk::{symbol_short, Bytes, Env};
///
/// let env = Env::default();
/// let world = SimpleWorld::new(&env);
/// let mut tracked = TrackedWorld::new(world);
///
/// let e1 = tracked.spawn_entity();
/// tracked.add_component(e1, symbol_short!("pos"), Bytes::new(&env));
///
/// assert!(tracked.tracker().was_added(e1, &symbol_short!("pos")));
/// ```
pub struct TrackedWorld {
    world: SimpleWorld,
    tracker: ChangeTracker,
}

impl TrackedWorld {
    /// Wrap a `SimpleWorld` with a new change tracker.
    pub fn new(world: SimpleWorld) -> Self {
        Self {
            world,
            tracker: ChangeTracker::new(),
        }
    }

    /// Access the underlying `SimpleWorld`.
    pub fn world(&self) -> &SimpleWorld {
        &self.world
    }

    /// Mutably access the underlying `SimpleWorld`.
    pub fn world_mut(&mut self) -> &mut SimpleWorld {
        &mut self.world
    }

    /// Access the change tracker.
    pub fn tracker(&self) -> &ChangeTracker {
        &self.tracker
    }

    /// Mutably access the change tracker (e.g. to clear it).
    pub fn tracker_mut(&mut self) -> &mut ChangeTracker {
        &mut self.tracker
    }

    /// Consume the wrapper and return the inner `SimpleWorld`.
    pub fn into_inner(self) -> SimpleWorld {
        self.world
    }

    /// Spawn a new entity (delegates to `SimpleWorld`).
    pub fn spawn_entity(&mut self) -> EntityId {
        self.world.spawn_entity()
    }

    /// Add a component, recording the change.
    pub fn add_component(&mut self, entity_id: EntityId, component_type: Symbol, data: Bytes) {
        let existed = self.world.has_component(entity_id, &component_type);
        self.world
            .add_component(entity_id, component_type.clone(), data);
        if existed {
            self.tracker.record_modify(entity_id, component_type);
        } else {
            self.tracker.record_add(entity_id, component_type);
        }
    }

    /// Add a component with explicit storage kind, recording the change.
    pub fn add_component_with_storage(
        &mut self,
        entity_id: EntityId,
        component_type: Symbol,
        data: Bytes,
        storage: ComponentStorage,
    ) {
        let existed = self.world.has_component(entity_id, &component_type);
        self.world
            .add_component_with_storage(entity_id, component_type.clone(), data, storage);
        if existed {
            self.tracker.record_modify(entity_id, component_type);
        } else {
            self.tracker.record_add(entity_id, component_type);
        }
    }

    /// Remove a component, recording the change.
    pub fn remove_component(&mut self, entity_id: EntityId, component_type: &Symbol) -> bool {
        let removed = self.world.remove_component(entity_id, component_type);
        if removed {
            self.tracker
                .record_remove(entity_id, component_type.clone());
        }
        removed
    }

    /// Get a component (delegates to `SimpleWorld`).
    pub fn get_component(&self, entity_id: EntityId, component_type: &Symbol) -> Option<Bytes> {
        self.world.get_component(entity_id, component_type)
    }

    /// Check if an entity has a component (delegates to `SimpleWorld`).
    pub fn has_component(&self, entity_id: EntityId, component_type: &Symbol) -> bool {
        self.world.has_component(entity_id, component_type)
    }

    /// Despawn an entity, recording removals for each component.
    pub fn despawn_entity(&mut self, entity_id: EntityId) {
        if let Some(types) = self.world.entity_components.get(entity_id) {
            for i in 0..types.len() {
                if let Some(t) = types.get(i) {
                    self.tracker.record_remove(entity_id, t);
                }
            }
        }
        self.world.despawn_entity(entity_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, Env};

    #[test]
    fn test_tracker_new() {
        let tracker = ChangeTracker::new();
        assert_eq!(tracker.tick(), 0);
        assert_eq!(tracker.change_count(), 0);
    }

    #[test]
    fn test_record_and_query_add() {
        let _env = Env::default();
        let mut tracker = ChangeTracker::new();
        tracker.record_add(1, symbol_short!("pos"));

        assert!(tracker.was_added(1, &symbol_short!("pos")));
        assert!(!tracker.was_added(1, &symbol_short!("vel")));
        assert!(!tracker.was_added(2, &symbol_short!("pos")));
    }

    #[test]
    fn test_record_and_query_remove() {
        let _env = Env::default();
        let mut tracker = ChangeTracker::new();
        tracker.record_remove(1, symbol_short!("pos"));

        assert!(tracker.was_removed(1, &symbol_short!("pos")));
        assert!(!tracker.was_removed(1, &symbol_short!("vel")));
    }

    #[test]
    fn test_record_and_query_modify() {
        let _env = Env::default();
        let mut tracker = ChangeTracker::new();
        tracker.record_modify(1, symbol_short!("pos"));

        assert!(tracker.was_modified(1, &symbol_short!("pos")));
        assert!(!tracker.was_modified(2, &symbol_short!("pos")));
    }

    #[test]
    fn test_entities_with_queries() {
        let _env = Env::default();
        let mut tracker = ChangeTracker::new();
        tracker.record_add(1, symbol_short!("pos"));
        tracker.record_add(2, symbol_short!("pos"));
        tracker.record_add(3, symbol_short!("vel"));

        let added = tracker.added_entities_with(&symbol_short!("pos"));
        assert_eq!(added.len(), 2);
        assert!(added.contains(&1));
        assert!(added.contains(&2));
    }

    #[test]
    fn test_clear_resets() {
        let _env = Env::default();
        let mut tracker = ChangeTracker::new();
        tracker.record_add(1, symbol_short!("pos"));
        tracker.record_remove(2, symbol_short!("vel"));
        tracker.record_modify(3, symbol_short!("hp"));
        assert_eq!(tracker.change_count(), 3);

        tracker.clear();
        assert_eq!(tracker.change_count(), 0);
        assert!(!tracker.was_added(1, &symbol_short!("pos")));
    }

    #[test]
    fn test_advance_tick() {
        let mut tracker = ChangeTracker::new();
        assert_eq!(tracker.tick(), 0);
        tracker.advance_tick();
        assert_eq!(tracker.tick(), 1);
        tracker.advance_tick();
        assert_eq!(tracker.tick(), 2);
    }

    #[test]
    fn test_tracked_world_add() {
        let env = Env::default();
        let world = SimpleWorld::new(&env);
        let mut tracked = TrackedWorld::new(world);

        let e1 = tracked.spawn_entity();
        let data = Bytes::from_array(&env, &[1, 2, 3]);
        tracked.add_component(e1, symbol_short!("pos"), data.clone());

        assert!(tracked.has_component(e1, &symbol_short!("pos")));
        assert!(tracked.tracker().was_added(e1, &symbol_short!("pos")));
        assert!(!tracked.tracker().was_modified(e1, &symbol_short!("pos")));
    }

    #[test]
    fn test_tracked_world_modify() {
        let env = Env::default();
        let world = SimpleWorld::new(&env);
        let mut tracked = TrackedWorld::new(world);

        let e1 = tracked.spawn_entity();
        let data1 = Bytes::from_array(&env, &[1]);
        tracked.add_component(e1, symbol_short!("pos"), data1);

        // Overwrite existing component → should record modify
        let data2 = Bytes::from_array(&env, &[2]);
        tracked.add_component(e1, symbol_short!("pos"), data2.clone());

        assert!(tracked.tracker().was_added(e1, &symbol_short!("pos")));
        assert!(tracked.tracker().was_modified(e1, &symbol_short!("pos")));
        assert_eq!(
            tracked.get_component(e1, &symbol_short!("pos")),
            Some(data2)
        );
    }

    #[test]
    fn test_tracked_world_remove() {
        let env = Env::default();
        let world = SimpleWorld::new(&env);
        let mut tracked = TrackedWorld::new(world);

        let e1 = tracked.spawn_entity();
        let data = Bytes::from_array(&env, &[1]);
        tracked.add_component(e1, symbol_short!("pos"), data);

        assert!(tracked.remove_component(e1, &symbol_short!("pos")));
        assert!(tracked.tracker().was_removed(e1, &symbol_short!("pos")));
        assert!(!tracked.has_component(e1, &symbol_short!("pos")));
    }

    #[test]
    fn test_tracked_world_despawn_records_removals() {
        let env = Env::default();
        let world = SimpleWorld::new(&env);
        let mut tracked = TrackedWorld::new(world);

        let e1 = tracked.spawn_entity();
        let data = Bytes::from_array(&env, &[1]);
        tracked.add_component(e1, symbol_short!("a"), data.clone());
        tracked.add_component(e1, symbol_short!("b"), data);

        tracked.tracker_mut().clear(); // clear add records
        tracked.despawn_entity(e1);

        assert!(tracked.tracker().was_removed(e1, &symbol_short!("a")));
        assert!(tracked.tracker().was_removed(e1, &symbol_short!("b")));
    }

    #[test]
    fn test_tracked_world_into_inner() {
        let env = Env::default();
        let world = SimpleWorld::new(&env);
        let mut tracked = TrackedWorld::new(world);

        let e1 = tracked.spawn_entity();
        let data = Bytes::from_array(&env, &[1]);
        tracked.add_component(e1, symbol_short!("test"), data);

        let inner = tracked.into_inner();
        assert!(inner.has_component(e1, &symbol_short!("test")));
    }
}
