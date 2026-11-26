mod model;
#[cfg(test)]
mod tests;

pub use model::{StoredComponentRef, TtlPolicy, WorldMetadata};

use self::model::{
    component_refs_contains, component_refs_upsert, component_refs_without, loaded_component_index,
    loaded_entity_index, LoadedComponent, LoadedEntity,
};
use super::dirty_tracker::DirtyTracker;
use super::keys;
use crate::component::ComponentStorage;
use crate::error::CougrError;
use crate::simple_world::{EntityId, SimpleWorld};
use alloc::vec::Vec;
use soroban_sdk::{Bytes, Env, IntoVal, Symbol, TryFromVal, Val};

/// World that reads/writes individual entities from Soroban persistent storage.
///
/// Only dirty state is written back on `flush()`. Entities must be explicitly
/// loaded before they can be queried or modified.
pub struct StorageWorld {
    namespace: Symbol,
    ttl_policy: TtlPolicy,
    metadata: WorldMetadata,
    persisted_entity_count: u32,
    loaded_entities: Vec<LoadedEntity>,
    loaded_components: Vec<LoadedComponent>,
    dirty: DirtyTracker,
}

impl StorageWorld {
    /// Open a namespaced `StorageWorld` from Soroban persistent storage using
    /// the default [`TtlPolicy`].
    ///
    /// If no metadata exists under `namespace`, creates a fresh world.
    pub fn open(env: &Env, namespace: Symbol) -> Self {
        Self::open_with_ttl(env, namespace, TtlPolicy::default())
    }

    /// Open a namespaced `StorageWorld` with an explicit [`TtlPolicy`].
    ///
    /// Pass [`TtlPolicy::disabled()`] to opt out of automatic TTL extension.
    pub fn open_with_ttl(env: &Env, namespace: Symbol, ttl_policy: TtlPolicy) -> Self {
        let key = keys::meta_key(env, &namespace);
        let stored: Option<WorldMetadata> = env.storage().persistent().get(&key);
        let metadata = match stored {
            Some(meta) => {
                if ttl_policy.is_enabled() {
                    let threshold = ttl_policy.threshold.min(ttl_policy.extend_to);
                    env.storage()
                        .persistent()
                        .extend_ttl(&key, threshold, ttl_policy.extend_to);
                }
                meta
            }
            None => WorldMetadata {
                next_entity_id: 1,
                version: 0,
                entity_count: 0,
            },
        };
        let persisted_entity_count = metadata.entity_count;

        Self {
            namespace,
            ttl_policy,
            metadata,
            persisted_entity_count,
            loaded_entities: Vec::new(),
            loaded_components: Vec::new(),
            dirty: DirtyTracker::new(),
        }
    }

    /// Load world metadata from persistent storage under the `"default"` namespace.
    ///
    /// If no metadata exists, creates a fresh world.
    pub fn load_metadata(env: &Env) -> Self {
        Self::open(env, Symbol::new(env, "default"))
    }

    /// Configure the TTL extension policy using a builder pattern.
    pub fn with_ttl_policy(mut self, ttl_policy: TtlPolicy) -> Self {
        self.ttl_policy = ttl_policy;
        self
    }

    /// Update the TTL extension policy.
    pub fn set_ttl_policy(&mut self, ttl_policy: TtlPolicy) {
        self.ttl_policy = ttl_policy;
    }

    /// Returns the active [`TtlPolicy`].
    pub fn ttl_policy(&self) -> TtlPolicy {
        self.ttl_policy
    }

    /// Returns the namespace symbol for this world.
    pub fn namespace(&self) -> &Symbol {
        &self.namespace
    }

