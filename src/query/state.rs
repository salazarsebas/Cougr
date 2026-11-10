use crate::query::SimpleQuery;
use crate::simple_world::EntityId as SimpleEntityId;
use crate::simple_world::SimpleWorld;
use soroban_sdk::{Env, Vec};

/// Version-aware cache for a `SimpleQuery`.
#[derive(Debug, Clone)]
pub struct SimpleQueryState {
    query: SimpleQuery,
    cached_results: Vec<SimpleEntityId>,
    cached_version: u64,
}

impl SimpleQueryState {
    pub fn new(query: SimpleQuery, env: &Env) -> Self {
        Self {
            query,
            cached_results: Vec::new(env),
            cached_version: 0,
        }
    }

    pub fn execute<'a>(&'a mut self, world: &SimpleWorld, env: &Env) -> &'a Vec<SimpleEntityId> {
        if self.cached_version != world.version() {
            self.cached_results = self.query.execute(world, env);
            self.cached_version = world.version();
        }
        &self.cached_results
    }

    pub fn invalidate(&mut self) {
        self.cached_version = 0;
    }

    pub fn is_valid(&self, world_version: u64) -> bool {
        self.cached_version == world_version
    }

    pub fn query(&self) -> &SimpleQuery {
        &self.query
    }
}

/// Cached query for `SimpleWorld` that avoids re-scanning when the world hasn't changed.
///
/// Tracks a single component type and caches matching entity IDs.
/// Automatically invalidates when the world version changes.
///
/// # Example
/// ```
/// use cougr_core::query::SimpleQueryCache;
/// use cougr_core::simple_world::SimpleWorld;
/// use soroban_sdk::{symbol_short, Bytes, Env};
///
/// let env = Env::default();
/// let mut world = SimpleWorld::new(&env);
/// let entity = world.spawn_entity();
/// world.add_component(entity, symbol_short!("position"), Bytes::new(&env));
///
/// let mut cache = SimpleQueryCache::new(symbol_short!("position"), &env);
/// let entities = cache.execute(&world, &env);
/// assert_eq!(entities.len(), 1);
/// let entities2 = cache.execute(&world, &env);
/// assert_eq!(entities2.len(), 1);
/// ```
pub struct SimpleQueryCache {
    state: SimpleQueryState,
}

impl SimpleQueryCache {
    /// Create a new query cache for a specific component type
    pub fn new(component_type: soroban_sdk::Symbol, env: &Env) -> Self {
        Self {
            state: SimpleQueryState::new(SimpleQuery::new(env).with_component(component_type), env),
        }
    }

    /// Create a cache from an explicit `SimpleQuery`.
    pub fn from_query(query: SimpleQuery, env: &Env) -> Self {
        Self {
            state: SimpleQueryState::new(query, env),
        }
    }

    /// Execute the query, returning cached results if the world hasn't changed.
    pub fn execute(&mut self, world: &SimpleWorld, env: &Env) -> &Vec<SimpleEntityId> {
        self.state.execute(world, env)
    }

    /// Force invalidation of the cache.
    pub fn invalidate(&mut self) {
        self.state.invalidate();
    }

    /// Check if the cache is up-to-date with the given world version.
    pub fn is_valid(&self, world_version: u64) -> bool {
        self.state.is_valid(world_version)
    }

    /// Returns the underlying query.
    pub fn query(&self) -> &SimpleQuery {
        self.state.query()
    }
}
