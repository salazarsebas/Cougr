use soroban_sdk::contracterror;

/// Account-related errors for the Cougr framework.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AccountError {
    Unauthorized = 20,
    SessionExpired = 21,
    InvalidSignature = 22,
    CapabilityNotSupported = 23,
    SessionLimitReached = 24,
    InvalidScope = 25,
    BatchEmpty = 26,
    BatchTooLarge = 27,
    StorageError = 28,
    GuardianAlreadyExists = 29,
    RecoveryNotInitiated = 30,
    TimelockNotExpired = 31,
    ThresholdNotMet = 32,
    MaxGuardiansReached = 33,
    DeviceLimitReached = 34,
    DeviceNotFound = 35,
    RecoveryAlreadyActive = 36,
    NonceMismatch = 37,
    ActionNotAllowed = 38,
    SessionBudgetExceeded = 39,
    IntentExpired = 40,
    InvalidIntent = 41,
    SignerMismatch = 42,
    SessionRevoked = 43,
    SignerNotRegistered = 44,
    InvalidThreshold = 45,
    InsufficientGuardians = 46,
    DuplicateDeviceKey = 47,
}