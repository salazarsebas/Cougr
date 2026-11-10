//! Typed [`SimpleWorld`] construction and injection into contract storage.

use crate::component::ComponentTrait;
use crate::game::SorobanGame;
use crate::simple_world::{EntityId, SimpleWorld};
use soroban_sdk::Env;

use super::harness::GameHarness;

/// Build or seed a [`SimpleWorld`] and persist it through [`SorobanGame`].
pub struct WorldFixture {
    world: SimpleWorld,
}

impl WorldFixture {
    pub fn empty(env: &Env) -> Self {
        Self {
            world: SimpleWorld::new(env),
        }
    }

    pub fn with_world(world: SimpleWorld) -> Self {
        Self { world }
    }

    pub fn world(&self) -> &SimpleWorld {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut SimpleWorld {
        &mut self.world
    }

    pub fn spawn_entity(&mut self) -> EntityId {
        self.world.spawn_entity()
    }

    pub fn set_typed<T: ComponentTrait>(&mut self, env: &Env, entity: EntityId, component: &T) {
        self.world.set_typed(env, entity, component);
    }

    pub fn entity_count(&self) -> u32 {
        self.world.next_entity_id().saturating_sub(1)
    }

    /// Write this world into the contract's instance storage via `SorobanGame`.
    pub fn inject<G: SorobanGame>(&self, harness: &GameHarness) {
        let env = harness.env();
        harness.as_contract(|| {
            G::save_world(env, &self.world);
        });
    }

    /// Read the live world back out of contract instance storage.
    pub fn read_from_contract<G: SorobanGame>(harness: &GameHarness) -> Self {
        let env = harness.env();
        let world = harness.as_contract(|| G::load_world(env));
        Self { world }
    }
}
