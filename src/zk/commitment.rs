//! Pedersen commitment scheme using BN254 G1 curve points.
//!
//! A Pedersen commitment `C = v*G + r*H` allows committing to a value `v`
//! with blinding factor `r`, such that:
//! - The commitment hides `v` (information-theoretically secure with random `r`)
//! - The commitment is binding (computationally secure under discrete log assumption)

use soroban_sdk::{contracttype, Env};

use super::crypto::{bn254_g1_add, bn254_g1_mul};
use super::error::ZKError;
use super::types::{G1Point, Scalar};

/// Pedersen commitment parameters: two independent G1 generator points.
///
/// `g` is the "value generator" and `h` is the "blinding generator".
/// These must be chosen such that the discrete log of `h` with respect to `g`
/// is unknown (nothing-up-my-sleeve construction recommended).
#[contracttype]
#[derive(Clone, Debug)]
pub struct PedersenParams {
    /// Value generator point (G1).
    pub g: G1Point,
    /// Blinding generator point (G1).
    pub h: G1Point,
}

/// A Pedersen commitment point on BN254 G1.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PedersenCommitment {
    /// The commitment point C = v*G + r*H.
    pub point: G1Point,
}

/// Create a Pedersen commitment: `C = value * G + blinding * H`.
///
/// # Arguments
/// - `params`: Generator points (G, H)
/// - `value`: The value to commit to (scalar field element)
/// - `blinding`: Random blinding factor (scalar field element)
pub fn pedersen_commit(
    env: &Env,
    params: &PedersenParams,
    value: &Scalar,
    blinding: &Scalar,
) -> Result<PedersenCommitment, ZKError> {
    let vg = bn254_g1_mul(env, &params.g, value)?;
    let rh = bn254_g1_mul(env, &params.h, blinding)?;
    let point = bn254_g1_add(env, &vg, &rh)?;
    Ok(PedersenCommitment { point })
}

/// Verify a Pedersen commitment opening.
///
/// Checks that `commitment == value * G + blinding * H`.
pub fn pedersen_verify(
    env: &Env,
    params: &PedersenParams,
    commitment: &PedersenCommitment,
    value: &Scalar,
    blinding: &Scalar,
) -> Result<bool, ZKError> {
    let expected = pedersen_commit(env, params, value, blinding)?;
    Ok(commitment.point.bytes == expected.point.bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{BytesN, Env};

    #[test]
    fn test_pedersen_params_creation() {
        let env = Env::default();
        let params = PedersenParams {
            g: G1Point {
                bytes: BytesN::from_array(&env, &[0u8; 64]),
            },
            h: G1Point {
                bytes: BytesN::from_array(&env, &[0u8; 64]),
            },
        };
        assert_eq!(params.g.bytes.len(), 64);
        assert_eq!(params.h.bytes.len(), 64);
    }

    #[test]
    fn test_pedersen_commitment_type_creation() {
        let env = Env::default();
        let commitment = PedersenCommitment {
            point: G1Point {
                bytes: BytesN::from_array(&env, &[0u8; 64]),
            },
        };
        assert_eq!(commitment.point.bytes.len(), 64);
    }

    #[test]
    fn test_pedersen_same_commitment_equals() {
        let env = Env::default();
        let c1 = PedersenCommitment {
            point: G1Point {
                bytes: BytesN::from_array(&env, &[1u8; 64]),
            },
        };
        let c2 = PedersenCommitment {
            point: G1Point {
                bytes: BytesN::from_array(&env, &[1u8; 64]),
            },
        };
        assert_eq!(c1.point.bytes, c2.point.bytes);
    }

    #[test]
    fn test_pedersen_different_commitments_differ() {
        let env = Env::default();
        let c1 = PedersenCommitment {
            point: G1Point {
                bytes: BytesN::from_array(&env, &[1u8; 64]),
            },
        };
        let c2 = PedersenCommitment {
            point: G1Point {
                bytes: BytesN::from_array(&env, &[2u8; 64]),
            },
        };
        assert_ne!(c1.point.bytes, c2.point.bytes);
    }
}
