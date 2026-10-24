use soroban_sdk::contracterror;

/// Errors returned by the reusable standards layer.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StandardsError {
    Unauthorized = 60,
    AlreadyInitialized = 61,
    OwnerNotSet = 62,
    PendingOwnerNotSet = 63,
    PendingOwnerMismatch = 64,
    RoleAlreadyGranted = 65,
    RoleNotGranted = 66,
    MissingRoleAdmin = 67,
    Paused = 68,
    NotPaused = 69,
    ExecutionLocked = 70,
    RecoveryActive = 71,
    RecoveryInactive = 72,
    BatchEmpty = 73,
    BatchTooLarge = 74,
    OperationNotReady = 75,
    OperationExpired = 76,
    OperationNotFound = 77,
    OperationAlreadyExecuted = 78,
    ExecutionNotLocked = 79,
    ScheduleOverflow = 80,
}
