mod indexing;
#[cfg(test)]
mod tests;

use crate::component::{ComponentStorage, ComponentTrait};
use crate::ecs_events::ObservableComponentTrait;
use crate::rich_component::{rich_component_key, RichComponentTrait};
use soroban_sdk::{contracttype, Bytes, Env, IntoVal, Map, Symbol, TryFromVal, Val, Vec};

/// Simple entity ID type for Soroban-optimized ECS.
pub type EntityId = u32;

/// Simplified game world optimized for Soroban on-chain storage.
///
/// Uses `Map`-based storage for O(log n) component lookups instead of
/// linear scans. This is the recommended ECS container for Soroban contracts.
///
/// ## Dual-Map storage
///
/// Components are split into two maps based on their `ComponentStorage` kind:
/// - **Table** (`components`): Frequently-iterated components (e.g., Position, Velocity).
///   Queried by `get_entities_with_component()`.
/// - **Sparse** (`sparse_components`): Infrequently-accessed marker or tag components.
///   Not included in the default entity query; use `get_all_entities_with_component()` to include them.
///
/// Both maps are transparent to `get_component()`, `has_component()`, and `remove_component()`.
///
/// # Example
/// ```
/// use cougr_core::component::ComponentStorage;
/// use cougr_core::simple_world::SimpleWorld;
/// use soroban_sdk::{symbol_short, Bytes, Env};
///
/// let env = Env::default();
/// let mut world = SimpleWorld::new(&env);
/// let entity_id = world.spawn_entity();
/// world.add_component(entity_id, symbol_short!("position"), Bytes::new(&env));
/// world.add_component_with_storage(
///     entity_id,
///     symbol_short!("marker"),
///     Bytes::new(&env),
///     ComponentStorage::Sparse,
/// );
/// assert!(world.has_component(entity_id, &symbol_short!("position")));
/// ```
#[contracttype]
#[derive(Clone, Debug)]
pub struct SimpleWorld {
    pub(crate) next_entity_id: u32,
    /// Table component data keyed by (entity_id, component_type).
    pub(crate) components: Map<(u32, Symbol), Bytes>,
    /// Sparse component data keyed by (entity_id, component_type).
    pub(crate) sparse_components: Map<(u32, Symbol), Bytes>,
    /// Tracks which component types each entity has.
    pub(crate) entity_components: Map<u32, Vec<Symbol>>,
    /// Direct index for frequently queried table-backed components.
    pub(crate) table_index: Map<Symbol, Vec<u32>>,
    /// Direct index for all components regardless of backing storage.
    pub(crate) all_index: Map<Symbol, Vec<u32>>,
    /// Version counter incremented on structural changes (add/remove/despawn).
    /// Used for query cache invalidation.
    pub(crate) version: u64,
}

impl SimpleWorld {
    pub fn new(env: &Env) -> Self {
        Self {
            next_entity_id: 1,
            components: Map::new(env),
            sparse_components: Map::new(env),
            entity_components: Map::new(env),
            table_index: Map::new(env),
            all_index: Map::new(env),
            version: 0,
        }
    }

    /// Returns the current world version for cache invalidation.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Returns the next entity ID that will be assigned on spawn.
    pub fn next_entity_id(&self) -> EntityId {
        self.next_entity_id
    }

    /// Returns the environment backing this world storage.
    pub fn env(&self) -> &Env {
        self.components.env()
    }

    pub fn spawn_entity(&mut self) -> EntityId {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        id
    }

    fn has_component_in_table(&self, entity_id: EntityId, component_type: &Symbol) -> bool {
        self.components
            .contains_key((entity_id, component_type.clone()))
    }

    fn has_component_in_sparse(&self, entity_id: EntityId, component_type: &Symbol) -> bool {
        self.sparse_components
            .contains_key((entity_id, component_type.clone()))
    }

    /// Add a component using the default **Table** storage.
    pub fn add_component(&mut self, entity_id: EntityId, component_type: Symbol, data: Bytes) {
        self.add_component_with_storage(entity_id, component_type, data, ComponentStorage::Table);
    }

