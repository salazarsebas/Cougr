use crate::simple_world::{EntityId, SimpleWorld};
use alloc::vec::Vec;
use soroban_sdk::{Bytes, Symbol};

/// Callback invoked when a component is added to an entity.
pub type OnAddHook = fn(entity_id: EntityId, component_type: &Symbol, data: &Bytes);

/// Callback invoked when a component is removed from an entity.
pub type OnRemoveHook = fn(entity_id: EntityId, component_type: &Symbol);

/// Registry of component lifecycle hooks.
///
/// Hooks are runtime-only (not persisted to on-chain storage) and must be
/// re-registered each contract invocation. They allow reactive patterns
/// such as updating indexes or cleaning up related state.
///
/// # Example
/// ```
/// use cougr_core::runtime::HookRegistry;
/// use soroban_sdk::{symbol_short, Bytes, Symbol};
///
/// fn on_add(_entity_id: u32, _ctype: &Symbol, _data: &Bytes) {}
///
/// let mut hooks = HookRegistry::new();
/// hooks.on_add(symbol_short!("pos"), on_add);
/// assert_eq!(hooks.add_hook_count(), 1);
/// ```
pub struct HookRegistry {
    add_hooks: Vec<(Symbol, OnAddHook)>,
    remove_hooks: Vec<(Symbol, OnRemoveHook)>,
}

impl HookRegistry {
    /// Create an empty hook registry.
    pub fn new() -> Self {
        Self {
            add_hooks: Vec::new(),
            remove_hooks: Vec::new(),
        }
    }

    /// Register a hook that fires when a component of the given type is added.
    pub fn on_add(&mut self, component_type: Symbol, hook: OnAddHook) {
        self.add_hooks.push((component_type, hook));
    }

    /// Register a hook that fires when a component of the given type is removed.
    pub fn on_remove(&mut self, component_type: Symbol, hook: OnRemoveHook) {
        self.remove_hooks.push((component_type, hook));
    }

    /// Fire all registered `on_add` hooks for the given component type.
    pub fn fire_on_add(&self, entity_id: EntityId, component_type: &Symbol, data: &Bytes) {
        for (ctype, hook) in &self.add_hooks {
            if ctype == component_type {
                hook(entity_id, component_type, data);
            }
        }
    }

    /// Fire all registered `on_remove` hooks for the given component type.
    pub fn fire_on_remove(&self, entity_id: EntityId, component_type: &Symbol) {
        for (ctype, hook) in &self.remove_hooks {
            if ctype == component_type {
                hook(entity_id, component_type);
            }
        }
    }

    /// Returns the number of registered add hooks.
    pub fn add_hook_count(&self) -> usize {
        self.add_hooks.len()
    }

