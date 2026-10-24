mod model;
#[cfg(test)]
mod tests;

pub use model::WorldMetadata;

use self::model::{
    component_types_contains, component_types_without, entity_ids_without, loaded_component_index,
    loaded_entity_index, LoadedComponent, LoadedEntity,
};
use super::dirty_tracker::DirtyTracker;
use super::keys;
use crate::error::CougrError;
use crate::simple_world::{EntityId, SimpleWorld};
use alloc::vec::Vec;
use soroban_sdk::{Bytes, Env, Symbol};

/// World that reads/writes individual entities from Soroban persistent storage.
///
/// Only dirty state is written back on `flush()`. Entities must be explicitly
/// loaded before they can be queried or modified.
pub struct StorageWorld {
    metadata: WorldMetadata,
    loaded_entities: Vec<LoadedEntity>,
    loaded_components: Vec<LoadedComponent>,
    dirty: DirtyTracker,
}

impl StorageWorld {
    /// Load world metadata from persistent storage.
    ///
    /// If no metadata exists, creates a fresh world.
    pub fn load_metadata(env: &Env) -> Self {
        let key = keys::meta_key(env);
        let metadata: WorldMetadata =
            env.storage()
                .persistent()
                .get(&key)
                .unwrap_or(WorldMetadata {
                    next_entity_id: 1,
                    version: 0,
                    entity_count: 0,
                    entity_ids: soroban_sdk::Vec::new(env),
                });

        Self {
            metadata,
            loaded_entities: Vec::new(),
            loaded_components: Vec::new(),
            dirty: DirtyTracker::new(),
        }
    }

    /// Load a single entity's component list and component data from storage.
    pub fn load_entity(&mut self, env: &Env, entity_id: EntityId) -> Result<(), CougrError> {
        if loaded_entity_index(&self.loaded_entities, entity_id).is_some() {
            return Ok(());
        }

        let key = keys::entity_key(env, entity_id);
        let component_types: soroban_sdk::Vec<Symbol> = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(CougrError::EntityNotFound)?;

        for i in 0..component_types.len() {
            if let Some(component_type) = component_types.get(i) {
                let ckey = keys::component_key(env, entity_id, &component_type);
                if let Some(data) = env.storage().persistent().get::<_, Bytes>(&ckey) {
                    self.loaded_components.push(LoadedComponent {
                        entity_id,
                        component_type,
                        data,
                    });
                }
            }
        }

        self.loaded_entities.push(LoadedEntity {
            entity_id,
            component_types,
        });

        Ok(())
    }

    /// Load multiple entities from storage.
    pub fn load_entities(&mut self, env: &Env, entity_ids: &[EntityId]) -> Result<(), CougrError> {
        for &entity_id in entity_ids {
            self.load_entity(env, entity_id)?;
        }
        Ok(())
    }

    /// Spawn a new entity (assigns ID but doesn't write to storage until flush).
    pub fn spawn_entity(&mut self, env: &Env) -> EntityId {
        let entity_id = self.metadata.next_entity_id;
        self.metadata.next_entity_id += 1;
        self.metadata.entity_count += 1;
        self.metadata.entity_ids.push_back(entity_id);

        self.loaded_entities.push(LoadedEntity {
            entity_id,
            component_types: soroban_sdk::Vec::new(env),
        });

        self.dirty.mark_new_entity(entity_id);
        self.dirty.mark_entity_dirty(entity_id);
        entity_id
    }

    /// Add a component to an entity.
    pub fn add_component(
        &mut self,
        env: &Env,
        entity_id: EntityId,
        component_type: Symbol,
        data: Bytes,
    ) {
        self.ensure_loaded_entity(env, entity_id, &component_type);
        self.upsert_loaded_component(entity_id, &component_type, data);

        self.metadata.version += 1;
        self.dirty.mark_entity_dirty(entity_id);
        self.dirty.mark_component_dirty(entity_id, component_type);
        self.dirty.mark_meta_dirty();
    }

    /// Get a component's data from loaded state.
    pub fn get_component(&self, entity_id: EntityId, component_type: &Symbol) -> Option<Bytes> {
        loaded_component_index(&self.loaded_components, entity_id, component_type)
            .map(|index| self.loaded_components[index].data.clone())
    }

    /// Check if a loaded entity has a component.
    pub fn has_component(&self, entity_id: EntityId, component_type: &Symbol) -> bool {
        loaded_component_index(&self.loaded_components, entity_id, component_type).is_some()
    }

    /// Remove a component from an entity.
    pub fn remove_component(&mut self, entity_id: EntityId, component_type: &Symbol) -> bool {
        let Some(component_index) =
            loaded_component_index(&self.loaded_components, entity_id, component_type)
        else {
            return false;
        };

        self.loaded_components.remove(component_index);
        if let Some(entity_index) = loaded_entity_index(&self.loaded_entities, entity_id) {
            let updated = component_types_without(
                &self.loaded_entities[entity_index].component_types,
                component_type,
            );
            self.loaded_entities[entity_index].component_types = updated;
        }

        self.metadata.version += 1;
        self.dirty.mark_entity_dirty(entity_id);
        self.dirty
            .mark_component_dirty(entity_id, component_type.clone());
        self.dirty.mark_meta_dirty();
        true
    }

