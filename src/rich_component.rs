use soroban_sdk::{symbol_short, Env, Symbol};

use crate::simple_world::EntityId;

/// Marker trait for components that use Soroban's `#[contracttype]` XDR codec
/// for storage, enabling `Vec`, `String`, `Option`, nested structs, and enums
/// with data - types not supported by the primitive-only [`ComponentTrait`].
///
/// Implement via [`impl_rich_component!`]. The struct must derive
/// `#[contracttype]` so that Soroban handles XDR serialization automatically.
///
/// Rich components are stored in Soroban's **instance storage**, separate from
/// the `Map`-backed ECS storage used by [`ComponentTrait`]. Entity IDs are
/// shared - the same entity can hold both kinds simultaneously.
///
/// # Example
/// ```no_run
/// use cougr_core::impl_rich_component;
/// use soroban_sdk::{contracttype, String, Vec};
///
/// #[contracttype]
/// #[derive(Clone)]
/// pub struct PlayerProfile {
///     pub name: String,
///     pub scores: Vec<u32>,
///     pub level: Option<u32>,
/// }
///
/// impl_rich_component!(PlayerProfile, "player_profile");
/// ```
///
/// [`ComponentTrait`]: crate::component::ComponentTrait
pub trait RichComponentTrait {
    /// Returns a `Symbol` identifying this component type. No 9-character limit.
    fn component_type(env: &Env) -> Symbol;
}

/// Computes the Soroban instance-storage key for a rich component.
///
/// Uses a namespaced tuple key `(cougr_rc, entity_id, component_sym)` to
/// avoid collisions with other contract storage keys.
pub(crate) fn rich_component_key(
    _env: &Env,
    entity_id: EntityId,
    component_sym: Symbol,
) -> (Symbol, u32, Symbol) {
    (symbol_short!("cougr_rc"), entity_id, component_sym)
}
