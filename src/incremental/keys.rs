//! Storage key construction helpers for per-entity persistent storage.
//!
//! Storage key layout in Soroban persistent storage (namespaced by `ns: Symbol`):
//! - `("cougr_meta", ns)` -> `WorldMetadata`
//! - `("cougr_ent", ns, entity_id)` -> `Vec<StoredComponentRef>` (component types + storage kind)
//! - `("cougr_cmp", ns, entity_id, component_type)` -> `Bytes` (component data)
//! - `("cougr_eix", ns, bucket)` -> `Vec<EntityId>` (bucketed live entity index, up to 64 IDs/bucket)
//! - `("cougr_eloc", ns, entity_id)` -> `u32` (flat index position for O(1) swap-remove)

use crate::simple_world::EntityId;
use soroban_sdk::{Env, Symbol};

/// Number of entity IDs stored per `cougr_eix` index bucket.
pub const ENTITY_INDEX_BUCKET_SIZE: u32 = 64;

/// Storage key for world metadata.
pub fn meta_key(env: &Env, namespace: &Symbol) -> (Symbol, Symbol) {
    (Symbol::new(env, "cougr_meta"), namespace.clone())
}

/// Storage key for an entity's component type list.
pub fn entity_key(
    env: &Env,
    namespace: &Symbol,
    entity_id: EntityId,
) -> (Symbol, Symbol, EntityId) {
    (Symbol::new(env, "cougr_ent"), namespace.clone(), entity_id)
}

/// Storage key for a specific component on an entity.
pub fn component_key(
    env: &Env,
    namespace: &Symbol,
    entity_id: EntityId,
    component_type: &Symbol,
) -> (Symbol, Symbol, EntityId, Symbol) {
    (
        Symbol::new(env, "cougr_cmp"),
        namespace.clone(),
        entity_id,
        component_type.clone(),
    )
}

/// Storage key for a bucket of live entity IDs.
pub fn entity_index_bucket_key(
    env: &Env,
    namespace: &Symbol,
    bucket: u32,
) -> (Symbol, Symbol, u32) {
    (Symbol::new(env, "cougr_eix"), namespace.clone(), bucket)
}

/// Storage key for an entity's flat index position within the bucketed index.
pub fn entity_index_loc_key(
    env: &Env,
    namespace: &Symbol,
    entity_id: EntityId,
) -> (Symbol, Symbol, EntityId) {
    (Symbol::new(env, "cougr_eloc"), namespace.clone(), entity_id)
}
