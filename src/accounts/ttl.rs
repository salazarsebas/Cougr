use soroban_sdk::{Env, IntoVal, Val};

/// Approximate number of Stellar ledgers closed in one day (~5 seconds per ledger).
pub(crate) const DAY_IN_LEDGERS: u32 = 17_280;

/// Minimum remaining TTL (7 days = 120,960 ledgers) below which persistent
/// account storage entries (replay nonces and session-key lists) are bumped.
pub(crate) const ACCOUNT_TTL_THRESHOLD: u32 = 7 * DAY_IN_LEDGERS;

/// Target TTL (30 days = 518,400 ledgers) applied whenever a persistent
/// account storage entry is written or read at or below [`ACCOUNT_TTL_THRESHOLD`].
pub(crate) const ACCOUNT_TTL_EXTEND_TO: u32 = 30 * DAY_IN_LEDGERS;

/// Extend the persistent storage TTL for `key` to [`ACCOUNT_TTL_EXTEND_TO`]
/// ledgers whenever its remaining TTL is at or below [`ACCOUNT_TTL_THRESHOLD`].
///
/// Callers must ensure `key` is present in persistent storage before calling
/// this helper.
pub(crate) fn extend_persistent_ttl<K>(env: &Env, key: &K)
where
    K: IntoVal<Env, Val>,
{
    env.storage()
        .persistent()
        .extend_ttl(key, ACCOUNT_TTL_THRESHOLD, ACCOUNT_TTL_EXTEND_TO);
}
