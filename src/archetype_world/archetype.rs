//! Archetype definition and entity storage.
//!
//! An archetype groups entities with the exact same component composition.
//! This enables efficient iteration: systems that need Position + Velocity
//! only scan archetypes that contain both, skipping unrelated entities.

use crate::simple_world::EntityId;
use soroban_sdk::{contracttype, Bytes, Env, Map, Symbol, Vec};

/// Unique identifier for an archetype.
pub type ArchetypeId = u32;

/// An archetype groups entities with the exact same component set.
///
/// Entities in the same archetype share the same component types, enabling
/// efficient batch iteration. Component data is stored per (entity, type).
#[contracttype]
#[derive(Clone, Debug)]
pub struct Archetype {
    /// Unique archetype identifier.
    pub id: u32,
    /// Sorted list of component types in this archetype.
    pub component_types: Vec<Symbol>,
    /// List of entity IDs in this archetype.
    pub entities: Vec<u32>,
    /// Component data keyed by (entity_id, component_type).
    pub data: Map<(u32, Symbol), Bytes>,
}

impl Archetype {
    /// Create a new empty archetype with the given component types.
    pub fn new(env: &Env, id: ArchetypeId, component_types: Vec<Symbol>) -> Self {
        Self {
            id,
            component_types,
            entities: Vec::new(env),
            data: Map::new(env),
        }
    }

    /// Check if this archetype contains all required component types.
    pub fn matches(&self, required: &[Symbol]) -> bool {
        for req in required {
            if !self.has_component_type(req) {
                return false;
            }
        }
        true
    }

    /// Check if this archetype has a specific component type.
    pub fn has_component_type(&self, component_type: &Symbol) -> bool {
        for i in 0..self.component_types.len() {
            if let Some(t) = self.component_types.get(i) {
                if &t == component_type {
                    return true;
                }
            }
        }
        false
    }

    /// Add an entity to this archetype.
    pub fn add_entity(&mut self, entity_id: EntityId) {
        self.entities.push_back(entity_id);
    }

    /// Remove an entity from this archetype, returning its component data.
    pub fn remove_entity(&mut self, entity_id: EntityId, env: &Env) -> Map<Symbol, Bytes> {
        // Remove from entity list
        let mut new_entities = Vec::new(env);
        for i in 0..self.entities.len() {
            if let Some(eid) = self.entities.get(i) {
                if eid != entity_id {
                    new_entities.push_back(eid);
                }
            }
        }
        self.entities = new_entities;

        // Extract component data
        let mut extracted: Map<Symbol, Bytes> = Map::new(env);
        for i in 0..self.component_types.len() {
            if let Some(ct) = self.component_types.get(i) {
                if let Some(d) = self.data.get((entity_id, ct.clone())) {
                    extracted.set(ct.clone(), d);
                    self.data.remove((entity_id, ct));
                }
            }
        }

        extracted
    }

    /// Get a component value for an entity.
    pub fn get_component(&self, entity_id: EntityId, component_type: &Symbol) -> Option<Bytes> {
        self.data.get((entity_id, component_type.clone()))
    }

    /// Set a component value for an entity.
    pub fn set_component(&mut self, entity_id: EntityId, component_type: Symbol, data: Bytes) {
        self.data.set((entity_id, component_type), data);
    }

    /// Returns the number of entities in this archetype.
    pub fn entity_count(&self) -> u32 {
        self.entities.len()
    }

    /// Check if an entity is in this archetype.
    pub fn contains_entity(&self, entity_id: EntityId) -> bool {
        for i in 0..self.entities.len() {
            if let Some(eid) = self.entities.get(i) {
                if eid == entity_id {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, vec, Env};

    #[test]
    fn test_new_archetype() {
        let env = Env::default();
        let types = vec![&env, symbol_short!("pos"), symbol_short!("vel")];
        let arch = Archetype::new(&env, 0, types);

        assert_eq!(arch.id, 0);
        assert_eq!(arch.entity_count(), 0);
        assert!(arch.has_component_type(&symbol_short!("pos")));
        assert!(arch.has_component_type(&symbol_short!("vel")));
        assert!(!arch.has_component_type(&symbol_short!("hp")));
    }

    #[test]
    fn test_matches() {
        let env = Env::default();
        let types = vec![&env, symbol_short!("pos"), symbol_short!("vel")];
        let arch = Archetype::new(&env, 0, types);

        assert!(arch.matches(&[symbol_short!("pos")]));
        assert!(arch.matches(&[symbol_short!("pos"), symbol_short!("vel")]));
        assert!(!arch.matches(&[symbol_short!("hp")]));
        assert!(!arch.matches(&[symbol_short!("pos"), symbol_short!("hp")]));
    }

    #[test]
    fn test_add_and_get_component() {
        let env = Env::default();
        let types = vec![&env, symbol_short!("pos")];
        let mut arch = Archetype::new(&env, 0, types);

        arch.add_entity(1);
        arch.set_component(1, symbol_short!("pos"), Bytes::from_array(&env, &[10, 20]));

        assert_eq!(arch.entity_count(), 1);
        assert!(arch.contains_entity(1));
        assert_eq!(
            arch.get_component(1, &symbol_short!("pos")),
            Some(Bytes::from_array(&env, &[10, 20]))
        );
    }

    #[test]
    fn test_remove_entity() {
        let env = Env::default();
        let types = vec![&env, symbol_short!("pos"), symbol_short!("vel")];
        let mut arch = Archetype::new(&env, 0, types);

        arch.add_entity(1);
        arch.set_component(1, symbol_short!("pos"), Bytes::from_array(&env, &[1]));
        arch.set_component(1, symbol_short!("vel"), Bytes::from_array(&env, &[2]));

        let extracted = arch.remove_entity(1, &env);
        assert_eq!(arch.entity_count(), 0);
        assert!(!arch.contains_entity(1));
        assert_eq!(extracted.len(), 2);
    }
}
