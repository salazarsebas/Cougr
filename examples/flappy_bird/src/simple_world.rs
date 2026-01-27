use soroban_sdk::{contracttype, Bytes, Env, Symbol, Vec};

/// Simple entity ID
pub type EntityId = u32;

/// Simple component storage entry
#[contracttype]
#[derive(Clone, Debug)]
pub struct ComponentEntry {
    pub entity_id: EntityId,
    pub component_type: Symbol,
    pub data: Bytes,
}

/// Simplified game world for Soroban
#[contracttype]
#[derive(Clone, Debug)]
pub struct SimpleWorld {
    pub next_entity_id: EntityId,
    pub components: Vec<ComponentEntry>,
}

impl SimpleWorld {
    pub fn new(env: &Env) -> Self {
        Self {
            next_entity_id: 1,
            components: Vec::new(env),
        }
    }

    pub fn spawn_entity(&mut self) -> EntityId {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        id
    }

    pub fn add_component(&mut self, entity_id: EntityId, component_type: Symbol, data: Bytes) {
        // Remove existing component if any
        self.remove_component(entity_id, &component_type);

        let entry = ComponentEntry {
            entity_id,
            component_type,
            data,
        };
        self.components.push_back(entry);
    }

    pub fn get_component(&self, entity_id: EntityId, component_type: &Symbol) -> Option<Bytes> {
        for i in 0..self.components.len() {
            let entry = self.components.get(i).unwrap();
            if entry.entity_id == entity_id && &entry.component_type == component_type {
                return Some(entry.data.clone());
            }
        }
        None
    }

    pub fn remove_component(&mut self, entity_id: EntityId, component_type: &Symbol) -> bool {
        let env = &self.components.env();
        let mut new_components = Vec::new(env);
        let mut found = false;

        for i in 0..self.components.len() {
            let entry = self.components.get(i).unwrap();
            if entry.entity_id == entity_id && &entry.component_type == component_type {
                found = true;
            } else {
                new_components.push_back(entry);
            }
        }

        if found {
            self.components = new_components;
        }
        found
    }

    pub fn has_component(&self, entity_id: EntityId, component_type: &Symbol) -> bool {
        for i in 0..self.components.len() {
            let entry = self.components.get(i).unwrap();
            if entry.entity_id == entity_id && &entry.component_type == component_type {
                return true;
            }
        }
        false
    }

    pub fn get_entities_with_component(&self, component_type: &Symbol, env: &Env) -> Vec<EntityId> {
        let mut entities = Vec::new(env);
        for i in 0..self.components.len() {
            let entry = self.components.get(i).unwrap();
            if &entry.component_type == component_type {
                entities.push_back(entry.entity_id);
            }
        }
        entities
    }

    pub fn despawn_entity(&mut self, entity_id: EntityId) {
        let env = &self.components.env();
        let mut new_components = Vec::new(env);

        for i in 0..self.components.len() {
            let entry = self.components.get(i).unwrap();
            if entry.entity_id != entity_id {
                new_components.push_back(entry);
            }
        }

        self.components = new_components;
    }
}
