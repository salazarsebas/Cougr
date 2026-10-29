//! Archetype-based ECS world implementation.
//!
//! Groups entities by component composition for efficient queries.
//! An alternative to `SimpleWorld` that trades cheaper queries for
//! costlier add/remove operations (archetype migration).
//!
//! # Modules
//!
//! - **`archetype`**: Archetype definition and entity storage
//! - **`world`**: The `ArchetypeWorld` implementation
//! - **`query`**: Archetype-aware queries with caching and exclusions

pub mod archetype;
pub mod query;
pub mod world;

pub use archetype::{Archetype, ArchetypeId};
pub use query::{
    archetype_query, ArchetypeQuery, ArchetypeQueryBuilder, ArchetypeQueryCache,
    ArchetypeQueryState,
};
pub use world::ArchetypeWorld;
