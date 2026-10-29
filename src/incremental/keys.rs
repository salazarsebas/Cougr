//! Storage key construction helpers for per-entity persistent storage.
//!
//! Storage key layout in Soroban persistent storage:
//! - `("cougr_meta",)` -> `WorldMetadata`
//! - `("cougr_ent", entity_id)` -> `Vec<Symbol>` (component types)
//! - `("cougr_cmp", entity_id, component_type)` -> `Bytes` (component data)

use crate::simple_world::EntityId;
use soroban_sdk::{Env, Symbol};

/// Storage key for world metadata.
pub fn meta_key(env: &Env) -> Symbol {
    Symbol::new(env, "cougr_meta")
}

/// Storage key for an entity's component type list.
pub fn entity_key(env: &Env, entity_id: EntityId) -> (Symbol, EntityId) {
    (Symbol::new(env, "cougr_ent"), entity_id)
}

/// Storage key for a specific component on an entity.
pub fn component_key(
    env: &Env,
    entity_id: EntityId,
    component_type: &Symbol,
) -> (Symbol, EntityId, Symbol) {
    (
        Symbol::new(env, "cougr_cmp"),
        entity_id,
        component_type.clone(),
    )
}
