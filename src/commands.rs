use crate::component::ComponentStorage;
use crate::simple_world::{EntityId, SimpleWorld};
use alloc::vec::Vec;
use soroban_sdk::{Bytes, Symbol};

/// The kind of structural change to apply.
enum CommandKind {
    Spawn,
    Despawn,
    AddComponent,
    RemoveComponent,
}

/// A single queued structural change.
struct CommandEntry {
    kind: CommandKind,
    entity_id: Option<EntityId>,
    component_type: Option<Symbol>,
    data: Option<Bytes>,
    storage: ComponentStorage,
}

/// A deferred command queue for safe structural changes during system iteration.
///
/// Instead of mutating the world directly while iterating entities,
/// systems can queue commands and apply them after iteration completes.
///
/// # Example
/// ```
/// use cougr_core::commands::CommandQueue;
/// use cougr_core::simple_world::SimpleWorld;
/// use soroban_sdk::{symbol_short, Bytes, Env};
///
/// let env = Env::default();
/// let mut world = SimpleWorld::new(&env);
/// let mut commands = CommandQueue::new();
///
/// commands.spawn();
/// let spawned = commands.apply(&mut world);
/// commands = CommandQueue::new();
/// commands.add_component(spawned[0], symbol_short!("bullet"), Bytes::new(&env));
/// commands.apply(&mut world);
///
/// assert!(world.has_component(spawned[0], &symbol_short!("bullet")));
/// ```
pub struct CommandQueue {
    commands: Vec<CommandEntry>,
}

impl CommandQueue {
    /// Create an empty command queue.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Queue a new entity spawn. The entity ID is assigned when `apply()` is called.
    pub fn spawn(&mut self) {
        self.commands.push(CommandEntry {
            kind: CommandKind::Spawn,
            entity_id: None,
            component_type: None,
            data: None,
            storage: ComponentStorage::Table,
        });
    }

    /// Queue an entity despawn.
    pub fn despawn(&mut self, entity_id: EntityId) {
        self.commands.push(CommandEntry {
            kind: CommandKind::Despawn,
            entity_id: Some(entity_id),
            component_type: None,
            data: None,
            storage: ComponentStorage::Table,
        });
    }

    /// Queue adding a component (Table storage) to an entity.
    pub fn add_component(&mut self, entity_id: EntityId, component_type: Symbol, data: Bytes) {
        self.commands.push(CommandEntry {
            kind: CommandKind::AddComponent,
            entity_id: Some(entity_id),
            component_type: Some(component_type),
            data: Some(data),
            storage: ComponentStorage::Table,
        });
    }

    /// Queue adding a component with Sparse storage to an entity.
    pub fn add_sparse_component(
        &mut self,
        entity_id: EntityId,
        component_type: Symbol,
        data: Bytes,
    ) {
        self.commands.push(CommandEntry {
            kind: CommandKind::AddComponent,
            entity_id: Some(entity_id),
            component_type: Some(component_type),
            data: Some(data),
            storage: ComponentStorage::Sparse,
        });
    }

    /// Queue removing a component from an entity.
    pub fn remove_component(&mut self, entity_id: EntityId, component_type: Symbol) {
        self.commands.push(CommandEntry {
            kind: CommandKind::RemoveComponent,
            entity_id: Some(entity_id),
            component_type: Some(component_type),
            data: None,
            storage: ComponentStorage::Table,
        });
    }

    /// Apply all queued commands to the world in order.
    ///
    /// Returns the IDs of any spawned entities (in spawn order).
    /// Consumes the queue.
    pub fn apply(self, world: &mut SimpleWorld) -> Vec<EntityId> {
        let mut spawned_ids = Vec::new();

        for entry in self.commands {
            match entry.kind {
                CommandKind::Spawn => {
                    let id = world.spawn_entity();
                    spawned_ids.push(id);
                }
                CommandKind::Despawn => {
                    if let Some(entity_id) = entry.entity_id {
                        world.despawn_entity(entity_id);
                    }
                }
                CommandKind::AddComponent => {
                    if let (Some(entity_id), Some(component_type), Some(data)) =
                        (entry.entity_id, entry.component_type, entry.data)
                    {
                        world.add_component_with_storage(
                            entity_id,
                            component_type,
                            data,
                            entry.storage,
                        );
                    }
                }
                CommandKind::RemoveComponent => {
                    if let (Some(entity_id), Some(component_type)) =
                        (entry.entity_id, entry.component_type)
                    {
                        world.remove_component(entity_id, &component_type);
                    }
                }
            }
        }

        spawned_ids
    }

