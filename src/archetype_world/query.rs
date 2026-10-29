//! Archetype-aware query utilities.
//!
//! Provides cached archetype queries with version-based invalidation,
//! and queries with both required and excluded component types.

use super::world::ArchetypeWorld;
use crate::simple_world::EntityId;
use soroban_sdk::{Env, Symbol, Vec};

/// Cached archetype query with version-based invalidation.
///
/// Caches query results and re-executes only when the world version changes.
/// Follows the same pattern as `SimpleQueryCache` from `src/query.rs`.
pub struct ArchetypeQueryCache {
    query: ArchetypeQuery,
    cached_version: u64,
    cached_results: Option<Vec<EntityId>>,
}

/// Query builder and executable query for `ArchetypeWorld`.
#[derive(Debug, Clone)]
pub struct ArchetypeQuery {
    required_components: alloc::vec::Vec<Symbol>,
    excluded_components: alloc::vec::Vec<Symbol>,
    any_components: alloc::vec::Vec<Symbol>,
}

impl ArchetypeQuery {
    pub fn new() -> Self {
        Self {
            required_components: alloc::vec::Vec::new(),
            excluded_components: alloc::vec::Vec::new(),
            any_components: alloc::vec::Vec::new(),
        }
    }

    pub fn with_component(mut self, component_type: Symbol) -> Self {
        self.required_components.push(component_type);
        self
    }

    pub fn with_components(mut self, component_types: &[Symbol]) -> Self {
        for component_type in component_types {
            self.required_components.push(component_type.clone());
        }
        self
    }

    pub fn without_component(mut self, component_type: Symbol) -> Self {
        self.excluded_components.push(component_type);
        self
    }

    pub fn without_components(mut self, component_types: &[Symbol]) -> Self {
        for component_type in component_types {
            self.excluded_components.push(component_type.clone());
        }
        self
    }

    pub fn with_any_component(mut self, component_type: Symbol) -> Self {
        self.any_components.push(component_type);
        self
    }

    pub fn with_any_components(mut self, component_types: &[Symbol]) -> Self {
        for component_type in component_types {
            self.any_components.push(component_type.clone());
        }
        self
    }

