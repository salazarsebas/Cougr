use crate::simple_world::EntityId as SimpleEntityId;
use soroban_sdk::Vec;

pub(super) fn contains_entity(entities: &Vec<SimpleEntityId>, entity_id: SimpleEntityId) -> bool {
    for i in 0..entities.len() {
        if let Some(candidate) = entities.get(i) {
            if candidate == entity_id {
                return true;
            }
        }
    }
    false
}
