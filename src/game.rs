use crate::simple_world::SimpleWorld;
use soroban_sdk::{Env, Symbol};

/// Standard trait for Soroban contracts that use Cougr's ECS.
///
/// Provides a consistent pattern for loading and persisting the game world
/// to Soroban instance storage - removing the need to repeat the storage key
/// symbol in every contract entrypoint. Wire up once with
/// [`impl_soroban_game!`] and then call `load_world` / `save_world` throughout
/// the contract implementation.
///
/// # Example
/// ```
/// use cougr_core::game::SorobanGame;
/// use cougr_core::impl_soroban_game;
/// use soroban_sdk::{contract, Env};
/// use soroban_sdk::testutils::Register as _;
///
/// #[contract]
/// pub struct MyGame;
///
/// impl_soroban_game!(MyGame, "world");
///
/// # fn main() {
/// # let env = Env::default();
/// # let contract_id = env.register(MyGame, ());
/// # env.as_contract(&contract_id, || {
/// let mut world = MyGame::load_world(&env);
/// let id = world.spawn_entity();
/// MyGame::save_world(&env, &world);
/// # assert!(id > 0);
/// # });
/// # }
/// ```
pub trait SorobanGame {
    /// The Soroban `Symbol` key used to store the world in instance storage.
    fn world_key(env: &Env) -> Symbol;

    /// Load the game world from Soroban instance storage.
    ///
    /// Returns a fresh empty world if the contract has not been initialised yet
    /// (i.e. the key does not exist in instance storage).
    fn load_world(env: &Env) -> SimpleWorld {
        SimpleWorld::load_from_instance(env, &Self::world_key(env))
    }

    /// Persist the game world to Soroban instance storage.
    ///
    /// Call at the end of every mutating entrypoint after all ECS operations
    /// have been applied.
    fn save_world(env: &Env, world: &SimpleWorld) {
        world.save_to_instance(env, &Self::world_key(env));
    }
}
