use soroban_sdk::contracterror;

/// Error types for the Cougr ECS framework.
///
/// Uses `#[contracterror]` for Soroban contract compatibility.
/// Each variant maps to a `u32` error code for on-chain error reporting.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CougrError {
    /// Entity with the given ID was not found
    EntityNotFound = 1,
    /// Component not found for the given entity
    ComponentNotFound = 2,
    /// Failed to deserialize component/resource data
    DeserializationFailed = 3,
    /// Data length does not match expected size
    InvalidDataLength = 4,
    /// Index out of bounds during storage access
    IndexOutOfBounds = 5,
    /// Resource with the given type was not found
    ResourceNotFound = 6,
    /// Storage operation failed
    StorageError = 7,
}

/// Convenience type alias for Results using CougrError
pub type CougrResult<T> = Result<T, CougrError>;
