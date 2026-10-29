//! Curated ECS runtime surface.
//!
//! This module defines the stable conceptual model for Cougr's Soroban-first ECS path:
//!
//! - `SimpleWorld` and `ArchetypeWorld` are the supported runtime backends
//! - `GameApp` is the recommended orchestration entrypoint
//! - `SimpleQuery` is the default query model for Soroban gameplay loops
//! - `CommandQueue` is the deferred structural mutation primitive

use crate::archetype_world::ArchetypeWorld;
use crate::component::ComponentTrait;
use crate::query::QueryStorage;
use crate::simple_world::{EntityId, SimpleWorld};
use soroban_sdk::{Env, Symbol, Vec};

/// Supported runtime backends for Cougr's ECS layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldBackend {
    Simple,
    Archetype,
}

/// Shared read-only runtime contract for Cougr world backends.
///
/// This trait does not attempt to erase every difference between `SimpleWorld`
/// and `ArchetypeWorld`. It captures the stable overlap that gameplay systems
/// and tooling can rely on without committing to internal storage details.
pub trait RuntimeWorld {
    fn backend(&self) -> WorldBackend;
    fn entity_count(&self) -> usize;
    fn version(&self) -> u64;
    fn has_component(&self, entity_id: EntityId, component_type: &Symbol) -> bool;
    fn entities_with_component(
        &self,
        component_type: &Symbol,
        storage: QueryStorage,
        env: &Env,
    ) -> Vec<EntityId>;
}

/// Shared mutable runtime contract for Cougr's Soroban-first world backends.
///
/// This trait represents the stable gameplay mutation surface common to
/// `SimpleWorld` and `ArchetypeWorld`. It is intentionally narrower than the
/// full implementation of either backend.
pub trait RuntimeWorldMut: RuntimeWorld {
    fn spawn_entity(&mut self) -> EntityId;
    fn despawn_entity(&mut self, entity_id: EntityId, env: &Env);
    fn get_component(
        &self,
        entity_id: EntityId,
        component_type: &Symbol,
    ) -> Option<soroban_sdk::Bytes>;
    fn add_component(
        &mut self,
        entity_id: EntityId,
        component_type: Symbol,
        data: soroban_sdk::Bytes,
        env: &Env,
    );
    fn remove_component(&mut self, entity_id: EntityId, component_type: &Symbol, env: &Env)
        -> bool;

    fn get_typed<T: ComponentTrait>(&self, env: &Env, entity_id: EntityId) -> Option<T> {
        let bytes = self.get_component(entity_id, &T::component_type())?;
        T::deserialize(env, &bytes)
    }

    fn set_typed<T: ComponentTrait>(&mut self, env: &Env, entity_id: EntityId, component: &T) {
        self.add_component(
            entity_id,
            T::component_type(),
            component.serialize(env),
            env,
        );
    }

    fn has_typed<T: ComponentTrait>(&self, entity_id: EntityId) -> bool {
        self.has_component(entity_id, &T::component_type())
    }

    fn remove_typed<T: ComponentTrait>(&mut self, env: &Env, entity_id: EntityId) -> bool {
        self.remove_component(entity_id, &T::component_type(), env)
    }
}

impl RuntimeWorld for SimpleWorld {
    fn backend(&self) -> WorldBackend {
        WorldBackend::Simple
    }

    fn entity_count(&self) -> usize {
        self.entity_components.len().try_into().unwrap()
    }

    fn version(&self) -> u64 {
        self.version()
    }

    fn has_component(&self, entity_id: EntityId, component_type: &Symbol) -> bool {
        self.has_component(entity_id, component_type)
    }

    fn entities_with_component(
        &self,
        component_type: &Symbol,
        storage: QueryStorage,
        env: &Env,
    ) -> Vec<EntityId> {
        match storage {
            QueryStorage::Table => self.get_table_entities_with_component(component_type, env),
            QueryStorage::Any => self.get_all_entities_with_component(component_type, env),
        }
    }
}

impl RuntimeWorldMut for SimpleWorld {
    fn spawn_entity(&mut self) -> EntityId {
        SimpleWorld::spawn_entity(self)
    }

    fn despawn_entity(&mut self, entity_id: EntityId, _env: &Env) {
        SimpleWorld::despawn_entity(self, entity_id);
    }

    fn get_component(
        &self,
        entity_id: EntityId,
        component_type: &Symbol,
    ) -> Option<soroban_sdk::Bytes> {
        SimpleWorld::get_component(self, entity_id, component_type)
    }

    fn add_component(
        &mut self,
        entity_id: EntityId,
        component_type: Symbol,
        data: soroban_sdk::Bytes,
        _env: &Env,
    ) {
        SimpleWorld::add_component(self, entity_id, component_type, data);
    }

    fn remove_component(
        &mut self,
        entity_id: EntityId,
        component_type: &Symbol,
        _env: &Env,
    ) -> bool {
        SimpleWorld::remove_component(self, entity_id, component_type)
    }
}

impl RuntimeWorld for ArchetypeWorld {
    fn backend(&self) -> WorldBackend {
        WorldBackend::Archetype
    }

    fn entity_count(&self) -> usize {
        self.entity_archetype.len().try_into().unwrap()
    }

    fn version(&self) -> u64 {
        self.version()
    }

    fn has_component(&self, entity_id: EntityId, component_type: &Symbol) -> bool {
        self.has_component(entity_id, component_type)
    }

    fn entities_with_component(
        &self,
        component_type: &Symbol,
        _storage: QueryStorage,
        env: &Env,
    ) -> Vec<EntityId> {
        self.query(core::slice::from_ref(component_type), env)
    }
}

impl RuntimeWorldMut for ArchetypeWorld {
    fn spawn_entity(&mut self) -> EntityId {
        ArchetypeWorld::spawn_entity(self)
    }

    fn despawn_entity(&mut self, entity_id: EntityId, env: &Env) {
        ArchetypeWorld::despawn_entity(self, entity_id, env);
    }

    fn get_component(
        &self,
        entity_id: EntityId,
        component_type: &Symbol,
    ) -> Option<soroban_sdk::Bytes> {
        ArchetypeWorld::get_component(self, entity_id, component_type)
    }

    fn add_component(
        &mut self,
        entity_id: EntityId,
        component_type: Symbol,
        data: soroban_sdk::Bytes,
        env: &Env,
    ) {
        ArchetypeWorld::add_component(self, entity_id, component_type, data, env);
    }

    fn remove_component(
        &mut self,
        entity_id: EntityId,
        component_type: &Symbol,
        env: &Env,
    ) -> bool {
        ArchetypeWorld::remove_component(self, entity_id, component_type, env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Position;

    fn exercise_runtime_mut<W: RuntimeWorldMut>(world: &mut W, env: &Env) {
        let entity = world.spawn_entity();
        world.set_typed(env, entity, &Position::new(4, 5));
        assert!(world.has_typed::<Position>(entity));
        let pos: Position = world.get_typed(env, entity).unwrap();
        assert_eq!(pos.x, 4);
        assert!(world.remove_typed::<Position>(env, entity));
        world.despawn_entity(entity, env);
    }

    #[test]
    fn runtime_world_mut_supports_simple_world() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        exercise_runtime_mut(&mut world, &env);
        assert_eq!(world.backend(), WorldBackend::Simple);
    }

    #[test]
    fn runtime_world_mut_supports_archetype_world() {
        let env = Env::default();
        let mut world = ArchetypeWorld::new(&env);
        exercise_runtime_mut(&mut world, &env);
        assert_eq!(world.backend(), WorldBackend::Archetype);
    }
}
