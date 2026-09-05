use soroban_sdk::{contractevent, Bytes, Env, Symbol};

use crate::component::ComponentTrait;
use crate::simple_world::EntityId;

/// Soroban event emitted when a component is set on an entity.
///
/// Topics: `("COUGR", "set", component_type_symbol)`
/// Data: `{ "data": Bytes, "entity_id": u32 }`
///
/// Frontends can subscribe to all component mutations via the `COUGR`+`set`
/// prefix, or to a specific component type by also filtering on the third topic.
#[contractevent(topics = ["COUGR", "set"])]
pub struct ComponentSetEvent {
    #[topic]
    pub component_type: Symbol,
    pub entity_id: u32,
    pub data: Bytes,
}

/// Soroban event emitted when a component is removed from an entity.
///
/// Topics: `("COUGR", "del", component_type_symbol)`
/// Data: `{ "entity_id": u32 }`
#[contractevent(topics = ["COUGR", "del"])]
pub struct ComponentRemovedEvent {
    #[topic]
    pub component_type: Symbol,
    pub entity_id: u32,
}

/// Soroban event emitted when a rich component is set on an entity via
/// [`SimpleWorld::set_rich_observed`](crate::simple_world::SimpleWorld::set_rich_observed).
///
/// Topics: `("COUGR", "rich", component_type_symbol)`
/// Data: `{ "entity_id": u32 }`
///
/// Rich components are stored using Soroban's XDR codec. The full value is
/// not embedded in the event - off-chain indexers should query the contract's
/// instance storage for the updated value after receiving this notification.
#[contractevent(topics = ["COUGR", "rich"])]
pub struct RichComponentChangedEvent {
    #[topic]
    pub component_type: Symbol,
    pub entity_id: u32,
}

/// Extension trait for [`ComponentTrait`] types that emit structured Soroban
/// events on mutation.
///
/// Implement via [`impl_component_observed!`], which generates both
/// `ComponentTrait` and `ObservableComponentTrait` simultaneously.
///
/// [`ComponentTrait`]: crate::component::ComponentTrait
pub trait ObservableComponentTrait: ComponentTrait {
    /// Publish a `set` event. `data` is the serialized component bytes from
    /// [`ComponentTrait::serialize`].
    fn emit_set_event(env: &Env, entity_id: EntityId, data: &Bytes);

    /// Publish a `del` event carrying the entity ID as event data.
    fn emit_remove_event(env: &Env, entity_id: EntityId);
}