    pub fn execute(&self, world: &ArchetypeWorld, env: &Env) -> Vec<EntityId> {
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

    pub fn candidate_count(&self, world: &ArchetypeWorld, env: &Env) -> usize {
        self.candidate_entities(world, env)
            .len()
            .try_into()
            .unwrap()
    }

    fn candidate_entities(&self, world: &ArchetypeWorld, env: &Env) -> Vec<EntityId> {
        if !self.required_components.is_empty() {
            return world.query(&self.required_components, env);
        }

        if !self.any_components.is_empty() {
            let mut union = Vec::new(env);
            for component_type in &self.any_components {
                let results = world.query(core::slice::from_ref(component_type), env);
                for i in 0..results.len() {
                    if let Some(entity_id) = results.get(i) {
                        if !contains_entity(&union, entity_id) {
                            union.push_back(entity_id);
                        }
                    }
                }
            }
            return union;
        }

        let mut all_entities = Vec::new(env);
        for entity_id in world.entity_archetype.keys().iter() {
            all_entities.push_back(entity_id);
        }
        all_entities
    }

    fn matches(&self, world: &ArchetypeWorld, entity_id: EntityId) -> bool {
        for component_type in &self.required_components {
            if !world.has_component(entity_id, component_type) {
                return false;
            }
        }

        for component_type in &self.excluded_components {
            if world.has_component(entity_id, component_type) {
                return false;
            }
        }

        if self.any_components.is_empty() {
            return true;
        }

        for component_type in &self.any_components {
            if world.has_component(entity_id, component_type) {
                return true;
            }
        }

        false
    }
}

impl Default for ArchetypeQuery {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ArchetypeQueryBuilder {
    query: ArchetypeQuery,
}

impl ArchetypeQueryBuilder {
    pub fn new() -> Self {
        Self {
            query: ArchetypeQuery::new(),
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

    pub fn build(self) -> ArchetypeQuery {
        self.query
    }
}

impl Default for ArchetypeQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ArchetypeQueryState {
    query: ArchetypeQuery,
    cached_version: u64,
    cached_results: Option<Vec<EntityId>>,
}

impl ArchetypeQueryState {
    pub fn new(query: ArchetypeQuery) -> Self {
        Self {
            query,
            cached_version: u64::MAX,
            cached_results: None,
        }
    }

    pub fn execute(&mut self, world: &ArchetypeWorld, env: &Env) -> Vec<EntityId> {
        if self.cached_version == world.version() {
            if let Some(results) = &self.cached_results {
                return results.clone();
            }
        }

        let results = self.query.execute(world, env);
        self.cached_version = world.version();
        self.cached_results = Some(results.clone());
        results
    }

    pub fn invalidate(&mut self) {
        self.cached_version = u64::MAX;
        self.cached_results = None;
    }
}

impl ArchetypeQueryCache {
    /// Create a new cache for the given required components.
    pub fn new(required: alloc::vec::Vec<Symbol>) -> Self {
        Self {
            query: ArchetypeQuery::new().with_components(&required),
            cached_version: u64::MAX,
            cached_results: None,
        }
    }

    pub fn from_query(query: ArchetypeQuery) -> Self {
        Self {
            query,
            cached_version: u64::MAX,
            cached_results: None,
        }
    }

    /// Execute the query, using cached results if the world hasn't changed.
    pub fn execute(&mut self, world: &ArchetypeWorld, env: &Env) -> Vec<EntityId> {
        if self.cached_version == world.version() {
            if let Some(ref results) = self.cached_results {
                return results.clone();
            }
        }

        let results = self.query.execute(world, env);
        self.cached_version = world.version();
        self.cached_results = Some(results.clone());
        results
    }

    /// Force cache invalidation.
    pub fn invalidate(&mut self) {
        self.cached_version = u64::MAX;
        self.cached_results = None;
    }

    pub fn query(&self) -> &ArchetypeQuery {
        &self.query
    }
}

fn contains_entity(entities: &Vec<EntityId>, entity_id: EntityId) -> bool {
    for i in 0..entities.len() {
        if let Some(candidate) = entities.get(i) {
            if candidate == entity_id {
                return true;
            }
        }
    }
    false
}

/// Query with both required and excluded component types.
///
/// Returns entities that have ALL required components and NONE of the
/// excluded components.
pub fn archetype_query(
    world: &ArchetypeWorld,
    required: &[Symbol],
    excluded: &[Symbol],
    env: &Env,
) -> Vec<EntityId> {
    let candidates = world.query(required, env);

    if excluded.is_empty() {
        return candidates;
    }

    let mut results = Vec::new(env);
    for i in 0..candidates.len() {
        if let Some(eid) = candidates.get(i) {
            let mut exclude = false;
            for ex in excluded {
                if world.has_component(eid, ex) {
                    exclude = true;
                    break;
                }
            }
            if !exclude {
                results.push_back(eid);
            }
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, Bytes, Env};

    #[test]
    fn test_archetype_query_cache() {
        let env = Env::default();
        let mut world = ArchetypeWorld::new(&env);

        let e1 = world.spawn_entity();
        world.add_component(
            e1,
            symbol_short!("pos"),
            Bytes::from_array(&env, &[1]),
            &env,
        );

        let mut cache = ArchetypeQueryCache::new(alloc::vec![symbol_short!("pos")]);
        let results = cache.execute(&world, &env);
        assert_eq!(results.len(), 1);

        // Cache hit (same version)
        let results2 = cache.execute(&world, &env);
        assert_eq!(results2.len(), 1);

        // Add another entity, cache miss
        let e2 = world.spawn_entity();
        world.add_component(
            e2,
            symbol_short!("pos"),
            Bytes::from_array(&env, &[2]),
            &env,
        );
        let results3 = cache.execute(&world, &env);
        assert_eq!(results3.len(), 2);
    }

    #[test]
    fn test_archetype_query_cache_invalidate() {
        let env = Env::default();
        let mut world = ArchetypeWorld::new(&env);

        let e1 = world.spawn_entity();
        world.add_component(
            e1,
            symbol_short!("pos"),
            Bytes::from_array(&env, &[1]),
            &env,
        );

        let mut cache = ArchetypeQueryCache::new(alloc::vec![symbol_short!("pos")]);
        cache.execute(&world, &env);
        cache.invalidate();

        // After invalidation, should re-execute
        let results = cache.execute(&world, &env);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_archetype_query_with_exclusions() {
        let env = Env::default();
        let mut world = ArchetypeWorld::new(&env);

        let e1 = world.spawn_entity();
        let e2 = world.spawn_entity();

        // e1: pos
        world.add_component(
            e1,
            symbol_short!("pos"),
            Bytes::from_array(&env, &[1]),
            &env,
        );
        // e2: pos + dead
        world.add_component(
            e2,
            symbol_short!("pos"),
            Bytes::from_array(&env, &[2]),
            &env,
        );
        world.add_component(
            e2,
            symbol_short!("dead"),
            Bytes::from_array(&env, &[1]),
            &env,
        );

        // Query: has pos, not dead
        let results = archetype_query(
            &world,
            &[symbol_short!("pos")],
            &[symbol_short!("dead")],
            &env,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results.get(0), Some(e1));
    }

    #[test]
    fn test_archetype_query_no_exclusions() {
        let env = Env::default();
        let mut world = ArchetypeWorld::new(&env);

        let e1 = world.spawn_entity();
        world.add_component(
            e1,
            symbol_short!("pos"),
            Bytes::from_array(&env, &[1]),
            &env,
        );

        let results = archetype_query(&world, &[symbol_short!("pos")], &[], &env);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_archetype_query_builder_and_state() {
        let env = Env::default();
        let mut world = ArchetypeWorld::new(&env);

        let e1 = world.spawn_entity();
        world.add_component(
            e1,
            symbol_short!("pos"),
            Bytes::from_array(&env, &[1]),
            &env,
        );

        let query = ArchetypeQueryBuilder::new()
            .with_component(symbol_short!("pos"))
            .build();
        let mut state = ArchetypeQueryState::new(query);

        let first = state.execute(&world, &env);
        assert_eq!(first.len(), 1);

        let e2 = world.spawn_entity();
        world.add_component(
            e2,
            symbol_short!("pos"),
            Bytes::from_array(&env, &[2]),
            &env,
        );
        let second = state.execute(&world, &env);
        assert_eq!(second.len(), 2);
    }

    #[test]
    fn test_archetype_query_supports_any_filters_and_cache_from_query() {
        let env = Env::default();
        let mut world = ArchetypeWorld::new(&env);

        let e1 = world.spawn_entity();
        world.add_component(
            e1,
            symbol_short!("pos"),
            Bytes::from_array(&env, &[1]),
            &env,
        );
        world.add_component(
            e1,
            symbol_short!("vel"),
            Bytes::from_array(&env, &[2]),
            &env,
        );

        let e2 = world.spawn_entity();
        world.add_component(
            e2,
            symbol_short!("pos"),
            Bytes::from_array(&env, &[3]),
            &env,
        );
        world.add_component(
            e2,
            symbol_short!("sleep"),
            Bytes::from_array(&env, &[4]),
            &env,
        );

        let query = ArchetypeQueryBuilder::new()
            .with_components(&[symbol_short!("pos")])
            .without_components(&[symbol_short!("sleep")])
            .with_any_components(&[symbol_short!("vel"), symbol_short!("sleep")])
            .build();

        assert_eq!(query.candidate_count(&world, &env), 2);

        let mut cache = ArchetypeQueryCache::from_query(query);
        let results = cache.execute(&world, &env);
        assert_eq!(results.len(), 1);
        assert_eq!(results.get(0), Some(e1));
    }
}
