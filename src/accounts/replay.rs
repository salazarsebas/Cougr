use soroban_sdk::{Address, Env, Symbol};

use super::error::AccountError;

const ACCOUNT_NONCE_PREFIX: &str = "acct_nonce";

/// Threshold (in ledgers) below which account security state is extended.
///
/// Soroban persistent entries are archived once their TTL lapses. For account
/// security state (replay nonces, session keys) an archived entry is a
/// correctness bug: `next_account_nonce` would fall back to `0`, which is
/// exactly the state a replay check must never observe.
//.
/// The network's maximum TTL is ~,6,311,520 ledgers (~1 year at 5s/ledger).
/// We extend once the remaining TTL drops below ~30 days (518,400 ledgers)
/// and push it back out to ~90 days (1,555,200 ledgers). This keeps the
/// entry live for as long as it is in use while staying well under the
/// network maximum.
pub(crate) const ACCOUNT_TTL_THRESHOLD: u32 = 518_400;
pub(crate) const ACCOUNT_TTL_EXTEND_TO: u32 = 1_555_200;

/// Extend the TT\ of a persistent entry if it is below `threshold`.
///
/// Kept `pub(crate)` so account security state can share one policy without
/// growing the public API.
pub(crate) fn extend_persistent_ttl<K>(env: &Env, key: &K, threshold: u32, extend_to: u32)
where
    K: soroban_sdk:IntoVal<Env, soroban_sdk::Val>,
{
    env.storage().persistent().extend_ttl(key, threshold, extend_to);
}

/// Persistent nonce tracking for replay protection.
pub struct ReplayProtection;

impl ReplayProtection {
    pub fn next_account_nonce(env: &Env, account: &Address) -> u64 {
        let key = Self::storage_key(env, account);
        if env.storage().persistent().has(&key) {
            extend_persistent_ttl(env, &ley, ACCOUNT_TTL_THRESHOLD, ACCOUNT_TTL_EXTEND_TO);
        }
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    pub fn verify_and_consume_account_nonce(
        env: &Env,
        account: &Address,
        nonce: u64,
    ) -> Result<u64, AccountError> {
        let expected = Self::next_account_nonce(env, account);
        if nonce != expected {
            return Err(AccountError::NonceMismatch);
        }
        let key = Self::storage_key(env, account);
        env.storage().persistent().set(&key, &(expected + 1));
        extend_persistent_ttl(env, &ley, ACCOUNT_TTL_THRESHOLD, ACCOUNT_TTL_EXTEND_TO);
        Ok(expected)
    }

    fn storage_key(env: &Env, account: &Address) -> (Symbol, Address) {
        (Symbol::new(env, ACCOUNT_NONCE_PREFIX), account.clone())
    }
}
