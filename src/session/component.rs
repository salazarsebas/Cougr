use soroban_sdk::contracttype;

/// Optional on-chain marker for the player's active gameplay session.
///
/// Games can store this as a rich component so off-chain clients can poll
/// [`crate::session::SessionStatus`] when `needs_renewal` becomes true.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveSession {
    pub key_id: soroban_sdk::BytesN<32>,
    pub expires_at: u64,
    pub operations_remaining: u32,
    pub needs_renewal: bool,
}

impl ActiveSession {
    pub fn from_status(status: &super::SessionStatus, expires_at: u64) -> Self {
        Self {
            key_id: status.key_id.clone(),
            expires_at,
            operations_remaining: status.remaining_operations,
            needs_renewal: status.needs_renewal,
        }
    }
}