    /// Add a component, routing to the Table or Sparse map based on `storage`.
    pub fn add_component_with_storage(
        &mut self,
        entity_id: EntityId,
        component_type: Symbol,
        data: Bytes,
        storage: ComponentStorage,
    ) {
        self.version += 1;
        let was_in_table = self.has_component_in_table(entity_id, &component_type);
        let was_in_sparse = self.has_component_in_sparse(entity_id, &component_type);

        match storage {
            ComponentStorage::Table => {
                self.components
                    .set((entity_id, component_type.clone()), data);
                if was_in_sparse {
                    self.sparse_components
                        .remove((entity_id, component_type.clone()));
                }
            }
            ComponentStorage::Sparse => {
                self.sparse_components
                    .set((entity_id, component_type.clone()), data);
                if was_in_table {
                    self.components.remove((entity_id, component_type.clone()));
                }
            }
        }

        let mut types = self
            .entity_components
            .get(entity_id)
            .unwrap_or_else(|| Vec::new(self.components.env()));

        let mut found = false;
        for i in 0..types.len() {
            if let Some(t) = types.get(i) {
                if t == component_type {
                    found = true;
                    break;
                }
            }
        }
        if !found {
            types.push_back(component_type.clone());
        }
        self.entity_components.set(entity_id, types);

        indexing::push_index(&mut self.all_index, &component_type, entity_id);
        match storage {
            ComponentStorage::Table => {
                indexing::push_index(&mut self.table_index, &component_type, entity_id);
            }
            ComponentStorage::Sparse => {
                indexing::remove_from_index(&mut self.table_index, &component_type, entity_id);
            }
        }
    }

    /// Get a component's data, checking both Table and Sparse maps transparently.
    pub fn get_component(&self, entity_id: EntityId, component_type: &Symbol) -> Option<Bytes> {
        self.components
            .get((entity_id, component_type.clone()))
            .or_else(|| {
                self.sparse_components
                    .get((entity_id, component_type.clone()))
            })
    }

    /// Remove a component from both Table and Sparse maps transparently.
    pub fn remove_component(&mut self, entity_id: EntityId, component_type: &Symbol) -> bool {
        self.version += 1;
        let removed = self
            .components
            .remove((entity_id, component_type.clone()))
            .or_else(|| {
                self.sparse_components
                    .remove((entity_id, component_type.clone()))
            });

        if removed.is_some() {
            if let Some(types) = self.entity_components.get(entity_id) {
                let env = self.components.env();
                let mut new_types = Vec::new(env);
                for i in 0..types.len() {
                    if let Some(t) = types.get(i) {
                        if &t != component_type {
                            new_types.push_back(t);
                        }
                    }
                }
                if new_types.is_empty() {
                    self.entity_components.remove(entity_id);
                } else {
                    self.entity_components.set(entity_id, new_types);
                }
            }
            indexing::remove_from_index(&mut self.all_index, component_type, entity_id);
            indexing::remove_from_index(&mut self.table_index, component_type, entity_id);
            true
        } else {
            false
        }
    }

    /// Check if an entity has a component in either Table or Sparse storage.
    pub fn has_component(&self, entity_id: EntityId, component_type: &Symbol) -> bool {
        self.has_component_in_table(entity_id, component_type)
            || self.has_component_in_sparse(entity_id, component_type)
    }

