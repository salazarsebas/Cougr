use soroban_sdk::{BytesN, Env, Vec};

use super::types::{G1Point, G2Point, Groth16Proof, Scalar, VerificationKey};

/// Create a mock G1 point filled with zeros (not a valid curve point).
/// For testing only - do not use in production.
pub fn mock_g1_point(env: &Env) -> G1Point {
    G1Point {
        bytes: BytesN::from_array(env, &[0u8; 64]),
    }
}

/// Create a mock G2 point filled with zeros (not a valid curve point).
/// For testing only.
pub fn mock_g2_point(env: &Env) -> G2Point {
    G2Point {
        bytes: BytesN::from_array(env, &[0u8; 128]),
    }
}

/// Create a mock scalar from a `u64` value (zero-padded to 32 bytes, big-endian).
/// For testing only.
pub fn mock_scalar(env: &Env, value: u64) -> Scalar {
    let mut bytes = [0u8; 32];
    bytes[24..32].copy_from_slice(&value.to_be_bytes());
    Scalar {
        bytes: BytesN::from_array(env, &bytes),
    }
}

/// Create a mock Groth16 proof with zero-filled points.
/// For testing only.
pub fn mock_proof(env: &Env) -> Groth16Proof {
    Groth16Proof {
        a: mock_g1_point(env),
        b: mock_g2_point(env),
        c: mock_g1_point(env),
    }
}

/// Create a mock verification key with `num_public_inputs` IC points.
/// For testing only.
pub fn mock_verification_key(env: &Env, num_public_inputs: u32) -> VerificationKey {
    let mut ic = Vec::new(env);
    for _ in 0..=num_public_inputs {
        ic.push_back(mock_g1_point(env));
    }
    VerificationKey {
        alpha: mock_g1_point(env),
        beta: mock_g2_point(env),
        gamma: mock_g2_point(env),
        delta: mock_g2_point(env),
        ic,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_mock_g1_point() {
        let env = Env::default();
        let point = mock_g1_point(&env);
        assert_eq!(point.bytes.len(), 64);
    }

    #[test]
    fn test_mock_g2_point() {
        let env = Env::default();
        let point = mock_g2_point(&env);
        assert_eq!(point.bytes.len(), 128);
    }

    #[test]
    fn test_mock_scalar() {
        let env = Env::default();
        let scalar = mock_scalar(&env, 42);
        assert_eq!(scalar.bytes.len(), 32);
    }

    #[test]
    fn test_mock_proof() {
        let env = Env::default();
        let proof = mock_proof(&env);
        assert_eq!(proof.a.bytes.len(), 64);
        assert_eq!(proof.b.bytes.len(), 128);
        assert_eq!(proof.c.bytes.len(), 64);
    }

    #[test]
    fn test_mock_verification_key() {
        let env = Env::default();
        let vk = mock_verification_key(&env, 3);
        assert_eq!(vk.ic.len(), 4); // num_public_inputs + 1
    }
}