    /// Load a single entity's component list and component data from storage.
    pub fn load_entity(&mut self, env: &Env, entity_id: EntityId) -> Result<(), CougrError> {
        if self.dirty.despawned().contains(&entity_id) {
            return Err(CougrError::EntityNotFound);
        }

        if loaded_entity_index(&self.loaded_entities, entity_id).is_some() {
            return Ok(());
        }

        let key = keys::entity_key(env, &self.namespace, entity_id);
        let components: soroban_sdk::Vec<StoredComponentRef> = self
            .read_persistent(env, &key)
            .ok_or(CougrError::EntityNotFound)?;

        for i in 0..components.len() {
            if let Some(cref) = components.get(i) {
                let ckey =
                    keys::component_key(env, &self.namespace, entity_id, &cref.component_type);
                if let Some(data) = self.read_persistent::<_, Bytes>(env, &ckey) {
                    self.loaded_components.push(LoadedComponent {
                        entity_id,
                        component_type: cref.component_type,
                        data,
                        storage: cref.storage,
                    });
                }
            }
        }

        self.loaded_entities.push(LoadedEntity {
            entity_id,
            components,
        });

        let loc_key = keys::entity_index_loc_key(env, &self.namespace, entity_id);
        if let Some(pos) = self.read_persistent::<_, u32>(env, &loc_key) {
            let bucket_idx = pos / keys::ENTITY_INDEX_BUCKET_SIZE;
            let bucket_key = keys::entity_index_bucket_key(env, &self.namespace, bucket_idx);
            if env.storage().persistent().has(&bucket_key) {
                self.extend_entry_ttl(env, &bucket_key);
            }
        }

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

        self.loaded_entities.push(LoadedEntity {
            entity_id,
            components: soroban_sdk::Vec::new(env),
        });

        self.dirty.mark_new_entity(entity_id);
        self.dirty.mark_entity_dirty(entity_id);
        entity_id
    }

    /// Add a component to an entity using [`ComponentStorage::Table`].
    ///
    /// If the entity is not yet cached in memory, it is loaded from persistent
    /// storage first so existing components are preserved. Returns
    /// [`CougrError::EntityNotFound`] if the entity does not exist or has been
    /// despawned.
    pub fn add_component(
        &mut self,
        env: &Env,
        entity_id: EntityId,
        component_type: Symbol,
        data: Bytes,
    ) -> Result<(), CougrError> {
        self.add_component_with_storage(
            env,
            entity_id,
            component_type,
            data,
            ComponentStorage::Table,
        )
    }

