use soroban_sdk::{Address, BytesN, Env};

use super::device_storage::DeviceStorage;
use super::error::AccountError;
use super::intent::SignedIntent;
use super::recovery_storage::RecoveryStorage;
use super::storage::SessionStorage;

/// Generic policy interface used by the account kernel and related subsystems.
pub trait Policy<C> {
    fn evaluate(&self, env: &Env, context: &C) -> Result<(), AccountError>;
}

pub struct IntentContext<'a> {
    pub account: &'a Address,
    pub intent: &'a SignedIntent,
}

pub struct SessionContext<'a> {
    pub account: &'a Address,
    pub intent: &'a SignedIntent,
}

pub struct DeviceContext<'a> {
    pub account: &'a Address,
    pub key_id: &'a BytesN<32>,
}

pub struct RecoveryContext<'a> {
    pub account: &'a Address,
    pub guardian: &'a Address,
}

/// Reject expired intents.
pub struct IntentExpiryPolicy;

impl Policy<IntentContext<'_>> for IntentExpiryPolicy {
    fn evaluate(&self, env: &Env, context: &IntentContext<'_>) -> Result<(), AccountError> {
        if env.ledger().timestamp() >= context.intent.expires_at {
            return Err(AccountError::IntentExpired);
        }
        Ok(())
    }
}

/// Enforces session existence, expiry, scope, budget and nonce.
pub struct SessionPolicy;

impl Policy<SessionContext<'_>> for SessionPolicy {
    fn evaluate(&self, env: &Env, context: &SessionContext<'_>) -> Result<(), AccountError> {
        let session =
            SessionStorage::load(env, context.account, &context.intent.signer.session_key_id)
                .ok_or(AccountError::SessionRevoked)?;

        if env.ledger().timestamp() >= session.scope.expires_at {
            return Err(AccountError::SessionExpired);
        }
        if session.operations_used >= session.scope.max_operations {
            return Err(AccountError::SessionBudgetExceeded);
        }
        if session.next_nonce != context.intent.nonce {
            return Err(AccountError::NonceMismatch);
        }
        if !SessionStorage::is_action_allowed(&session.scope, &context.intent.action.system_name) {
            return Err(AccountError::ActionNotAllowed);
        }
        Ok(())
    }
}

/// Ensures a bound device key is active under the account's device policy.
pub struct ActiveDevicePolicy;

impl Policy<DeviceContext<'_>> for ActiveDevicePolicy {
    fn evaluate(&self, env: &Env, context: &DeviceContext<'_>) -> Result<(), AccountError> {
        let devices = DeviceStorage::load_devices(env, context.account);
        for i in 0..devices.len() {
            if let Some(device) = devices.get(i) {
                if &device.key_id == context.key_id && device.is_active {
                    return Ok(());
                }
            }
        }
        Err(AccountError::DeviceNotFound)
    }
}

/// Ensures an address is currently configured as a guardian.
pub struct GuardianPolicy;

impl Policy<RecoveryContext<'_>> for GuardianPolicy {
    fn evaluate(&self, env: &Env, context: &RecoveryContext<'_>) -> Result<(), AccountError> {
        let guardians = RecoveryStorage::load_guardians(env, context.account);
        for i in 0..guardians.len() {
            if let Some(guardian) = guardians.get(i) {
                if &guardian.address == context.guardian {
                    return Ok(());
                }
            }
        }
        Err(AccountError::Unauthorized)
    }
}
