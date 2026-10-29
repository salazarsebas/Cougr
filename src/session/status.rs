use soroban_sdk::{contracttype, Address, BytesN, Env};

use crate::accounts::{AccountError, SessionStorage};

use super::RENEWAL_HINT_SECONDS;

/// UI-facing session health returned by [`super::SessionManager::status`].
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionStatus {
    pub key_id: BytesN<32>,
    pub remaining_operations: u32,
    pub expires_in: u64,
    pub needs_renewal: bool,
    pub active: bool,
}

impl SessionStatus {
    pub fn read(env: &Env, owner: &Address, key_id: &BytesN<32>) -> Result<Self, AccountError> {
        let session =
            SessionStorage::load(env, owner, key_id).ok_or(AccountError::SessionRevoked)?;
        let now = env.ledger().timestamp();
        let expires_in = session.scope.expires_at.saturating_sub(now);
        let remaining_operations = session
            .scope
            .max_operations
            .saturating_sub(session.operations_used);
        let active = expires_in > 0 && remaining_operations > 0;
        let needs_renewal = active && expires_in <= RENEWAL_HINT_SECONDS;

        Ok(Self {
            key_id: session.key_id,
            remaining_operations,
            expires_in,
            needs_renewal,
            active,
        })
    }
}