    /// Despawn an entity, removing all its components.
    pub fn despawn_entity(&mut self, entity_id: EntityId) {
        self.loaded_components
            .retain(|component| component.entity_id != entity_id);
        self.loaded_entities
            .retain(|entity| entity.entity_id != entity_id);

        self.metadata.entity_ids = entity_ids_without(&self.metadata.entity_ids, entity_id);
        self.metadata.entity_count = self.metadata.entity_count.saturating_sub(1);
        self.metadata.version += 1;

        self.dirty.mark_despawned(entity_id);
        self.dirty.mark_meta_dirty();
    }

    /// Flush all dirty state to Soroban persistent storage.
    ///
    /// Only writes entries that have been modified since the last flush.
    pub fn flush(&mut self, env: &Env) {
        if !self.dirty.is_dirty() {
            return;
        }

        self.flush_metadata(env);
        self.flush_entity_component_lists(env);
        self.flush_components(env);
        self.flush_despawned_entities(env);
        self.dirty.clear();
    }

    /// Returns the current version counter.
    pub fn version(&self) -> u64 {
        self.metadata.version
    }

    /// Returns the next entity ID.
    pub fn next_entity_id(&self) -> EntityId {
        self.metadata.next_entity_id
    }

    /// Returns the number of live entities.
    pub fn entity_count(&self) -> u32 {
        self.metadata.entity_count
    }

    /// Returns all known entity IDs from metadata.
    pub fn entity_ids(&self) -> &soroban_sdk::Vec<EntityId> {
        &self.metadata.entity_ids
    }

    /// Convert loaded state to a `SimpleWorld`.
    ///
    /// Useful for running systems that expect `SimpleWorld`.
    pub fn to_simple_world(&self, env: &Env) -> SimpleWorld {
        let mut world = SimpleWorld::new(env);
        world.next_entity_id = self.metadata.next_entity_id;
        world.version = self.metadata.version;

        for entity in &self.loaded_entities {
            for i in 0..entity.component_types.len() {
                if let Some(component_type) = entity.component_types.get(i) {
                    if let Some(data) = self.get_component(entity.entity_id, &component_type) {
                        world
                            .components
                            .set((entity.entity_id, component_type.clone()), data);
                    }
                }
            }
            world
                .entity_components
                .set(entity.entity_id, entity.component_types.clone());
        }

        world.version = self.metadata.version;
        world
    }

    /// Create a StorageWorld from a SimpleWorld (for migration).
    ///
    /// Marks everything as dirty so the next `flush()` writes all state.
    pub fn from_simple_world(world: &SimpleWorld, env: &Env) -> Self {
        let mut entity_ids = soroban_sdk::Vec::new(env);
        let mut loaded_entities = Vec::new();
        let mut loaded_components = Vec::new();
        let mut dirty = DirtyTracker::new();
        let mut entity_count: u32 = 0;

        for entity_id in world.entity_components.keys().iter() {
            entity_ids.push_back(entity_id);
            entity_count += 1;

            if let Some(component_types) = world.entity_components.get(entity_id) {
                for i in 0..component_types.len() {
                    if let Some(component_type) = component_types.get(i) {
                        if let Some(data) = world.get_component(entity_id, &component_type) {
                            loaded_components.push(LoadedComponent {
                                entity_id,
                                component_type: component_type.clone(),
                                data,
                            });
                            dirty.mark_component_dirty(entity_id, component_type);
                        }
                    }
                }

                loaded_entities.push(LoadedEntity {
                    entity_id,
                    component_types,
                });
                dirty.mark_entity_dirty(entity_id);
                dirty.mark_new_entity(entity_id);
            }
        }

        dirty.mark_meta_dirty();

        Self {
            metadata: WorldMetadata {
                next_entity_id: world.next_entity_id,
                version: world.version,
                entity_count,
                entity_ids,
            },
            loaded_entities,
            loaded_components,
            dirty,
        }
    }

    fn ensure_loaded_entity(&mut self, env: &Env, entity_id: EntityId, component_type: &Symbol) {
        if let Some(entity_index) = loaded_entity_index(&self.loaded_entities, entity_id) {
            if !component_types_contains(
                &self.loaded_entities[entity_index].component_types,
                component_type,
            ) {
                self.loaded_entities[entity_index]
                    .component_types
                    .push_back(component_type.clone());
            }
            return;
        }

        let mut component_types = soroban_sdk::Vec::new(env);
        component_types.push_back(component_type.clone());
        self.loaded_entities.push(LoadedEntity {
            entity_id,
            component_types,
        });
    }

    fn upsert_loaded_component(
        &mut self,
        entity_id: EntityId,
        component_type: &Symbol,
        data: Bytes,
    ) {
        if let Some(component_index) =
            loaded_component_index(&self.loaded_components, entity_id, component_type)
        {
            self.loaded_components[component_index].data = data;
            return;
        }

        self.loaded_components.push(LoadedComponent {
            entity_id,
            component_type: component_type.clone(),
            data,
        });
    }

    fn flush_metadata(&self, env: &Env) {
        if self.dirty.is_meta_dirty() {
            let key = keys::meta_key(env);
            env.storage().persistent().set(&key, &self.metadata);
        }
    }

    fn flush_entity_component_lists(&self, env: &Env) {
        for &entity_id in self.dirty.dirty_entities() {
            if let Some(entity_index) = loaded_entity_index(&self.loaded_entities, entity_id) {
                let key = keys::entity_key(env, entity_id);
                env.storage()
