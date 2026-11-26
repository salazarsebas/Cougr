//! Incremental serialization for per-entity persistent storage.
//!
//! Unlike `SimpleWorld` which stores all ECS state in a single
//! `#[contracttype]` struct, the incremental module splits state
//! into per-entity storage entries. Only dirty (modified) data is
//! written on `flush()`, reducing gas costs for partial updates.
//!
//! # Key Layout
//!
//! All keys are namespaced by `ns: Symbol` so multiple `StorageWorld`
//! instances can coexist in a single contract without colliding:
//!
//! - `("cougr_meta", ns)` -> [`WorldMetadata`] (`next_entity_id`, `version`, `entity_count`)
//! - `("cougr_ent", ns, entity_id)` -> `Vec<StoredComponentRef>` (component types and storage kind)
//! - `("cougr_cmp", ns, entity_id, component_type)` -> `Bytes` (component payload)
//! - `("cougr_eix", ns, bucket)` -> `Vec<EntityId>` (bucketed live entity index, 64 IDs/bucket)
//! - `("cougr_eloc", ns, entity_id)` -> `u32` (flat index position for O(1) swap-remove)
//!
//! # TTL Behavior
//!
//! Every persistent entry read or written by [`StorageWorld`] has its TTL
//! extended according to [`TtlPolicy`] (defaults: `threshold = 1_000`,
//! `extend_to = 10_000` ledgers). Pass [`TtlPolicy::disabled()`] to
//! [`StorageWorld::open_with_ttl`] (or call `with_ttl_policy`) to opt out.
//!
//! # Load / Mutate / Flush Contract
//!
//! 1. Call [`StorageWorld::open`] (or [`StorageWorld::load_metadata`] for `"default"`).
//! 2. Load the entities you need with [`StorageWorld::load_entity`] /
//!    [`StorageWorld::load_entities`], or mutate via `spawn_entity`,
//!    `add_component` (which auto-loads unloaded existing entities and returns
//!    `CougrError::EntityNotFound` for unknown/despawned IDs), `remove_component`,
//!    or `despawn_entity`.
//! 3. Optionally bridge to systems with [`StorageWorld::to_simple_world`] and
//!    write back diffs via [`StorageWorld::apply_simple_world`]. Because
//!    `SimpleWorld` drops an entity from `entity_components` when its last
//!    component is removed, `apply_simple_world` despawns any loaded entity
//!    absent from the view's `entity_components` (even though `to_simple_world`
//!    preserves an empty-component entry for entities that started with none).
//! 4. Call [`StorageWorld::flush`] to persist dirty writes and delete removed
//!    components and despawned entities.
//!
//! # Modules
//!
//! - **`keys`**: Storage key construction helpers
//! - **`dirty_tracker`**: Tracks which entries need flushing
//! - **`storage_world`**: The `StorageWorld` implementation

pub mod dirty_tracker;
pub mod keys;
pub mod storage_world;

pub use storage_world::{StorageWorld, TtlPolicy, WorldMetadata};