    pub fn get_entities_with_component(&self, component_type: &Symbol, env: &Env) -> Vec<EntityId> {
        self.table_index
            .get(component_type.clone())
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Get entities that have the given component in **Table** storage only.
    /// This is the fast path for querying frequently-iterated components.
    pub fn get_table_entities_with_component(
        &self,
        component_type: &Symbol,
        env: &Env,
    ) -> Vec<EntityId> {
        self.table_index
            .get(component_type.clone())
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Get entities that have the given component in **either** Table or Sparse storage.
    pub fn get_all_entities_with_component(
        &self,
        component_type: &Symbol,
        env: &Env,
    ) -> Vec<EntityId> {
        self.all_index
            .get(component_type.clone())
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Returns the number of entities indexed for a component in table storage only.
    pub fn table_component_count(&self, component_type: &Symbol) -> usize {
        self.table_index
            .get(component_type.clone())
            .map(|entities| entities.len())
            .unwrap_or(0)
            .try_into()
            .unwrap()
    }

    /// Returns the number of entities indexed for a component across both storage classes.
    pub fn component_count(&self, component_type: &Symbol) -> usize {
        self.all_index
            .get(component_type.clone())
            .map(|entities| entities.len())
            .unwrap_or(0)
            .try_into()
            .unwrap()
    }

    /// Get a component and deserialize it into the concrete type.
    ///
    /// # Example
    /// ```
    /// use cougr_core::component::Position;
    /// use cougr_core::simple_world::SimpleWorld;
    /// use soroban_sdk::Env;
    ///
    /// let env = Env::default();
    /// let mut world = SimpleWorld::new(&env);
    /// let entity_id = world.spawn_entity();
    /// world.set_typed(&env, entity_id, &Position::new(10, 20));
    /// let pos: Option<Position> = world.get_typed::<Position>(&env, entity_id);
    /// assert_eq!(pos.unwrap().x, 10);
    /// ```
    pub fn get_typed<T: ComponentTrait>(&self, env: &Env, entity_id: EntityId) -> Option<T> {
        let bytes = self.get_component(entity_id, &T::component_type())?;
        T::deserialize(env, &bytes)
    }

    /// Serialize a component and store it, using the type's default storage kind.
    ///
    /// # Example
    /// ```
    /// use cougr_core::component::Position;
    /// use cougr_core::simple_world::SimpleWorld;
    /// use soroban_sdk::Env;
    ///
    /// let env = Env::default();
    /// let mut world = SimpleWorld::new(&env);
    /// let entity_id = world.spawn_entity();
    /// world.set_typed(&env, entity_id, &Position::new(10, 20));
    /// assert!(world.has_typed::<Position>(entity_id));
    /// ```
    pub fn set_typed<T: ComponentTrait>(&mut self, env: &Env, entity_id: EntityId, component: &T) {
        let symbol = T::component_type();
        let data = component.serialize(env);
        let storage = T::default_storage();
        self.add_component_with_storage(entity_id, symbol, data, storage);
    }

    /// Check if an entity has a component of the given type.
    pub fn has_typed<T: ComponentTrait>(&self, entity_id: EntityId) -> bool {
        self.has_component(entity_id, &T::component_type())
    }

    /// Remove a component of the given type from an entity.
    pub fn remove_typed<T: ComponentTrait>(&mut self, entity_id: EntityId) -> bool {
        self.remove_component(entity_id, &T::component_type())
    }

    /// Store a rich component using Soroban's `#[contracttype]` XDR codec.
    ///
    /// Supports `Vec`, `String`, `Option`, nested structs, and enums with data.
    /// Rich components live in **instance storage** and are keyed separately
    /// from the ECS `Map`-backed components. Entity IDs are shared.
    ///
    /// # Example
    /// ```no_run
    /// use cougr_core::impl_rich_component;
    /// use cougr_core::simple_world::SimpleWorld;
    /// use soroban_sdk::{contracttype, Env, String};
    ///
    /// #[contracttype]
    /// #[derive(Clone)]
    /// pub struct PlayerName { pub name: String }
    ///
    /// impl_rich_component!(PlayerName, "player_name");
    ///
    /// let env = Env::default();
    /// let mut world = SimpleWorld::new(&env);
    /// let entity = world.spawn_entity();
    /// world.set_rich(&env, entity, &PlayerName { name: String::from_str(&env, "Alice") });
    /// ```
    pub fn set_rich<T>(&mut self, env: &Env, entity_id: EntityId, component: &T)
    where
        T: RichComponentTrait + IntoVal<Env, Val>,
    {
        let key = rich_component_key(env, entity_id, T::component_type(env));
        env.storage().instance().set(&key, component);
    }

    /// Retrieve a rich component from instance storage, or `None` if absent.
    pub fn get_rich<T>(&self, env: &Env, entity_id: EntityId) -> Option<T>
    where
        T: RichComponentTrait + TryFromVal<Env, Val>,
    {
        let key = rich_component_key(env, entity_id, T::component_type(env));
        env.storage().instance().get(&key)
    }

    /// Remove a rich component from instance storage.
    pub fn remove_rich<T: RichComponentTrait>(&mut self, env: &Env, entity_id: EntityId) {
        let key = rich_component_key(env, entity_id, T::component_type(env));
        env.storage().instance().remove(&key);
    }

    /// Store a component and emit a structured Soroban `set` event.
    ///
    /// The component type must implement [`ObservableComponentTrait`] (via
    /// [`impl_component_observed!`]) and derive `#[contracttype]`. The event
    /// has topics `(COUGR, set, component_symbol)` and carries the
    /// deserialized struct as data.
    ///
    /// [`ObservableComponentTrait`]: crate::ecs_events::ObservableComponentTrait
    pub fn set_typed_observed<T>(&mut self, env: &Env, entity_id: EntityId, component: &T)
    where
        T: ObservableComponentTrait,
    {
        let data = component.serialize(env);
        self.add_component_with_storage(
            entity_id,
            T::component_type(),
            data.clone(),
            T::default_storage(),
        );
        T::emit_set_event(env, entity_id, &data);
    }

    /// Remove a component and emit a structured Soroban `del` event if the
    /// component was present.
    ///
    /// The event has topics `(COUGR, del, component_type)` and carries the
    /// entity ID as data.
    ///
    /// Returns `true` if a component was removed.
    pub fn remove_component_observed(
        &mut self,
        entity_id: EntityId,
        component_type: &Symbol,
        env: &Env,
    ) -> bool {
        let removed = self.remove_component(entity_id, component_type);
        if removed {
            crate::ecs_events::ComponentRemovedEvent {
                component_type: component_type.clone(),
                entity_id,
            }
            .publish(env);
        }
        removed
    }

    pub fn despawn_entity(&mut self, entity_id: EntityId) {
        self.version += 1;
        if let Some(types) = self.entity_components.get(entity_id) {
            for i in 0..types.len() {
                if let Some(t) = types.get(i) {
                    self.components.remove((entity_id, t.clone()));
                    self.sparse_components.remove((entity_id, t.clone()));
                    indexing::remove_from_index(&mut self.all_index, &t, entity_id);
                    indexing::remove_from_index(&mut self.table_index, &t, entity_id);
                }
            }
        }
        self.entity_components.remove(entity_id);
    }

    // ─── Soroban instance-storage persistence ─────────────────────────────────

    /// Load a `SimpleWorld` from Soroban instance storage.
    ///
    /// Returns a fresh empty world if the key does not exist yet. Pair with
    /// [`save_to_instance`] to build a clean contract entrypoint pattern.
    /// Prefer the [`SorobanGame`](crate::game::SorobanGame) trait +
    /// [`impl_soroban_game!`] macro to avoid repeating the storage key in
    /// every entrypoint.
    ///
    /// # Example
    /// ```no_run
    /// use cougr_core::simple_world::SimpleWorld;
    /// use soroban_sdk::{symbol_short, Env};
    ///
    /// let env = Env::default();
    /// let key = symbol_short!("WORLD");
    /// let world = SimpleWorld::load_from_instance(&env, &key);
    /// ```
    pub fn load_from_instance(env: &Env, key: &Symbol) -> Self {
        env.storage()
            .instance()
            .get(key)
            .unwrap_or_else(|| SimpleWorld::new(env))
    }

    /// Persist this world to Soroban instance storage under `key`.
    ///
    /// # Example
    /// ```no_run
    /// use cougr_core::simple_world::SimpleWorld;
    /// use soroban_sdk::{symbol_short, Env};
    ///
    /// let env = Env::default();
    /// let key = symbol_short!("WORLD");
    /// let world = SimpleWorld::new(&env);
    /// world.save_to_instance(&env, &key);
    /// ```
    pub fn save_to_instance(&self, env: &Env, key: &Symbol) {
        env.storage().instance().set(key, self);
    }

    /// Store a rich component and emit a
    /// [`RichComponentChangedEvent`](crate::ecs_events::RichComponentChangedEvent).
    ///
    /// Off-chain indexers subscribed to `("COUGR", "rich", <type>)` topics
    /// receive the notification and can query instance storage for the full
    /// updated value.
    pub fn set_rich_observed<T>(&mut self, env: &Env, entity_id: EntityId, component: &T)
    where
        T: RichComponentTrait + IntoVal<Env, Val>,
    {
        self.set_rich(env, entity_id, component);
        crate::ecs_events::RichComponentChangedEvent {
            component_type: T::component_type(env),
            entity_id,
        }
        .publish(env);
    }

    /// Remove a rich component and emit a `del` event if it was present.
    pub fn remove_rich_observed<T: RichComponentTrait>(&mut self, env: &Env, entity_id: EntityId) {
        let key = rich_component_key(env, entity_id, T::component_type(env));
        if env.storage().instance().has(&key) {
            env.storage().instance().remove(&key);
            crate::ecs_events::ComponentRemovedEvent {
                component_type: T::component_type(env),
                entity_id,
            }
            .publish(env);
        }
    }
}
