use soroban_sdk::{contracttype, Bytes, Env, Symbol, Vec};
use cougr_core::{World, EntityId, Component, Resource, Storage};

/// Serialized representation of World components for Soroban storage
/// We store each component separately since Storage struct isn't directly serializable
#[contracttype]
#[derive(Clone, Debug)]
pub struct SerializedComponent {
    pub entity_id: u64,
    pub entity_gen: u32,
    pub component_type: Symbol,
    pub component_data: Bytes,
}

/// Helper to serialize World storage components
pub fn serialize_world_components(world: &World, env: &Env) -> Vec<SerializedComponent> {
    let mut serialized = Vec::new(env);

    // Iterate through all entities and their components
    for entity in world.entities.iter_entities() {
        let entity_id = entity.id();
        for component_type in entity.component_types().iter() {
            if let Some(component) = world.storage.get_component(entity_id, component_type.clone()) {
                let ser_comp = SerializedComponent {
                    entity_id: entity_id.id(),
                    entity_gen: entity_id.generation(),
                    component_type: component.component_type().clone(),
                    component_data: component.data().clone(),
                };
                serialized.push_back(ser_comp);
            }
        }
    }

    serialized
}

/// Helper to deserialize World storage components
pub fn deserialize_world_components(world: &mut World, components: &Vec<SerializedComponent>) {
    for ser_comp in components.iter() {
        let entity_id = EntityId::new(ser_comp.entity_id, ser_comp.entity_gen);
        let component = Component::new(ser_comp.component_type.clone(), ser_comp.component_data.clone());

        // Add component to storage (entity should already exist)
        world.storage.add_component(entity_id, component);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, Bytes};

    #[test]
    fn test_serialize_deserialize_empty_components() {
        let env = Env::default();
        let world = World::new();

        let serialized = serialize_world_components(&world, &env);
        assert_eq!(serialized.len(), 0);
    }

    #[test]
    fn test_serialize_deserialize_components() {
        let env = Env::default();
        let mut world = World::new();

        // Spawn an entity with a component
        let mut components = Vec::new(&env);
        let mut data = Bytes::new(&env);
        data.append(&Bytes::from_array(&env, &[1, 2, 3, 4]));
        let component = Component::new(symbol_short!("test"), data.clone());
        components.push_back(component);

        let entity = world.spawn(components);
        let entity_id = entity.id();

        // Serialize components
        let serialized = serialize_world_components(&world, &env);
        assert_eq!(serialized.len(), 1);

        // Create new world and deserialize
        let mut new_world = World::new();
        let new_entity = new_world.spawn_empty();
        assert_eq!(entity_id, new_entity.id());

        deserialize_world_components(&mut new_world, &serialized);

        // Verify component exists in new world
        let component = new_world.get_component(entity_id, &symbol_short!("test"));
        assert!(component.is_some());
        let retrieved = component.unwrap();
        assert_eq!(retrieved.data(), &data);
    }
}
