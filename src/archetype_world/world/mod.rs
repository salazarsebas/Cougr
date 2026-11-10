mod helpers;
#[cfg(test)]
mod tests;

use super::archetype::{Archetype, ArchetypeId};
use crate::component::ComponentTrait;
use crate::simple_world::{EntityId, SimpleWorld};
use soroban_sdk::{contracttype, Bytes, Env, Map, Symbol, Vec};

/// World that groups entities by archetype for efficient queries.
///
/// Each unique combination of component types forms an archetype.
/// Entities with the same component set share an archetype, enabling
/// batch iteration without per-entity type checks.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ArchetypeWorld {
    pub(crate) next_entity_id: u32,
    pub(crate) next_archetype_id: u32,
    pub(crate) archetypes: Map<u32, Archetype>,
    pub(crate) archetype_index: Map<Vec<Symbol>, u32>,
    pub(crate) entity_archetype: Map<u32, u32>,
    pub(crate) version: u64,
}

impl ArchetypeWorld {
    pub fn new(env: &Env) -> Self {
        Self {
            next_entity_id: 1,
            next_archetype_id: 0,
            archetypes: Map::new(env),
            archetype_index: Map::new(env),
            entity_archetype: Map::new(env),
            version: 0,
        }
    }

    pub fn spawn_entity(&mut self) -> EntityId {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        id
    }

    pub fn add_component(
        &mut self,
        entity_id: EntityId,
        component_type: Symbol,
        data: Bytes,
        env: &Env,
    ) {
        self.version += 1;

        if let Some(arch_id) = self.entity_archetype.get(entity_id) {
            if let Some(mut arch) = self.archetypes.get(arch_id) {
                if arch.has_component_type(&component_type) {
                    arch.set_component(entity_id, component_type, data);
                    self.archetypes.set(arch_id, arch);
                    return;
                }

                let new_types =
                    helpers::build_new_types(&arch.component_types, &component_type, env);
                let mut extracted = arch.remove_entity(entity_id, env);
                extracted.set(component_type, data);
                self.archetypes.set(arch_id, arch);

                let target_id = self.get_or_create_archetype(new_types, env);
                if let Some(mut target) = self.archetypes.get(target_id) {
                    target.add_entity(entity_id);
                    for key in extracted.keys().iter() {
                        if let Some(d) = extracted.get(key.clone()) {
                            target.set_component(entity_id, key, d);
                        }
                    }
                    self.archetypes.set(target_id, target);
                }
                self.entity_archetype.set(entity_id, target_id);
            }
        } else {
            let types = helpers::canonicalize_single(env, &component_type);
            let arch_id = self.get_or_create_archetype(types, env);
            if let Some(mut arch) = self.archetypes.get(arch_id) {
                arch.add_entity(entity_id);
                arch.set_component(entity_id, component_type, data);
                self.archetypes.set(arch_id, arch);
            }
            self.entity_archetype.set(entity_id, arch_id);
        }
    }

    pub fn remove_component(
        &mut self,
        entity_id: EntityId,
        component_type: &Symbol,
        env: &Env,
    ) -> bool {
        let arch_id = match self.entity_archetype.get(entity_id) {
            Some(id) => id,
            None => return false,
        };

        let mut arch = match self.archetypes.get(arch_id) {
            Some(a) => a,
            None => return false,
        };

        if !arch.has_component_type(component_type) {
            return false;
        }

        self.version += 1;

        let mut new_type_list: alloc::vec::Vec<Symbol> = alloc::vec::Vec::new();
        for i in 0..arch.component_types.len() {
            if let Some(t) = arch.component_types.get(i) {
                if &t != component_type {
                    new_type_list.push(t);
                }
            }
        }

        let mut extracted = arch.remove_entity(entity_id, env);
        extracted.remove(component_type.clone());
        self.archetypes.set(arch_id, arch);

        if new_type_list.is_empty() {
            self.entity_archetype.remove(entity_id);
        } else {
            let new_types = helpers::vec_from_slice(env, &new_type_list);
            let target_id = self.get_or_create_archetype(new_types, env);
            if let Some(mut target) = self.archetypes.get(target_id) {
                target.add_entity(entity_id);
                for key in extracted.keys().iter() {
                    if let Some(d) = extracted.get(key.clone()) {
                        target.set_component(entity_id, key, d);
                    }
                }
                self.archetypes.set(target_id, target);
            }
            self.entity_archetype.set(entity_id, target_id);
        }

        true
    }

