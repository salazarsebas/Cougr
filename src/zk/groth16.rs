use soroban_sdk::Env;

use super::crypto::{bn254_g1_add, bn254_g1_mul, bn254_g1_neg, bn254_pairing_check};
use super::error::ZKError;
use super::types::{Groth16Proof, Scalar, VerificationKey};

/// Verify a Groth16 proof against a verification key and public inputs.
///
/// Implements the standard Groth16 verification equation:
///   e(A, B) == e(alpha, beta) * e(vk_x, gamma) * e(C, delta)
///
/// Where `vk_x = IC[0] + sum(public_inputs[i] * IC[i+1])`.
///
/// # Arguments
/// - `env`: Soroban environment
/// - `vk`: Verification key from the trusted setup
/// - `proof`: The Groth16 proof (A, B, C points)
/// - `public_inputs`: Public inputs as BN254 scalars
///
/// # Returns
/// - `Ok(true)` if the proof is valid
/// - `Ok(false)` if the pairing check fails
/// - `Err(ZKError)` if inputs are malformed
///
/// # Verification Contract
///
/// This verifier treats the following as explicit contract guarantees:
/// - `vk.ic.len()` must equal `public_inputs.len() + 1`
/// - malformed verification-key shape returns `InvalidVerificationKey`
/// - mismatched pairing input lengths return `InvalidInput`
///
/// This verifier does **not** currently promise stronger normalization or
/// subgroup-validation guarantees beyond what Soroban's BN254 host types
/// enforce when decoding points and scalars. For that reason the Groth16
/// verification flow remains part of Cougr's experimental privacy surface.
pub fn validate_groth16_contract(
    vk: &VerificationKey,
    public_inputs: &[Scalar],
) -> Result<(), ZKError> {
    if vk.ic.is_empty() || vk.ic.len() as usize != public_inputs.len() + 1 {
        return Err(ZKError::InvalidVerificationKey);
    }

    Ok(())
}

pub fn verify_groth16(
    env: &Env,
    vk: &VerificationKey,
    proof: &Groth16Proof,
    public_inputs: &[Scalar],
) -> Result<bool, ZKError> {
    validate_groth16_contract(vk, public_inputs)?;

    // Compute vk_x = IC[0] + sum(public_inputs[i] * IC[i+1])
    let mut vk_x = vk.ic.get(0).ok_or(ZKError::InvalidVerificationKey)?;

    for (i, input) in public_inputs.iter().enumerate() {
        let ic_point = vk
            .ic
            .get((i + 1) as u32)
            .ok_or(ZKError::InvalidVerificationKey)?;
        let term = bn254_g1_mul(env, &ic_point, input)?;
        vk_x = bn254_g1_add(env, &vk_x, &term)?;
    }

    // Pairing check (snarkjs-compatible):
    //   e(-A, B) * e(alpha, beta) * e(vk_x, gamma) * e(C, delta) == 1
    let neg_a = bn254_g1_neg(env, &proof.a);
    let g1_points = [neg_a, vk.alpha.clone(), vk_x, proof.c.clone()];
    let g2_points = [
        proof.b.clone(),
        vk.beta.clone(),
        vk.gamma.clone(),
        vk.delta.clone(),
    ];

    bn254_pairing_check(env, &g1_points, &g2_points)
}

#[cfg(test)]
mod tests {
    use super::super::types::G1Point;
    use super::*;
    use soroban_sdk::{BytesN, Env, Vec};

    fn make_g1(env: &Env) -> G1Point {
        G1Point {
            bytes: BytesN::from_array(env, &[0u8; 64]),
        }
    }

    fn make_g2(env: &Env) -> super::super::types::G2Point {
        super::super::types::G2Point {
            bytes: BytesN::from_array(env, &[0u8; 128]),
        }
    }

    #[test]
    fn test_verify_groth16_wrong_ic_length() {
        let env = Env::default();
        let g1 = make_g1(&env);
        let g2 = make_g2(&env);

        let mut ic = Vec::new(&env);
        ic.push_back(g1.clone()); // IC has 1 element

        let vk = VerificationKey {
            alpha: g1.clone(),
            beta: g2.clone(),
            gamma: g2.clone(),
            delta: g2,
            ic,
        };

        let proof = Groth16Proof {
            a: g1.clone(),
            b: make_g2(&env),
            c: g1,
        };

        // 1 public input but IC has only 1 element (needs 2)
        let scalar = Scalar {
            bytes: BytesN::from_array(&env, &[0u8; 32]),
        };
        let result = verify_groth16(&env, &vk, &proof, &[scalar]);
        assert_eq!(result, Err(ZKError::InvalidVerificationKey));
    }

    #[test]
    fn test_verify_groth16_empty_ic() {
        let env = Env::default();
        let g1 = make_g1(&env);
        let g2 = make_g2(&env);

        let ic = Vec::new(&env); // empty

        let vk = VerificationKey {
            alpha: g1.clone(),
            beta: g2.clone(),
            gamma: g2.clone(),
            delta: g2,
            ic,
        };

        let proof = Groth16Proof {
            a: g1.clone(),
            b: make_g2(&env),
            c: g1,
        };

        // 0 public inputs but IC has 0 elements (needs 1)
        let result = verify_groth16(&env, &vk, &proof, &[]);
        assert_eq!(result, Err(ZKError::InvalidVerificationKey));
    }

    #[test]
    fn test_validate_groth16_contract_accepts_matching_lengths() {
        let env = Env::default();
        let g1 = make_g1(&env);
        let g2 = make_g2(&env);
        let mut ic = Vec::new(&env);
        ic.push_back(g1.clone());
        ic.push_back(g1);

        let vk = VerificationKey {
            alpha: make_g1(&env),
            beta: g2.clone(),
            gamma: g2.clone(),
            delta: g2,
            ic,
        };

        let scalar = Scalar {
            bytes: BytesN::from_array(&env, &[0u8; 32]),
        };
        assert_eq!(validate_groth16_contract(&vk, &[scalar]), Ok(()));
    }
}
