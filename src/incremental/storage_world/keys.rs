use soroban_sdk::{Env, Symbol};

/// Default namespace used by `StorageWorld::load_metadata`.
pub(super) const DEFAULT_NAMESPACE: &str = "default";

/// Build a namespaced entity-index bucket key.
pub(super) fn entity_bucket_key(env: &Env, namespace: &Symbol, bucket: u32) -> (Symbol, Symbol, u32) {
    (Symbol::new(env, "cougr_eix"), namespace.clone(), bucket)
}

/// Build a namespaced metadata key.
pub(super) fn metadata_key(env: &Env, namespace: &Symbol) -> (Symbol, Symbol) {
    (Symbol::new(env, "cougr_meta"), namespace.clone())
}

/// Build a namespaced entity-list key for one entity.
pub(super) fn entity_components_key(
    env: &Env,
    namespace: &Symbol,
    entity_id: u32,
) -> (Symbol, Symbol, u32) {
    (Symbol::new(env, "cougr_ent"), namespace.clone(), entity_id)
}

/// Build a namespaced component data key.
pub(super) fn component_key(
    env: &Env,
    namespace: &Symbol,
    entity_id: u32,
    component_type: &Symbol,
) -> (Symbol, Symbol, u32, Symbol) {
    (
        Symbol::new(env, "cougr_cmp"),
        namespace.clone(),
        entity_id,
        component_type.clone(),
    )
}
