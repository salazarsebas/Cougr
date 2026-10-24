//! Incremental serialization for per-entity persistent storage.
//!
//! Unlike `SimpleWorld` which stores all ECS state in a single
//! `#[contracttype]` struct, the incremental module splits state
//! into per-entity storage entries. Only dirty (modified) data is
//! written on `flush()`, reducing gas costs for partial updates.
//!
//! # Key layout
//!
//! Every key carries a world namespace (a `Symbol`) so multiple
//! `StorageWorld`s can coexist in one contract:
//!
//! - `("cougr_meta", ns)`: world metadata (`next_entity_id`, `version`, `entity_count`)
//! - `("cougr_eix", ns, bucket)`: bucketed live entity index (up to 64 ids per bucket)
//! - `("cougr_ent", ns, entity)`: per-entity component type list
//! - `("cougr_cmp", ns, entity, type)`: per-component data
//!
//! # TTL behavior
//!
//! Every persistent entry read or written by a world has its TTL extended
//! according to a `TtlConfig` attached to the world. Defaults are conservative
//! and can be replaced via `StorageWorld::with_ttl_config`; a policy with
//! `threshold == 0` and `extend_to == 0` disables extension.
//!
//! # Load / mutate / flush contract
//!
//! 1. `StorageWorld::open(env, namespace)` loads metadata (no entity data).
//! 2. Entities are loaded on demand (`load_entity`) or implicitly by mutation.
//! 3. Mutations update memory and mark entries dirty.
//! 4. `flush(env)` writes dirty entries, deletes removed components and
//!    despawned entities, and persists the index and metadata.
//!
//! Storage entries are only consistent with memory after `flush()`.
//!
//! # Modules
//!
//! - **`keys`**: Storage key construction helpers
//! - **`dirty_tracker`**: Tracks which entries need flushing
//! - **`storage_world`**: The `StorageWorld` implementation
//!
//! # Trade-offs
//!
//! - Cheaper writes (only modified entries are flushed)
//! - Costlier full scans (no single-Map iteration)
//! - Best suited for games where systems operate on known entities

pub mod dirty_tracker;
pub mod keys;
pub mod storage_world;

pub use storage_world::{StorageWorld, TtlConfig, WorldMetadata};