    /// Add a component to an entity with an explicit [`ComponentStorage`] kind.
    pub fn add_component_with_storage(
        &mut self,
        env: &Env,
        entity_id: EntityId,
        component_type: Symbol,
        data: Bytes,
        storage: ComponentStorage,
    ) -> Result<(), CougrError> {
        self.ensure_loaded_entity(env, entity_id, &component_type, storage)?;
        self.upsert_loaded_component(entity_id, &component_type, data, storage);

        self.metadata.version += 1;
        self.dirty.mark_entity_dirty(entity_id);
        self.dirty.mark_component_dirty(entity_id, component_type);
        self.dirty.mark_meta_dirty();
        Ok(())
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
    ///
    /// If the entity is not yet cached in memory, it is loaded from persistent
    /// storage first so its component list and remaining components stay intact.
    /// Returns `false` if the entity does not exist or does not have the component.
    pub fn remove_component(
        &mut self,
        env: &Env,
        entity_id: EntityId,
        component_type: &Symbol,
    ) -> bool {
        if loaded_entity_index(&self.loaded_entities, entity_id).is_none()
            && self.load_entity(env, entity_id).is_err()
        {
            return false;
        }

        let Some(component_index) =
            loaded_component_index(&self.loaded_components, entity_id, component_type)
        else {
            return false;
        };

        self.loaded_components.remove(component_index);
        if let Some(entity_index) = loaded_entity_index(&self.loaded_entities, entity_id) {
            let updated = component_refs_without(
                &self.loaded_entities[entity_index].components,
                component_type,
            );
            self.loaded_entities[entity_index].components = updated;
        }

        self.metadata.version += 1;
        self.dirty.mark_entity_dirty(entity_id);
        self.dirty
            .mark_component_removed(entity_id, component_type.clone());
        self.dirty.mark_meta_dirty();
        true
    }

    /// Despawn an entity, removing all its components.
    pub fn despawn_entity(&mut self, entity_id: EntityId) {
        self.loaded_components
            .retain(|component| component.entity_id != entity_id);
        self.loaded_entities
            .retain(|entity| entity.entity_id != entity_id);

        self.metadata.entity_count = self.metadata.entity_count.saturating_sub(1);
        self.metadata.version += 1;

        self.dirty.mark_despawned(entity_id);
        self.dirty.mark_meta_dirty();
    }

    /// Flush all dirty state to Soroban persistent storage.
    ///
    /// Only writes or deletes entries that have been modified since the last flush.
    pub fn flush(&mut self, env: &Env) {
        if !self.dirty.is_dirty() {
            return;
        }

        self.flush_entity_index(env);
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

    /// Read one page (`bucket`) of live entity IDs from the bucketed entity index.
    ///
    /// Each bucket holds up to [`keys::ENTITY_INDEX_BUCKET_SIZE`] (64) IDs.
    pub fn entity_ids_page(&self, env: &Env, bucket: u32) -> soroban_sdk::Vec<EntityId> {
        let key = keys::entity_index_bucket_key(env, &self.namespace, bucket);
        self.read_persistent(env, &key)
            .unwrap_or_else(|| soroban_sdk::Vec::new(env))
    }

    /// Returns the number of buckets needed for the current live entity count.
    pub fn entity_index_bucket_count(&self) -> u32 {
        self.metadata
            .entity_count
            .div_ceil(keys::ENTITY_INDEX_BUCKET_SIZE)
    }

    /// Convert loaded state to a properly indexed [`SimpleWorld`].
    ///
    /// Preserves each component's [`ComponentStorage`] kind so queries and
    /// `get_entities_with_component` work on the returned view.
    pub fn to_simple_world(&self, env: &Env) -> SimpleWorld {
        let mut world = SimpleWorld::new(env);

        for entity in &self.loaded_entities {
            if entity.components.is_empty() {
                world
                    .entity_components
                    .set(entity.entity_id, soroban_sdk::Vec::new(env));
            } else {
                for i in 0..entity.components.len() {
                    if let Some(cref) = entity.components.get(i) {
                        if let Some(data) =
                            self.get_component(entity.entity_id, &cref.component_type)
                        {
                            world.add_component_with_storage(
                                entity.entity_id,
                                cref.component_type,
                                data,
                                cref.storage,
                            );
                        }
                    }
                }
            }
        }

        world.next_entity_id = self.metadata.next_entity_id;
        world.version = self.metadata.version;
        world
    }

    /// Create a [`StorageWorld`] from a [`SimpleWorld`] under the `"default"` namespace.
    ///
    /// Marks everything as dirty so the next `flush()` writes all state.
    pub fn from_simple_world(world: &SimpleWorld, env: &Env) -> Self {
        Self::from_simple_world_with_namespace(world, env, Symbol::new(env, "default"))
    }

    /// Create a namespaced [`StorageWorld`] from a [`SimpleWorld`].
    ///
    /// Reconciles against any existing persisted index in `namespace` so
    /// round-tripping an already-persisted world does not duplicate index entries,
    /// and marks all entities and components in `world` dirty for the next `flush()`.
    pub fn from_simple_world_with_namespace(
        world: &SimpleWorld,
        env: &Env,
        namespace: Symbol,
    ) -> Self {
        let meta_key = keys::meta_key(env, &namespace);
        let persisted_entity_count = env
            .storage()
            .persistent()
            .get::<_, WorldMetadata>(&meta_key)
            .map(|m| m.entity_count)
            .unwrap_or(0);

        let mut loaded_entities = Vec::new();
        let mut loaded_components = Vec::new();
        let mut dirty = DirtyTracker::new();
        let mut entity_count: u32 = 0;

        if persisted_entity_count > 0 {
            let bucket_count = persisted_entity_count.div_ceil(keys::ENTITY_INDEX_BUCKET_SIZE);
            for bucket_idx in 0..bucket_count {
                let bucket_key = keys::entity_index_bucket_key(env, &namespace, bucket_idx);
                if let Some(bucket) = env
                    .storage()
                    .persistent()
                    .get::<_, soroban_sdk::Vec<EntityId>>(&bucket_key)
                {
                    for i in 0..bucket.len() {
                        if let Some(existing_id) = bucket.get(i) {
                            if !world.entity_components.contains_key(existing_id) {
                                dirty.mark_despawned(existing_id);
                            }
                        }
                    }
                }
            }
        }

        for entity_id in world.entity_components.keys().iter() {
            entity_count += 1;

            if let Some(component_types) = world.entity_components.get(entity_id) {
                let mut refs = soroban_sdk::Vec::new(env);
                for i in 0..component_types.len() {
                    if let Some(component_type) = component_types.get(i) {
                        if let Some((data, storage)) =
                            simple_world_component(world, entity_id, &component_type)
                        {
                            refs.push_back(StoredComponentRef {
                                component_type: component_type.clone(),
                                storage,
                            });
                            loaded_components.push(LoadedComponent {
                                entity_id,
                                component_type: component_type.clone(),
                                data,
                                storage,
                            });
                            dirty.mark_component_dirty(entity_id, component_type);
                        }
                    }
                }

                loaded_entities.push(LoadedEntity {
                    entity_id,
                    components: refs,
                });
                dirty.mark_entity_dirty(entity_id);
                let loc_key = keys::entity_index_loc_key(env, &namespace, entity_id);
                if !env.storage().persistent().has(&loc_key) {
                    dirty.mark_new_entity(entity_id);
                }
            }
        }

        dirty.mark_meta_dirty();

        Self {
            namespace,
            ttl_policy: TtlPolicy::default(),
            metadata: WorldMetadata {
                next_entity_id: world.next_entity_id,
                version: world.version,
                entity_count,
            },
            persisted_entity_count,
            loaded_entities,
            loaded_components,
            dirty,
        }
    }

    /// Diff a [`SimpleWorld`] view against the currently loaded state and mark
    /// only actual changes dirty (changed data/storage, added/removed components,
    /// newly spawned and despawned entities).
    ///
    /// Note: `SimpleWorld` drops an entity from `entity_components` when its
    /// last component is removed, so `apply_simple_world` cannot distinguish
    /// "removed last component" from "despawned" and despawns any loaded entity
    /// absent from `simple.entity_components`.
    pub fn apply_simple_world(&mut self, env: &Env, simple: &SimpleWorld) {
        let old_next_entity_id = self.metadata.next_entity_id;

        // 1. Loaded entities missing from the SimpleWorld view were despawned.
        let loaded_ids: Vec<EntityId> = self.loaded_entities.iter().map(|e| e.entity_id).collect();
        for entity_id in loaded_ids {
            if !simple.entity_components.contains_key(entity_id) {
                self.despawn_entity(entity_id);
            }
        }

        // 2. Diff remaining loaded entities and add newly spawned entities.
        for entity_id in simple.entity_components.keys().iter() {
            let Some(view_types) = simple.entity_components.get(entity_id) else {
                continue;
            };

            if entity_id >= old_next_entity_id {
                // Newly spawned in the SimpleWorld view.
                let mut refs = soroban_sdk::Vec::new(env);
                for i in 0..view_types.len() {
                    if let Some(component_type) = view_types.get(i) {
                        if let Some((data, storage)) =
                            simple_world_component(simple, entity_id, &component_type)
                        {
                            refs.push_back(StoredComponentRef {
                                component_type: component_type.clone(),
                                storage,
                            });
                            self.loaded_components.push(LoadedComponent {
                                entity_id,
                                component_type: component_type.clone(),
                                data,
                                storage,
                            });
                            self.dirty.mark_component_dirty(entity_id, component_type);
                        }
                    }
                }
                self.loaded_entities.push(LoadedEntity {
                    entity_id,
                    components: refs,
                });
                self.metadata.entity_count += 1;
                self.metadata.version += 1;
                self.dirty.mark_new_entity(entity_id);
                self.dirty.mark_entity_dirty(entity_id);
                self.dirty.mark_meta_dirty();
                continue;
            }

            // Existing loaded entity: diff component set and values.
            let Some(entity_idx) = loaded_entity_index(&self.loaded_entities, entity_id) else {
                continue;
            };

            // Check for removed components.
            let mut removed_types: Vec<Symbol> = Vec::new();
            let existing_refs = self.loaded_entities[entity_idx].components.clone();
            for i in 0..existing_refs.len() {
                if let Some(cref) = existing_refs.get(i) {
                    if !symbol_vec_contains(&view_types, &cref.component_type) {
                        removed_types.push(cref.component_type);
                    }
                }
            }
            for rtype in &removed_types {
                self.remove_component(env, entity_id, rtype);
            }

            // Check for added or modified components.
            for i in 0..view_types.len() {
                let Some(component_type) = view_types.get(i) else {
                    continue;
                };
                let Some((new_data, new_storage)) =
                    simple_world_component(simple, entity_id, &component_type)
                else {
                    continue;
                };

                if let Some(cidx) =
                    loaded_component_index(&self.loaded_components, entity_id, &component_type)
                {
                    let existing = &self.loaded_components[cidx];
                    let data_changed = existing.data != new_data;
                    let storage_changed = existing.storage != new_storage;
                    if data_changed || storage_changed {
                        self.loaded_components[cidx].data = new_data;
                        self.loaded_components[cidx].storage = new_storage;
                        self.dirty
                            .mark_component_dirty(entity_id, component_type.clone());
                        if storage_changed {
                            component_refs_upsert(
                                &mut self.loaded_entities[entity_idx].components,
                                &component_type,
                                new_storage,
                            );
                            self.dirty.mark_entity_dirty(entity_id);
                        }
                        self.metadata.version += 1;
                        self.dirty.mark_meta_dirty();
                    }
                } else {
                    // Newly added component on existing loaded entity.
                    component_refs_upsert(
                        &mut self.loaded_entities[entity_idx].components,
                        &component_type,
                        new_storage,
                    );
                    self.loaded_components.push(LoadedComponent {
                        entity_id,
                        component_type: component_type.clone(),
                        data: new_data,
                        storage: new_storage,
                    });
                    self.metadata.version += 1;
                    self.dirty.mark_entity_dirty(entity_id);
                    self.dirty.mark_component_dirty(entity_id, component_type);
                    self.dirty.mark_meta_dirty();
                }
            }
        }

        if simple.next_entity_id > self.metadata.next_entity_id {
            self.metadata.next_entity_id = simple.next_entity_id;
            self.dirty.mark_meta_dirty();
        }
    }

    fn ensure_loaded_entity(
        &mut self,
        env: &Env,
        entity_id: EntityId,
        component_type: &Symbol,
        storage: ComponentStorage,
    ) -> Result<(), CougrError> {
        if loaded_entity_index(&self.loaded_entities, entity_id).is_none() {
            self.load_entity(env, entity_id)?;
        }

        let entity_index = loaded_entity_index(&self.loaded_entities, entity_id)
            .ok_or(CougrError::EntityNotFound)?;

        if !component_refs_contains(
            &self.loaded_entities[entity_index].components,
            component_type,
        ) {
            self.loaded_entities[entity_index]
                .components
                .push_back(StoredComponentRef {
                    component_type: component_type.clone(),
                    storage,
                });
        } else {
            component_refs_upsert(
                &mut self.loaded_entities[entity_index].components,
                component_type,
                storage,
            );
        }
        Ok(())
    }

    fn upsert_loaded_component(
        &mut self,
        entity_id: EntityId,
        component_type: &Symbol,
        data: Bytes,
        storage: ComponentStorage,
    ) {
        if let Some(component_index) =
            loaded_component_index(&self.loaded_components, entity_id, component_type)
        {
            self.loaded_components[component_index].data = data;
            self.loaded_components[component_index].storage = storage;
            return;
        }

        self.loaded_components.push(LoadedComponent {
            entity_id,
            component_type: component_type.clone(),
            data,
            storage,
        });
    }

    fn extend_entry_ttl<K>(&self, env: &Env, key: &K)
    where
        K: IntoVal<Env, Val>,
    {
        if self.ttl_policy.is_enabled() {
            let threshold = self.ttl_policy.threshold.min(self.ttl_policy.extend_to);
            env.storage()
                .persistent()
                .extend_ttl(key, threshold, self.ttl_policy.extend_to);
        }
    }

    fn read_persistent<K, V>(&self, env: &Env, key: &K) -> Option<V>
    where
        K: IntoVal<Env, Val>,
        V: TryFromVal<Env, Val>,
    {
        let val: Option<V> = env.storage().persistent().get(key);
        if val.is_some() {
            self.extend_entry_ttl(env, key);
        }
        val
    }

    fn write_persistent<K, V>(&self, env: &Env, key: &K, value: &V)
    where
        K: IntoVal<Env, Val>,
        V: IntoVal<Env, Val>,
    {
        env.storage().persistent().set(key, value);
        self.extend_entry_ttl(env, key);
    }

    fn flush_entity_index(&mut self, env: &Env) {
        let new_entities = self.dirty.new_entities();
        let despawned = self.dirty.despawned();

        // Swap-remove persisted despawned entities from the bucketed index.
        for &entity_id in despawned {
            if new_entities.contains(&entity_id) {
                continue;
            }
            let loc_key = keys::entity_index_loc_key(env, &self.namespace, entity_id);
            let Some(pos) = env.storage().persistent().get::<_, u32>(&loc_key) else {
                continue;
            };

            let last_pos = self.persisted_entity_count.saturating_sub(1);
            let last_bucket_idx = last_pos / keys::ENTITY_INDEX_BUCKET_SIZE;
            let last_bucket_key =
                keys::entity_index_bucket_key(env, &self.namespace, last_bucket_idx);

            if let Some(mut last_bucket) = env
                .storage()
                .persistent()
                .get::<_, soroban_sdk::Vec<EntityId>>(&last_bucket_key)
            {
                let last_idx = last_bucket.len().saturating_sub(1);
                if let Some(last_id) = last_bucket.get(last_idx) {
                    last_bucket.pop_back();
                    if pos == last_pos {
                        if last_bucket.is_empty() {
                            env.storage().persistent().remove(&last_bucket_key);
                        } else {
                            self.write_persistent(env, &last_bucket_key, &last_bucket);
                        }
                    } else {
                        let bucket_idx = pos / keys::ENTITY_INDEX_BUCKET_SIZE;
                        let offset = pos % keys::ENTITY_INDEX_BUCKET_SIZE;
                        if bucket_idx == last_bucket_idx {
                            last_bucket.set(offset, last_id);
                            self.write_persistent(env, &last_bucket_key, &last_bucket);
                        } else {
                            if last_bucket.is_empty() {
                                env.storage().persistent().remove(&last_bucket_key);
                            } else {
                                self.write_persistent(env, &last_bucket_key, &last_bucket);
                            }
                            let bucket_key =
                                keys::entity_index_bucket_key(env, &self.namespace, bucket_idx);
                            if let Some(mut bucket) = env
                                .storage()
                                .persistent()
                                .get::<_, soroban_sdk::Vec<EntityId>>(&bucket_key)
                            {
                                bucket.set(offset, last_id);
                                self.write_persistent(env, &bucket_key, &bucket);
                            }
                        }
                        let moved_loc_key =
                            keys::entity_index_loc_key(env, &self.namespace, last_id);
                        self.write_persistent(env, &moved_loc_key, &pos);
                    }
                }
            }

            env.storage().persistent().remove(&loc_key);
            self.persisted_entity_count = self.persisted_entity_count.saturating_sub(1);
        }

        // Append newly spawned (and still live) entities to the last bucket.
        for &entity_id in new_entities {
            if despawned.contains(&entity_id) {
                continue;
            }
            let pos = self.persisted_entity_count;
            let bucket_idx = pos / keys::ENTITY_INDEX_BUCKET_SIZE;
            let bucket_key = keys::entity_index_bucket_key(env, &self.namespace, bucket_idx);
            let mut bucket: soroban_sdk::Vec<EntityId> = env
                .storage()
                .persistent()
                .get(&bucket_key)
                .unwrap_or_else(|| soroban_sdk::Vec::new(env));
            bucket.push_back(entity_id);
            self.write_persistent(env, &bucket_key, &bucket);

            let loc_key = keys::entity_index_loc_key(env, &self.namespace, entity_id);
            self.write_persistent(env, &loc_key, &pos);
            self.persisted_entity_count += 1;
        }

        self.metadata.entity_count = self.persisted_entity_count;
    }

    fn flush_metadata(&self, env: &Env) {
        if self.dirty.is_meta_dirty() {
            let key = keys::meta_key(env, &self.namespace);
            self.write_persistent(env, &key, &self.metadata);
        }
    }

    fn flush_entity_component_lists(&self, env: &Env) {
        for &entity_id in self.dirty.dirty_entities() {
            if self.dirty.despawned().contains(&entity_id) {
                continue;
            }
            if let Some(entity_index) = loaded_entity_index(&self.loaded_entities, entity_id) {
                let key = keys::entity_key(env, &self.namespace, entity_id);
                self.write_persistent(env, &key, &self.loaded_entities[entity_index].components);
            }
        }
    }

    fn flush_components(&self, env: &Env) {
        for (entity_id, component_type) in self.dirty.dirty_components() {
            if self.dirty.despawned().contains(entity_id) {
                continue;
            }

            if let Some(component_index) =
                loaded_component_index(&self.loaded_components, *entity_id, component_type)
            {
                let key = keys::component_key(env, &self.namespace, *entity_id, component_type);
                self.write_persistent(env, &key, &self.loaded_components[component_index].data);
            }
        }

        for (entity_id, component_type) in self.dirty.removed_components() {
            if self.dirty.despawned().contains(entity_id) {
                continue;
            }
            let key = keys::component_key(env, &self.namespace, *entity_id, component_type);
            env.storage().persistent().remove(&key);
        }
    }

    fn flush_despawned_entities(&self, env: &Env) {
        for &entity_id in self.dirty.despawned() {
            let entity_key = keys::entity_key(env, &self.namespace, entity_id);
            if let Some(components) = env
                .storage()
                .persistent()
                .get::<_, soroban_sdk::Vec<StoredComponentRef>>(&entity_key)
            {
                for i in 0..components.len() {
                    if let Some(cref) = components.get(i) {
                        let component_key = keys::component_key(
                            env,
                            &self.namespace,
                            entity_id,
                            &cref.component_type,
                        );
                        env.storage().persistent().remove(&component_key);
                    }
                }
            }
            env.storage().persistent().remove(&entity_key);
        }
    }
}

fn simple_world_component(
    world: &SimpleWorld,
    entity_id: EntityId,
    component_type: &Symbol,
) -> Option<(Bytes, ComponentStorage)> {
    if let Some(data) = world.components.get((entity_id, component_type.clone())) {
        Some((data, ComponentStorage::Table))
    } else {
        world
            .sparse_components
            .get((entity_id, component_type.clone()))
            .map(|data| (data, ComponentStorage::Sparse))
    }
}

fn symbol_vec_contains(vec: &soroban_sdk::Vec<Symbol>, target: &Symbol) -> bool {
    for i in 0..vec.len() {
        if let Some(item) = vec.get(i) {
            if &item == target {
                return true;
            }
        }
    }
    false
}
