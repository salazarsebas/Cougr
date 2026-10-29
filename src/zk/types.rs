use soroban_sdk::{contracttype, BytesN, Vec};

/// A BN254 G1 affine point (compressed, 64 bytes serialized).
///
/// Wraps the soroban-sdk `Bn254G1Affine` for use in Cougr contract types.
#[contracttype]
#[derive(Clone, Debug)]
pub struct G1Point {
    pub bytes: BytesN<64>,
}

/// A BN254 G2 affine point (compressed, 128 bytes serialized).
#[contracttype]
#[derive(Clone, Debug)]
pub struct G2Point {
    pub bytes: BytesN<128>,
}

/// A BN254 scalar field element (Fr, 32 bytes).
#[contracttype]
#[derive(Clone, Debug)]
pub struct Scalar {
    pub bytes: BytesN<32>,
}

/// A Groth16 proof consisting of three curve points (A ∈ G1, B ∈ G2, C ∈ G1).
#[contracttype]
#[derive(Clone, Debug)]
pub struct Groth16Proof {
    pub a: G1Point,
    pub b: G2Point,
    pub c: G1Point,
}

/// A Groth16 verification key.
#[contracttype]
#[derive(Clone, Debug)]
pub struct VerificationKey {
    /// Alpha point (G1)
    pub alpha: G1Point,
    /// Beta point (G2)
    pub beta: G2Point,
    /// Gamma point (G2)
    pub gamma: G2Point,
    /// Delta point (G2)
    pub delta: G2Point,
    /// IC (input commitment) points (G1), one per public input + 1
    pub ic: Vec<G1Point>,
}

// ─── BLS12-381 Types ───────────────────────────────────────────

/// A BLS12-381 G1 affine point (96 bytes serialized).
#[contracttype]
#[derive(Clone, Debug)]
pub struct Bls12381G1Point {
    pub bytes: BytesN<96>,
}

/// A BLS12-381 G2 affine point (192 bytes serialized).
#[contracttype]
#[derive(Clone, Debug)]
pub struct Bls12381G2Point {
    pub bytes: BytesN<192>,
}

/// A BLS12-381 Fr scalar field element (32 bytes).
#[contracttype]
#[derive(Clone, Debug)]
pub struct Bls12381Scalar {
    pub bytes: BytesN<32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_g1_point_creation() {
        let env = Env::default();
        let bytes = BytesN::from_array(&env, &[0u8; 64]);
        let point = G1Point { bytes };
        assert_eq!(point.bytes.len(), 64);
    }

    #[test]
    fn test_g2_point_creation() {
        let env = Env::default();
        let bytes = BytesN::from_array(&env, &[0u8; 128]);
        let point = G2Point { bytes };
        assert_eq!(point.bytes.len(), 128);
    }

    #[test]
    fn test_scalar_creation() {
        let env = Env::default();
        let bytes = BytesN::from_array(&env, &[0u8; 32]);
        let scalar = Scalar { bytes };
        assert_eq!(scalar.bytes.len(), 32);
    }

    #[test]
    fn test_groth16_proof_creation() {
        let env = Env::default();
        let g1 = G1Point {
            bytes: BytesN::from_array(&env, &[0u8; 64]),
        };
        let g2 = G2Point {
            bytes: BytesN::from_array(&env, &[0u8; 128]),
        };
        let proof = Groth16Proof {
            a: g1.clone(),
            b: g2,
            c: g1,
        };
        assert_eq!(proof.a.bytes.len(), 64);
        assert_eq!(proof.b.bytes.len(), 128);
    }

    #[test]
    fn test_verification_key_creation() {
        let env = Env::default();
        let g1 = G1Point {
            bytes: BytesN::from_array(&env, &[0u8; 64]),
        };
        let g2 = G2Point {
            bytes: BytesN::from_array(&env, &[0u8; 128]),
        };
        let mut ic = Vec::new(&env);
        ic.push_back(g1.clone());

        let vk = VerificationKey {
            alpha: g1,
            beta: g2.clone(),
            gamma: g2.clone(),
            delta: g2,
            ic,
        };
        assert_eq!(vk.ic.len(), 1);
    }
}
