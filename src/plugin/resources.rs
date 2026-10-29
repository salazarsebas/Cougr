use crate::resource::{Resource, ResourceTrait};
use alloc::vec::Vec;
use soroban_sdk::Env;

pub(super) fn insert_resource<R: ResourceTrait>(
    resources: &mut Vec<Resource>,
    env: &Env,
    resource: &R,
) {
    let resource_type = R::resource_type();
    let encoded = Resource::new(resource_type.clone(), resource.serialize(env));

    for index in 0..resources.len() {
        if let Some(existing) = resources.get(index) {
            if existing.resource_type == resource_type {
                resources[index] = encoded;
                return;
            }
        }
    }

    resources.push(encoded);
}

pub(super) fn get_resource<R: ResourceTrait>(resources: &[Resource], env: &Env) -> Option<R> {
    let resource_type = R::resource_type();
    for resource in resources {
        if resource.resource_type == resource_type {
            return R::deserialize(env, &resource.data);
        }
    }
    None
}

pub(super) fn remove_resource<R: ResourceTrait>(resources: &mut Vec<Resource>) -> Option<Resource> {
    let resource_type = R::resource_type();
    let mut found = None;
    let mut retained = Vec::new();

    for resource in resources.iter() {
        if resource.resource_type == resource_type {
            found = Some(resource.clone());
        } else {
            retained.push(resource.clone());
        }
    }

    if found.is_some() {
        *resources = retained;
    }

    found
}