    pub fn get_component(&self, entity_id: EntityId, component_type: &Symbol) -> Option<Bytes> {
        let arch_id = self.entity_archetype.get(entity_id)?;
        let arch = self.archetypes.get(arch_id)?;
        arch.get_component(entity_id, component_type)
    }

    pub fn has_component(&self, entity_id: EntityId, component_type: &Symbol) -> bool {
        if let Some(arch_id) = self.entity_archetype.get(entity_id) {
            if let Some(arch) = self.archetypes.get(arch_id) {
                return arch.has_component_type(component_type);
            }
        }
        false
    }

    pub fn despawn_entity(&mut self, entity_id: EntityId, env: &Env) {
        if let Some(arch_id) = self.entity_archetype.get(entity_id) {
            if let Some(mut arch) = self.archetypes.get(arch_id) {
                arch.remove_entity(entity_id, env);
                self.archetypes.set(arch_id, arch);
            }
            self.entity_archetype.remove(entity_id);
        }
        self.version += 1;
    }

    pub fn query(&self, required_components: &[Symbol], env: &Env) -> Vec<EntityId> {
        let mut results = Vec::new(env);

        for key in self.archetypes.keys().iter() {
            if let Some(arch) = self.archetypes.get(key) {
                if arch.matches(required_components) {
                    for i in 0..arch.entities.len() {
                        if let Some(eid) = arch.entities.get(i) {
                            results.push_back(eid);
                        }
                    }
                }
            }
        }

        results
    }

    pub fn get_typed<T: ComponentTrait>(&self, env: &Env, entity_id: EntityId) -> Option<T> {
        let bytes = self.get_component(entity_id, &T::component_type())?;
        T::deserialize(env, &bytes)
    }

    pub fn set_typed<T: ComponentTrait>(&mut self, env: &Env, entity_id: EntityId, component: &T) {
        let symbol = T::component_type();
        let data = component.serialize(env);
        self.add_component(entity_id, symbol, data, env);
    }

    pub fn has_typed<T: ComponentTrait>(&self, entity_id: EntityId) -> bool {
        self.has_component(entity_id, &T::component_type())
    }

    pub fn remove_typed<T: ComponentTrait>(&mut self, env: &Env, entity_id: EntityId) -> bool {
        self.remove_component(entity_id, &T::component_type(), env)
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    /// Returns the next entity ID that will be assigned on spawn.
    pub fn next_entity_id(&self) -> EntityId {
        self.next_entity_id
    }

    pub fn to_simple_world(&self, env: &Env) -> SimpleWorld {
        let mut world = SimpleWorld::new(env);
        world.next_entity_id = self.next_entity_id;

        for arch_key in self.archetypes.keys().iter() {
            if let Some(arch) = self.archetypes.get(arch_key) {
                for i in 0..arch.entities.len() {
                    if let Some(eid) = arch.entities.get(i) {
                        for j in 0..arch.component_types.len() {
                            if let Some(ct) = arch.component_types.get(j) {
                                if let Some(data) = arch.get_component(eid, &ct) {
                                    world.add_component(eid, ct, data);
                                }
                            }
                        }
                    }
                }
            }
        }

        world.version = self.version;
        world
    }

    pub fn from_simple_world(simple: &SimpleWorld, env: &Env) -> Self {
        let mut world = Self::new(env);
        world.next_entity_id = simple.next_entity_id;

        for eid in simple.entity_components.keys().iter() {
            if let Some(types) = simple.entity_components.get(eid) {
                for i in 0..types.len() {
                    if let Some(ct) = types.get(i) {
                        if let Some(data) = simple.get_component(eid, &ct) {
                            world.add_component(eid, ct, data, env);
                        }
                    }
                }
            }
        }

        world.version = simple.version;
        world
    }

    fn get_or_create_archetype(&mut self, component_types: Vec<Symbol>, env: &Env) -> ArchetypeId {
        if let Some(existing) = self.archetype_index.get(component_types.clone()) {
            return existing;
        }

        let id = self.next_archetype_id;
        self.next_archetype_id += 1;

        let arch = Archetype::new(env, id, component_types.clone());
        self.archetypes.set(id, arch);
        self.archetype_index.set(component_types, id);
        id
    }
}
