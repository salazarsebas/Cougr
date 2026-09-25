//! Account abstraction for Cougr game accounts.
//!
//! This module provides a unified interface for both Classic (G-address)
//! and Contract (C-address) Stellar accounts, enabling features like
//! session keys for gasless gameplay.
//!
//! ## Architecture
//!
//! - root re-exports provide the default onboarding path
//! - advanced flows remain grouped by purpose (`recovery`, `multi_device`, `passkey`)
//! - storage submodules support maintainers and integrations that need lower-level access
//!
//! ## Usage
//!
//! ```no_run
//! use cougr_core::accounts::{ClassicAccount, CougrAccount, GameAction};
//! use soroban_sdk::{symbol_short, testutils::Address as _, Address, Bytes, Env};
//!
//! let env = Env::default();
//! let player_address = Address::generate(&env);
//! let account = ClassicAccount::new(player_address);
//! let action = GameAction { system_name: symbol_short!("move"), data: Bytes::new(&env) };
//! account.authorize(&env, &action)?;
//! # Ok::<(), cougr_core::accounts::AccountError>(())
//! ```

pub(crate) mod batch;
pub(crate) mod classic;
pub(crate) mod contract;
pub(crate) mod degradation;
pub(crate) mod device_storage;
pub(crate) mod error;
pub(crate) mod intent;
pub(crate) mod kernel;
pub mod multi_device;
pub(crate) mod policy;
pub mod recovery;
pub(crate) mod recovery_storage;
pub(crate) mod replay;
pub(crate) mod secp256r1_auth;
pub(crate) mod session_builder;
pub(crate) mod signer;
pub(crate) mod storage;
#[cfg(any(test, feature = "testutils"))]
pub(crate) mod testing;
pub(crate) mod traits;
pub(crate) mod types;

/// Passkey and WebAuthn helpers.
pub mod passkey {
    pub use super::secp256r1_auth::{verify_secp256r1, Secp256r1Key, Secp256r1Storage};
}

// Curated root re-exports for the Beta accounts API.
pub use batch::BatchBuilder;
pub use classic::ClassicAccount;
pub use contract::ContractAccount;
pub use degradation::{authorize_with_fallback, batch_or_sequential, require_capability};
pub use error::AccountError;
pub use kernel::AccountKernel;
pub use multi_device::{DeviceKey, DeviceManager, DevicePolicy, MultiDeviceProvider};
pub use recovery::{
    Guardian, RecoverableAccount, RecoveryConfig, RecoveryProvider, RecoveryRequest,
};
pub use replay::ReplayProtection;
pub use secp256r1_auth::{verify_secp256r1, Secp256r1Key, Secp256r1Storage};
pub use session_builder::{derive_session_key_id, SessionBuilder};
pub use storage::SessionStorage;
pub use traits::{CougrAccount, IntentAccount, SessionKeyProvider};
pub use types::{AccountCapabilities, GameAction, SessionKey, SessionScope};

// Lower-level support exports for advanced integrations.
pub use device_storage::DeviceStorage;
pub use intent::{
    AuthMethod, AuthResult, IntentProof, IntentProofKind, IntentSigner, SignedIntent, SignerRef,
};
pub use policy::{
    ActiveDevicePolicy, GuardianPolicy, IntentContext, IntentExpiryPolicy, Policy, RecoveryContext,
    SessionContext, SessionPolicy,
};
pub use recovery_storage::RecoveryStorage;
pub use signer::{AccountSigner, DirectAuthSigner, Secp256r1PasskeySigner, SessionAuthSigner};
#[cfg(any(test, feature = "testutils"))]
pub use testing::MockAccount;
