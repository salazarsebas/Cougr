//! Generic trait for game circuits and shared scalar encoding helpers.

use soroban_sdk::{unwrap::UnwrapOptimized, BytesN, Env};

use super::error::ZKError;
use super::groth16::verify_groth16;
use super::types::{Groth16Proof, Scalar, VerificationKey};

/// Generic trait for game circuits that verify ZK proofs on-chain.
///
/// Implementors provide a verification key. The default `verify_with_inputs`
/// method handles the common Groth16 verification flow.
///
/// # Example
/// ```no_run
/// use cougr_core::zk::{G1Point, G2Point, Scalar};
/// use cougr_core::zk::experimental::{GameCircuit, Groth16Proof, MovementCircuit, VerificationKey};
/// use soroban_sdk::{BytesN, Env, Vec};
///
/// let env = Env::default();
/// let g1 = G1Point { bytes: BytesN::from_array(&env, &[0u8; 64]) };
/// let g2 = G2Point { bytes: BytesN::from_array(&env, &[0u8; 128]) };
/// let vk = VerificationKey {
///     alpha: g1.clone(),
///     beta: g2.clone(),
///     gamma: g2.clone(),
///     delta: g2,
///     ic: Vec::from_array(&env, [g1.clone()]),
/// };
/// let proof = Groth16Proof { a: g1.clone(), b: vk.beta.clone(), c: g1 };
/// let public_inputs: [Scalar; 0] = [];
/// let circuit = MovementCircuit::new(vk, 10);
/// let _result = circuit.verify_with_inputs(&env, &proof, &public_inputs)?;
/// # Ok::<(), cougr_core::zk::ZKError>(())
/// ```
pub trait GameCircuit {
    /// Get the verification key for this circuit.
    fn verification_key(&self) -> &VerificationKey;

    /// Verify a proof against this circuit's VK and the given public inputs.
    ///
    /// Default implementation delegates to `verify_groth16`.
    fn verify_with_inputs(
        &self,
        env: &Env,
        proof: &Groth16Proof,
        public_inputs: &[Scalar],
    ) -> Result<bool, ZKError> {
        verify_groth16(env, self.verification_key(), proof, public_inputs)
    }
}

/// Encode a `u32` as a BN254 Fr scalar (big-endian), matching Circom/snarkjs public signals.
pub fn field_u32_to_scalar(env: &Env, val: u32) -> Scalar {
    let u = soroban_sdk::U256::from_u32(env, val);
    Scalar {
        bytes: u.to_be_bytes().try_into().unwrap_optimized(),
    }
}

/// Encode an `i32` as a BN254 Fr scalar (big-endian), matching Circom signal semantics.
pub fn field_i32_to_scalar(env: &Env, val: i32) -> Scalar {
    if val >= 0 {
        return field_u32_to_scalar(env, val as u32);
    }
    const FR_MODULUS_BE: [u8; 32] = [
        0x30, 0x64, 0x4e, 0x72, 0xe1, 0x31, 0xa0, 0x29, 0xb8, 0x50, 0x45, 0xb6, 0x81, 0x81, 0x58,
        0x5d, 0x28, 0x33, 0xe8, 0x48, 0x79, 0xb9, 0x70, 0x91, 0x43, 0xe1, 0xf5, 0x93, 0xf0, 0x00,
        0x00, 0x01,
    ];
    let modulus =
        soroban_sdk::U256::from_be_bytes(env, &soroban_sdk::Bytes::from_array(env, &FR_MODULUS_BE));
    let abs = soroban_sdk::U256::from_u32(env, (-val) as u32);
    let field = modulus.sub(&abs);
    Scalar {
        bytes: field.to_be_bytes().try_into().unwrap_optimized(),
    }
}

/// Convert a `u32` value to a BN254 scalar (little-endian, zero-padded to 32 bytes).
pub fn u32_to_scalar(env: &Env, val: u32) -> Scalar {
    let mut bytes = [0u8; 32];
    bytes[..4].copy_from_slice(&val.to_le_bytes());
    Scalar {
        bytes: BytesN::from_array(env, &bytes),
    }
}

/// Convert an `i32` value to a BN254 scalar (little-endian, zero-padded to 32 bytes).
pub fn i32_to_scalar(env: &Env, val: i32) -> Scalar {
    let mut bytes = [0u8; 32];
    bytes[..4].copy_from_slice(&val.to_le_bytes());
    Scalar {
        bytes: BytesN::from_array(env, &bytes),
    }
}

/// Convert a `u64` value to a BN254 scalar (little-endian, zero-padded to 32 bytes).
pub fn u64_to_scalar(env: &Env, val: u64) -> Scalar {
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&val.to_le_bytes());
    Scalar {
        bytes: BytesN::from_array(env, &bytes),
    }
}

/// Convert a `BytesN<32>` directly to a `Scalar` (identity mapping).
pub fn bytes32_to_scalar(val: &BytesN<32>) -> Scalar {
    Scalar { bytes: val.clone() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_u32_to_scalar() {
        let env = Env::default();
        let s = u32_to_scalar(&env, 42);
        assert_eq!(s.bytes.len(), 32);
        let arr = s.bytes.to_array();
        assert_eq!(arr[0], 42);
        assert_eq!(arr[1], 0);
    }

    #[test]
    fn test_i32_to_scalar_positive() {
        let env = Env::default();
        let s = i32_to_scalar(&env, 100);
        assert_eq!(s.bytes.len(), 32);
        let arr = s.bytes.to_array();
        assert_eq!(arr[0], 100);
    }

    #[test]
    fn test_i32_to_scalar_negative() {
        let env = Env::default();
        let s = i32_to_scalar(&env, -1);
        let arr = s.bytes.to_array();
        // -1 in little-endian i32 = [0xFF, 0xFF, 0xFF, 0xFF]
        assert_eq!(arr[0], 0xFF);
        assert_eq!(arr[1], 0xFF);
        assert_eq!(arr[2], 0xFF);
        assert_eq!(arr[3], 0xFF);
    }

    #[test]
    fn test_bytes32_to_scalar() {
        let env = Env::default();
        let b = BytesN::from_array(&env, &[7u8; 32]);
        let s = bytes32_to_scalar(&b);
        assert_eq!(s.bytes, b);
    }

    #[test]
    fn test_u64_to_scalar() {
        let env = Env::default();
        let s = u64_to_scalar(&env, 0x0102_0304_0506_0708);
        let arr = s.bytes.to_array();
        assert_eq!(arr[..8], [0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
    }
}
