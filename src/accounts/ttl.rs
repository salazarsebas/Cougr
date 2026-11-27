use soroban_sdk::{Env, IntoVal, Val};

/// Threshold percentage of the network's `max_ttl` below which persistent
/// account security entries are extended on read or write.
///
/// At 25% of Stellar Mainnet's 180-day `max_entry_ttl` (3,110,400 ledgers),
/// entries are bumped once roughly 45 days of TTL remain.
pub(crate) const PERSISTENT_TTL_THRESHOLD_PERCENT: u32 = 25;

/// Target TTL as a percentage of the network's `max_ttl` when extending
/// persistent account security entries.
///
/// At 50% of Stellar Mainnet's 180-day `max_entry_ttl` (3,110,400 ledgers),
/// active nonces and session lists stay live for roughly 90 days after each
/// extension without hard-coding a network-specific ledger count.
pub(crate) const PERSISTENT_TTL_EXTEND_TO_PERCENT: u32 = 50;

/// Compute `(threshold, extend_to)` ledger counts relative to the current
/// network's `max_ttl`.
pub(crate) fn persistent_ttl_bounds(env: &Env) -> (u32, u32) {
    let max_ttl = u64::from(env.storage().max_ttl());
    let extend_to = ((max_ttl * u64::from(PERSISTENT_TTL_EXTEND_TO_PERCENT)) / 100)
        .max(1)
        .min(max_ttl) as u32;
    let threshold = ((max_ttl * u64::from(PERSISTENT_TTL_THRESHOLD_PERCENT)) / 100)
        .max(1)
        .min(u64::from(extend_to)) as u32;
    (threshold, extend_to)
}

/// Extend the persistent storage TTL for `key` using the shared account
/// security bounds. Callers must ensure `key` already exists in persistent
/// storage.
pub(crate) fn extend_persistent_ttl<K: IntoVal<Env, Val>>(env: &Env, key: &K) {
    let (threshold, extend_to) = persistent_ttl_bounds(env);
    env.storage()
        .persistent()
        .extend_ttl(key, threshold, extend_to);
}
