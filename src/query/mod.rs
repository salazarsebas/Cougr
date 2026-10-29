mod helpers;
mod state;
#[cfg(test)]
mod tests;

pub use state::{SimpleQueryCache, SimpleQueryState};

use crate::simple_world::EntityId as SimpleEntityId;
use crate::simple_world::SimpleWorld;
use soroban_sdk::{Env, Symbol, Vec};

/// Which backing storage a `SimpleQuery` should consider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryStorage {
    /// Only scan table-backed components, optimized for gameplay loops.
    Table,
    /// Include both table-backed and sparse components.
    Any,
}

/// Query builder and executable query for `SimpleWorld`.
///
/// `SimpleQuery` is the preferred query surface for Soroban gameplay loops:
/// it supports required, excluded, and optional-match components while
/// selecting the narrowest indexed candidate set available.
#[derive(Debug, Clone)]
pub struct SimpleQuery {
    required_components: Vec<Symbol>,
    excluded_components: Vec<Symbol>,
    any_components: Vec<Symbol>,
    storage: QueryStorage,
}

impl SimpleQuery {
    /// Create an empty query over table-backed components.
    pub fn new(env: &Env) -> Self {
        Self {
            required_components: Vec::new(env),
            excluded_components: Vec::new(env),
            any_components: Vec::new(env),
            storage: QueryStorage::Table,
        }
    }

    /// Require the entity to have this component.
    pub fn with_component(mut self, component_type: Symbol) -> Self {
        self.required_components.push_back(component_type);
        self
    }

    /// Require the entity to have all components from this slice.
    pub fn with_components(mut self, component_types: &[Symbol]) -> Self {
        for component_type in component_types {
            self.required_components.push_back(component_type.clone());
        }
        self
    }

    /// Exclude entities with this component.
    pub fn without_component(mut self, component_type: Symbol) -> Self {
        self.excluded_components.push_back(component_type);
        self
    }

    /// Exclude entities that have any component from this slice.
    pub fn without_components(mut self, component_types: &[Symbol]) -> Self {
        for component_type in component_types {
            self.excluded_components.push_back(component_type.clone());
        }
        self
    }

    /// Require the entity to match at least one component from this set.
    pub fn with_any_component(mut self, component_type: Symbol) -> Self {
        self.any_components.push_back(component_type);
        self
    }

    /// Require the entity to match at least one component from this slice.
    pub fn with_any_components(mut self, component_types: &[Symbol]) -> Self {
        for component_type in component_types {
            self.any_components.push_back(component_type.clone());
        }
        self
    }

    /// Include both table and sparse storage during execution.
    pub fn include_sparse(mut self) -> Self {
        self.storage = QueryStorage::Any;
        self
    }

    /// Returns whether the query has no constraints.
    pub fn is_empty(&self) -> bool {
        self.required_components.is_empty()
            && self.excluded_components.is_empty()
            && self.any_components.is_empty()
    }

    /// Returns the current storage mode for the query.
    pub fn storage(&self) -> QueryStorage {
        self.storage
    }

    /// Execute the query against a `SimpleWorld`.
    pub fn execute(&self, world: &SimpleWorld, env: &Env) -> Vec<SimpleEntityId> {
        let candidates = self.candidate_entities(world, env);
        let mut results = Vec::new(env);

        for i in 0..candidates.len() {
            if let Some(entity_id) = candidates.get(i) {
                if self.matches(world, entity_id) {
                    results.push_back(entity_id);
                }
            }
        }

        results
    }

    /// Returns the number of entities that must be scanned before final filtering.
    pub fn candidate_count(&self, world: &SimpleWorld, env: &Env) -> usize {
        self.candidate_entities(world, env)
            .len()
            .try_into()
            .unwrap()
    }

    fn candidate_entities(&self, world: &SimpleWorld, env: &Env) -> Vec<SimpleEntityId> {
        let mut best: Option<Vec<SimpleEntityId>> = None;

        for i in 0..self.required_components.len() {
            if let Some(component_type) = self.required_components.get(i) {
                let entities = self.entities_for_component(world, &component_type, env);
                if best
                    .as_ref()
                    .map(|current| entities.len() < current.len())
                    .unwrap_or(true)
                {
                    best = Some(entities);
                }
            }
        }

        if let Some(candidates) = best {
            return candidates;
        }

        if !self.any_components.is_empty() {
            let mut union = Vec::new(env);
            for i in 0..self.any_components.len() {
                if let Some(component_type) = self.any_components.get(i) {
                    let entities = self.entities_for_component(world, &component_type, env);
                    for j in 0..entities.len() {
                        if let Some(entity_id) = entities.get(j) {
                            if !helpers::contains_entity(&union, entity_id) {
                                union.push_back(entity_id);
                            }
                        }
                    }
                }
            }
            return union;
        }

        let mut all_entities = Vec::new(env);
        for entity_id in world.entity_components.keys().iter() {
            all_entities.push_back(entity_id);
        }
        all_entities
    }

    fn entities_for_component(
        &self,
        world: &SimpleWorld,
        component_type: &Symbol,
        env: &Env,
    ) -> Vec<SimpleEntityId> {
        match self.storage {
            QueryStorage::Table => world.get_table_entities_with_component(component_type, env),
            QueryStorage::Any => world.get_all_entities_with_component(component_type, env),
        }
    }

    fn matches(&self, world: &SimpleWorld, entity_id: SimpleEntityId) -> bool {
        for i in 0..self.required_components.len() {
            if let Some(component_type) = self.required_components.get(i) {
                if !world.has_component(entity_id, &component_type) {
                    return false;
                }
            }
        }

        for i in 0..self.excluded_components.len() {
            if let Some(component_type) = self.excluded_components.get(i) {
                if world.has_component(entity_id, &component_type) {
                    return false;
                }
            }
        }

        if self.any_components.is_empty() {
            return true;
        }

        for i in 0..self.any_components.len() {
            if let Some(component_type) = self.any_components.get(i) {
                if world.has_component(entity_id, &component_type) {
                    return true;
                }
            }
        }

        false
    }
}

/// Builder for `SimpleQuery`.
pub struct SimpleQueryBuilder {
    query: SimpleQuery,
}

impl SimpleQueryBuilder {
    pub fn new(env: &Env) -> Self {
        Self {
            query: SimpleQuery::new(env),
        }
    }

    pub fn with_component(mut self, component_type: Symbol) -> Self {
        self.query = self.query.with_component(component_type);
        self
    }

    pub fn with_components(mut self, component_types: &[Symbol]) -> Self {
        self.query = self.query.with_components(component_types);
        self
    }

    pub fn without_component(mut self, component_type: Symbol) -> Self {
        self.query = self.query.without_component(component_type);
        self
    }

    pub fn without_components(mut self, component_types: &[Symbol]) -> Self {
        self.query = self.query.without_components(component_types);
        self
    }

    pub fn with_any_component(mut self, component_type: Symbol) -> Self {
        self.query = self.query.with_any_component(component_type);
        self
    }

    pub fn with_any_components(mut self, component_types: &[Symbol]) -> Self {
        self.query = self.query.with_any_components(component_types);
        self
    }

    pub fn include_sparse(mut self) -> Self {
        self.query = self.query.include_sparse();
        self
    }

    pub fn build(self) -> SimpleQuery {
        self.query
    }

    pub fn build_state(self, env: &Env) -> SimpleQueryState {
        SimpleQueryState::new(self.query, env)
    }
}