    /// Returns whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Returns the number of queued commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }
}

impl Default for CommandQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, Env};

    #[test]
    fn test_empty_queue() {
        let queue = CommandQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);

        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let spawned = queue.apply(&mut world);
        assert!(spawned.is_empty());
    }

    #[test]
    fn test_spawn_via_queue() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let mut queue = CommandQueue::new();

        queue.spawn();
        queue.spawn();
        assert_eq!(queue.len(), 2);

        let spawned = queue.apply(&mut world);
        assert_eq!(spawned.len(), 2);
        assert_eq!(spawned[0], 1);
        assert_eq!(spawned[1], 2);
    }

    #[test]
    fn test_despawn_via_queue() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let e1 = world.spawn_entity();
        let data = Bytes::from_array(&env, &[1]);
        world.add_component(e1, symbol_short!("pos"), data);

        let mut queue = CommandQueue::new();
        queue.despawn(e1);
        queue.apply(&mut world);

        assert!(!world.has_component(e1, &symbol_short!("pos")));
    }

    #[test]
    fn test_add_component_via_queue() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let e1 = world.spawn_entity();

        let mut queue = CommandQueue::new();
        let data = Bytes::from_array(&env, &[1, 2, 3]);
        queue.add_component(e1, symbol_short!("pos"), data.clone());
        queue.apply(&mut world);

        assert!(world.has_component(e1, &symbol_short!("pos")));
        assert_eq!(world.get_component(e1, &symbol_short!("pos")), Some(data));
    }

    #[test]
    fn test_add_sparse_component_via_queue() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let e1 = world.spawn_entity();

        let mut queue = CommandQueue::new();
        let data = Bytes::from_array(&env, &[0xAA]);
        queue.add_sparse_component(e1, symbol_short!("tag"), data.clone());
        queue.apply(&mut world);

        assert!(world.has_component(e1, &symbol_short!("tag")));
        assert_eq!(world.table_component_count(&symbol_short!("tag")), 0);
        assert_eq!(world.component_count(&symbol_short!("tag")), 1);
    }

    #[test]
    fn test_remove_component_via_queue() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let e1 = world.spawn_entity();
        let data = Bytes::from_array(&env, &[1]);
        world.add_component(e1, symbol_short!("pos"), data);

        let mut queue = CommandQueue::new();
        queue.remove_component(e1, symbol_short!("pos"));
        queue.apply(&mut world);

        assert!(!world.has_component(e1, &symbol_short!("pos")));
    }

    #[test]
    fn test_mixed_operations() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        // Pre-existing entity
        let e1 = world.spawn_entity();
        let data = Bytes::from_array(&env, &[1]);
        world.add_component(e1, symbol_short!("old"), data);

        let mut queue = CommandQueue::new();
        // Queue: spawn new entity, add component to e1, remove "old" from e1
        queue.spawn();
        let new_data = Bytes::from_array(&env, &[2, 3]);
        queue.add_component(e1, symbol_short!("new"), new_data.clone());
        queue.remove_component(e1, symbol_short!("old"));

        let spawned = queue.apply(&mut world);
        assert_eq!(spawned.len(), 1);
        assert_eq!(spawned[0], 2); // second entity
        assert!(world.has_component(e1, &symbol_short!("new")));
        assert!(!world.has_component(e1, &symbol_short!("old")));
    }

    #[test]
    fn test_queue_len_tracking() {
        let env = Env::default();
        let mut queue = CommandQueue::new();
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());

        queue.spawn();
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());

        let data = Bytes::from_array(&env, &[1]);
        queue.add_component(1, symbol_short!("test"), data);
        assert_eq!(queue.len(), 2);

        queue.remove_component(1, symbol_short!("test"));
        assert_eq!(queue.len(), 3);

        queue.despawn(1);
        assert_eq!(queue.len(), 4);
    }
}
