//! Reusable contract standards for Cougr integrations.
//!
//! The standards layer is intentionally framework-oriented rather than
//! example-specific. Each standard is storage-backed where persistence matters,
//! exposes typed state-transition events, and keeps authorization checks
//! explicit instead of hidden behind implicit caller assumptions.

mod access_control;
mod batch;
mod delayed_execution;
mod error;
mod execution_guard;
mod ownable;
mod pausable;
mod recovery_guard;

pub use access_control::{
    AccessControl, RoleAdminChangedEvent, RoleGrantedEvent, RoleRevokedEvent,
    DEFAULT_ADMIN_ROLE_NAME,
};
pub use batch::BatchExecutor;
pub use delayed_execution::{
    DelayedExecutionCancelledEvent, DelayedExecutionExecutedEvent, DelayedExecutionPolicy,
    DelayedExecutionScheduledEvent, DelayedOperation,
};
pub use error::StandardsError;
pub use execution_guard::{ExecutionGuard, ExecutionGuardEnteredEvent, ExecutionGuardExitedEvent};
pub use ownable::{
    Ownable, Ownable2Step, OwnershipTransferCancelledEvent, OwnershipTransferStartedEvent,
    OwnershipTransferredEvent,
};
pub use pausable::{Pausable, PausedEvent, UnpausedEvent};
pub use recovery_guard::{RecoveryGuard, RecoveryGuardActivatedEvent, RecoveryGuardClearedEvent};
