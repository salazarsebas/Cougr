use soroban_sdk::{Env, Symbol, Vec};

pub(super) fn canonicalize_single(env: &Env, component_type: &Symbol) -> Vec<Symbol> {
    let mut v = Vec::new(env);
    v.push_back(component_type.clone());
    v
}

pub(super) fn vec_from_slice(env: &Env, items: &[Symbol]) -> Vec<Symbol> {
    let mut v = Vec::new(env);
    for item in items {
        v.push_back(item.clone());
    }
    v
}

pub(super) fn build_new_types(existing: &Vec<Symbol>, new_type: &Symbol, env: &Env) -> Vec<Symbol> {
    let mut types: alloc::vec::Vec<Symbol> = alloc::vec::Vec::new();
    for i in 0..existing.len() {
        if let Some(t) = existing.get(i) {
            types.push(t);
        }
    }
    types.push(new_type.clone());
    types.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    vec_from_slice(env, &types)
}
