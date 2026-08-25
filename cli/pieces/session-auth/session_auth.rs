//! Session policy helpers copied from the `session_arena` example.

use cougr_core::accounts::{GameAction, SessionBuilder, SessionScope};
use soroban_sdk::{symbol_short, Bytes, Env, Symbol};

/// The only action a session key may authorize by default.
pub fn tap_action_name() -> Symbol {
    symbol_short!("tap")
}

/// Lifetime of one signed intent.
pub const INTENT_WINDOW: u64 = 60;
/// Maximum lifetime accepted by this policy.
pub const MAX_SESSION_LIFETIME: u64 = 86_400;
/// Maximum number of operations accepted by this policy.
pub const MAX_SESSION_OPERATIONS: u32 = 10_000;

/// Build a narrowly scoped session for a tap-like action.
pub fn tap_scope(env: &Env, max_operations: u32, expires_in: u64) -> SessionScope {
    SessionBuilder::new(env)
        .allow_action(tap_action_name())
        .max_operations(max_operations)
        .expires_in(expires_in)
        .build_scope()
}

/// Build the action payload used by the session.
pub fn tap_action(env: &Env) -> GameAction {
    GameAction {
        system_name: tap_action_name(),
        data: Bytes::new(env),
    }
}

/// Give an intent a short replay window.
pub fn intent_deadline(env: &Env) -> u64 {
    env.ledger().timestamp().saturating_add(INTENT_WINDOW)
}
