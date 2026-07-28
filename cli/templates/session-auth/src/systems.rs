//! Session policy and gameplay rules for {{crate_name}}.
//!
//! Everything that decides *what a session key is allowed to do* lives here, in
//! one place: the action name, the operation budget, the lifetime, and how long
//! a single signed intent stays valid. Widening a session's powers should be a
//! visible edit to this file, not a change buried in a contract entrypoint.

use cougr_core::accounts::{GameAction, SessionBuilder, SessionScope};
use soroban_sdk::{symbol_short, Bytes, Env, Symbol};

/// The only action a session key may authorize.
pub fn tap_action_name() -> Symbol {
    symbol_short!("tap")
}

/// How long a single signed intent stays valid, in seconds.
///
/// Short by design: an intent is signed and submitted in one round trip, so a
/// wide window only helps someone replaying a captured message.
pub const INTENT_WINDOW: u64 = 60;

/// Longest session lifetime the contract will approve or renew to, in seconds.
pub const MAX_SESSION_LIFETIME: u64 = 86_400;

/// Largest operation budget a single session may carry.
pub const MAX_SESSION_OPERATIONS: u32 = 10_000;

/// Why a requested session is not acceptable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionPolicyError {
    /// `max_operations` is zero or above [`MAX_SESSION_OPERATIONS`].
    BudgetOutOfRange,
    /// `expires_in` is zero or above [`MAX_SESSION_LIFETIME`].
    LifetimeOutOfRange,
}

/// Check a requested session against the contract's policy.
pub fn validate_request(max_operations: u32, expires_in: u64) -> Result<(), SessionPolicyError> {
    if max_operations == 0 || max_operations > MAX_SESSION_OPERATIONS {
        return Err(SessionPolicyError::BudgetOutOfRange);
    }
    if expires_in == 0 || expires_in > MAX_SESSION_LIFETIME {
        return Err(SessionPolicyError::LifetimeOutOfRange);
    }
    Ok(())
}

/// Build the scope a session key is created with.
///
/// The scope is the whole security story: one allowed action, a hard operation
/// budget, and an expiry. A key that leaks can do nothing else.
pub fn tap_scope(env: &Env, max_operations: u32, expires_in: u64) -> SessionScope {
    SessionBuilder::new(env)
        .allow_action(tap_action_name())
        .max_operations(max_operations)
        .expires_in(expires_in)
        .build_scope()
}

/// The gameplay action a tap authorizes.
pub fn tap_action(env: &Env) -> GameAction {
    GameAction {
        system_name: tap_action_name(),
        data: Bytes::new(env),
    }
}

/// Deadline for an intent signed at the current ledger time.
pub fn intent_deadline(env: &Env) -> u64 {
    env.ledger().timestamp().saturating_add(INTENT_WINDOW)
}