    /// Returns the number of registered remove hooks.
    pub fn remove_hook_count(&self) -> usize {
        self.remove_hooks.len()
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A wrapper around `SimpleWorld` that fires lifecycle hooks on component mutations.
///
/// Since `SimpleWorld` is a `#[contracttype]` struct and cannot hold function pointers,
/// this wrapper carries a separate `HookRegistry` alongside the world.
///
/// # Example
/// ```
/// use cougr_core::runtime::HookedWorld;
/// use cougr_core::SimpleWorld;
/// use soroban_sdk::{symbol_short, Bytes, Env, Symbol};
///
/// fn on_add(_entity_id: u32, _ctype: &Symbol, _data: &Bytes) {}
///
/// let env = Env::default();
/// let world = SimpleWorld::new(&env);
/// let mut hooked = HookedWorld::new(world);
/// let entity_id = hooked.spawn_entity();
/// hooked.hooks_mut().on_add(symbol_short!("pos"), on_add);
/// hooked.add_component(entity_id, symbol_short!("pos"), Bytes::new(&env));
/// assert!(hooked.has_component(entity_id, &symbol_short!("pos")));
/// ```
pub struct HookedWorld {
    world: SimpleWorld,
    hooks: HookRegistry,
}

impl HookedWorld {
    /// Wrap a `SimpleWorld` with an empty hook registry.
    pub fn new(world: SimpleWorld) -> Self {
        Self {
            world,
            hooks: HookRegistry::new(),
        }
    }

    /// Wrap a `SimpleWorld` with a pre-configured hook registry.
    pub fn with_hooks(world: SimpleWorld, hooks: HookRegistry) -> Self {
        Self { world, hooks }
    }

    /// Access the underlying `SimpleWorld`.
    pub fn world(&self) -> &SimpleWorld {
        &self.world
    }

    /// Mutably access the underlying `SimpleWorld`.
    pub fn world_mut(&mut self) -> &mut SimpleWorld {
        &mut self.world
    }

    /// Access the hook registry.
    pub fn hooks(&self) -> &HookRegistry {
        &self.hooks
    }

    /// Mutably access the hook registry.
    pub fn hooks_mut(&mut self) -> &mut HookRegistry {
        &mut self.hooks
    }

    /// Consume the wrapper and return the inner `SimpleWorld`.
    pub fn into_inner(self) -> SimpleWorld {
        self.world
    }

    /// Spawn a new entity (delegates to `SimpleWorld`).
    pub fn spawn_entity(&mut self) -> EntityId {
        self.world.spawn_entity()
    }

    /// Add a component, firing `on_add` hooks after insertion.
    pub fn add_component(&mut self, entity_id: EntityId, component_type: Symbol, data: Bytes) {
        self.world
            .add_component(entity_id, component_type.clone(), data.clone());
        self.hooks.fire_on_add(entity_id, &component_type, &data);
    }

    /// Remove a component, firing `on_remove` hooks before removal.
    pub fn remove_component(&mut self, entity_id: EntityId, component_type: &Symbol) -> bool {
        self.hooks.fire_on_remove(entity_id, component_type);
        self.world.remove_component(entity_id, component_type)
    }

    /// Get a component (delegates to `SimpleWorld`).
    pub fn get_component(&self, entity_id: EntityId, component_type: &Symbol) -> Option<Bytes> {
        self.world.get_component(entity_id, component_type)
    }

    /// Check if an entity has a component (delegates to `SimpleWorld`).
    pub fn has_component(&self, entity_id: EntityId, component_type: &Symbol) -> bool {
        self.world.has_component(entity_id, component_type)
    }

    /// Despawn an entity, firing `on_remove` hooks for each component.
    pub fn despawn_entity(&mut self, entity_id: EntityId) {
        // Fire on_remove for each component before despawning
        if let Some(types) = self.world.entity_components.get(entity_id) {
            for i in 0..types.len() {
                if let Some(t) = types.get(i) {
                    self.hooks.fire_on_remove(entity_id, &t);
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

    fn noop_add_hook(_entity_id: EntityId, _component_type: &Symbol, _data: &Bytes) {}
    fn noop_remove_hook(_entity_id: EntityId, _component_type: &Symbol) {}

    #[test]
    fn test_hook_registry_new() {
        let registry = HookRegistry::new();
        assert_eq!(registry.add_hook_count(), 0);
        assert_eq!(registry.remove_hook_count(), 0);
    }

    #[test]
    fn test_hook_registry_register() {
        let mut registry = HookRegistry::new();
        registry.on_add(symbol_short!("pos"), noop_add_hook);
        registry.on_remove(symbol_short!("pos"), noop_remove_hook);
        assert_eq!(registry.add_hook_count(), 1);
        assert_eq!(registry.remove_hook_count(), 1);
    }

    #[test]
    fn test_hook_registry_multiple_hooks() {
        let mut registry = HookRegistry::new();
        registry.on_add(symbol_short!("pos"), noop_add_hook);
        registry.on_add(symbol_short!("vel"), noop_add_hook);
        registry.on_remove(symbol_short!("pos"), noop_remove_hook);
        assert_eq!(registry.add_hook_count(), 2);
        assert_eq!(registry.remove_hook_count(), 1);
    }

    #[test]
    fn test_hooked_world_add_component() {
        let env = Env::default();
        let world = SimpleWorld::new(&env);
        let mut hooked = HookedWorld::new(world);
        hooked
            .hooks_mut()
            .on_add(symbol_short!("pos"), noop_add_hook);

        let e1 = hooked.spawn_entity();
        let data = Bytes::from_array(&env, &[1, 2, 3, 4]);
        hooked.add_component(e1, symbol_short!("pos"), data.clone());

        // Verify the component was actually added
        assert!(hooked.has_component(e1, &symbol_short!("pos")));
        assert_eq!(hooked.get_component(e1, &symbol_short!("pos")), Some(data));
    }

    #[test]
    fn test_hooked_world_remove_component() {
        let env = Env::default();
        let world = SimpleWorld::new(&env);
        let mut hooked = HookedWorld::new(world);
        hooked
            .hooks_mut()
            .on_remove(symbol_short!("pos"), noop_remove_hook);

        let e1 = hooked.spawn_entity();
        let data = Bytes::from_array(&env, &[1, 2, 3, 4]);
        hooked.add_component(e1, symbol_short!("pos"), data);

        assert!(hooked.remove_component(e1, &symbol_short!("pos")));
        assert!(!hooked.has_component(e1, &symbol_short!("pos")));
    }

    #[test]
    fn test_hooked_world_despawn() {
        let env = Env::default();
        let world = SimpleWorld::new(&env);
        let mut hooked = HookedWorld::new(world);
        hooked
            .hooks_mut()
            .on_remove(symbol_short!("a"), noop_remove_hook);

        let e1 = hooked.spawn_entity();
        let data = Bytes::from_array(&env, &[1]);
        hooked.add_component(e1, symbol_short!("a"), data.clone());
        hooked.add_component(e1, symbol_short!("b"), data);

        hooked.despawn_entity(e1);

        assert!(!hooked.has_component(e1, &symbol_short!("a")));
        assert!(!hooked.has_component(e1, &symbol_short!("b")));
    }

    #[test]
    fn test_hooked_world_into_inner() {
        let env = Env::default();
        let world = SimpleWorld::new(&env);
        let mut hooked = HookedWorld::new(world);

        let e1 = hooked.spawn_entity();
        let data = Bytes::from_array(&env, &[1]);
        hooked.add_component(e1, symbol_short!("test"), data);

        let inner = hooked.into_inner();
        assert!(inner.has_component(e1, &symbol_short!("test")));
    }

    #[test]
    fn test_hooked_world_with_hooks() {
        let env = Env::default();
        let world = SimpleWorld::new(&env);

        let mut hooks = HookRegistry::new();
        hooks.on_add(symbol_short!("pos"), noop_add_hook);

        let hooked = HookedWorld::with_hooks(world, hooks);
        assert_eq!(hooked.hooks().add_hook_count(), 1);
    }
}
