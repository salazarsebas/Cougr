//! Emergency pause helper copied from Cougr's standards layer.

pub use cougr_core::standards::{Pausable, PausedEvent, StandardsError, UnpausedEvent};

/// Construct a namespaced pause state for a contract.
pub fn new_pause_state(id: soroban_sdk::Symbol) -> Pausable {
    Pausable::new(id)
}
