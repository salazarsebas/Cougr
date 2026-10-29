//! End-to-end session UX for frictionless on-chain gameplay.
//!
//! Orchestrates [`crate::auth`] primitives into an approval → play → renew flow.
//! Maturity: Beta.

mod component;
mod manager;
mod status;

pub use component::ActiveSession;
pub use manager::SessionManager;
pub use status::SessionStatus;

/// Session layer version marker.
pub const MODULE_VERSION: &str = "0.1.0-session";

/// When `expires_in` drops below this many seconds, [`SessionStatus::needs_renewal`] is set.
pub const RENEWAL_HINT_SECONDS: u64 = 300;

/// Session operations reuse account kernel errors.
pub type SessionError = crate::accounts::AccountError;
