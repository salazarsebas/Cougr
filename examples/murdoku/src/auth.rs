//! Session key authorization wrapper for Murdoku.
//!
//! Provides `authorize_session`, `revoke_session`, and an internal
//! `require_player_auth` helper used by game entrypoints. Sessions are
//! stored in instance storage under the key `(Symbol("SESSION"), puzzle_id, player)`.

use soroban_sdk::{symbol_short, Address, Bytes, BytesN, Env, Symbol};

use cougr_core::auth::{
    authorize_with_fallback, ContractAccount, GameAction, SessionBuilder,
    SessionKey as CougrSessionKey,
};

/// Maximum ledger offset for session expiry (~24 hours at 5s/ledger)
const MAX_LEDGER_OFFSET: u32 = 17_280;

/// Authorize a session key scoped to `(player, puzzle_id)` for place/remove actions.
pub fn authorize_session(
    env: Env,
    player: Address,
    puzzle_id: u32,
    session_key: BytesN<32>,
    expires_at_ledger: u32,
) {
    // One-time root key authorization
    player.require_auth();

    let now_seq = env.ledger().sequence();
    assert!(expires_at_ledger > now_seq, "expiry must be in the future");
    assert!(
        expires_at_ledger <= now_seq + MAX_LEDGER_OFFSET,
        "expiry exceeds 24 hour window"
    );

    let expires_at = expires_at_ledger as u64;

    // Build a scoped session allowing only place_suspect and remove_suspect
    let _scope = SessionBuilder::new(&env)
        .allow_action(Symbol::new(&env, "place_suspect"))
        .allow_action(Symbol::new(&env, "remove_suspect"))
        .max_operations(200)
        .expires_at(expires_at)
        .build_scope();

    // Construct a Cougr session key compatible with account/session helpers.
    // Keep minimal tracking fields required by the Cougr types.
    let session = CougrSessionKey {
        key_id: session_key.clone(),
        scope: _scope,
        created_at: env.ledger().timestamp(),
        operations_used: 0u32,
        next_nonce: 0u64,
    };

    // Instance-scoped storage key: ("SESSION", puzzle_id, player)
    let storage_key = (Symbol::new(&env, "SESSION"), puzzle_id, player.clone());
    env.storage().instance().set(&storage_key, &session);
}

/// Internal helper used by game entrypoints.
///
/// Attempts session key authorization first; falls back to `player.require_auth()`.
pub fn require_player_auth(env: &Env, player: &Address, puzzle_id: u32, action: Symbol) {
    // Try to load a session scoped to (player, puzzle_id)
    let storage_key = (Symbol::new(env, "SESSION"), puzzle_id, player.clone());

    // Note: `instance().get` returns `Option<T>` when T: ContractType
    let maybe_session: Option<CougrSessionKey> = env.storage().instance().get(&storage_key);

    // Build the GameAction for authorization checks
    let action_obj = GameAction {
        system_name: action,
        data: Bytes::new(env),
    };

    let account = ContractAccount::new(player.clone());

    if let Some(session) = maybe_session {
        // Attempt session authorization; if it fails we fall back
        let res = authorize_with_fallback(env, &account, &action_obj, Some(&session));
        if res.is_ok() {
            return;
        }
    }

    // Fallback to root key authorization
    player.require_auth();
}

/// Revoke a session previously created with `authorize_session`.
pub fn revoke_session(env: Env, player: Address, puzzle_id: u32) {
    player.require_auth();
    let storage_key = (Symbol::new(&env, "SESSION"), puzzle_id, player);
    // Idempotent: if nothing exists this is a no-op
    if env.storage().instance().has(&storage_key) {
        env.storage().instance().remove(&storage_key);
    }
}
