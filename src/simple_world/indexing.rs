use crate::simple_world::EntityId;
use soroban_sdk::{Map, Symbol, Vec};

pub(super) fn index_contains(index: &Vec<u32>, entity_id: EntityId) -> bool {
    for i in 0..index.len() {
        if let Some(candidate) = index.get(i) {
            if candidate == entity_id {
                return true;
            }
        }
    }
    false
}

pub(super) fn push_index(
    index: &mut Map<Symbol, Vec<u32>>,
    component_type: &Symbol,
    entity_id: EntityId,
) {
    let env = index.env();
    let mut entities = index
        .get(component_type.clone())
        .unwrap_or_else(|| Vec::new(env));
    if !index_contains(&entities, entity_id) {
        entities.push_back(entity_id);
        index.set(component_type.clone(), entities);
    }
}

pub(super) fn remove_from_index(
    index: &mut Map<Symbol, Vec<u32>>,
    component_type: &Symbol,
    entity_id: EntityId,
) {
    if let Some(entities) = index.get(component_type.clone()) {
        let env = index.env();
        let mut filtered = Vec::new(env);
        for i in 0..entities.len() {
            if let Some(candidate) = entities.get(i) {
                if candidate != entity_id {
                    filtered.push_back(candidate);
                }
            }
        }

        if filtered.is_empty() {
            index.remove(component_type.clone());
        } else {
            index.set(component_type.clone(), filtered);
        }
    }
}
