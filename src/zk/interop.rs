//! Snarkjs/Circom interoperability constructors for Cougr ZK types.
//!
//! Eliminates the manual byte-wiring needed to go from snarkjs output to
//! Cougr types. Use `include_bytes!` to embed raw point bytes as compile-time
//! constants in your contract.
//!
//! ## Expected byte layout
//!
//! All coordinates are **big-endian BN254 field elements**:
//!
//! - **G1 points** (64 bytes): `x ‖ y`, each 32 bytes.
//! - **G2 points** (128 bytes): `x_c0 ‖ x_c1 ‖ y_c0 ‖ y_c1`, each 32 bytes.
//!
//! `snarkjs zkey export rawcalldata` produces this layout after stripping the
//! `0x` prefix and converting from hex.
//!
//! ## Example
//! ```no_run
//! use cougr_core::zk::{Groth16Proof, VerificationKey};
//! use soroban_sdk::Env;
//!
//! // embed at build time:
//! // const VK_ALPHA: [u8; 64]   = *include_bytes!("../circuits/vk_alpha.bin");
//! // const VK_BETA:  [u8; 128]  = *include_bytes!("../circuits/vk_beta.bin");
//! // ...
//! // const IC: [[u8; 64]; 2]    = [ ... ];
//! //
//! // let vk = VerificationKey::from_raw_bytes(&env,
//! //     &VK_ALPHA, &VK_BETA, &VK_GAMMA, &VK_DELTA, &IC);
//! ```

use soroban_sdk::{BytesN, Env, Vec};

use super::types::{G1Point, G2Point, Groth16Proof, VerificationKey};

impl VerificationKey {
    /// Construct a `VerificationKey` from raw uncompressed BN254 point bytes.
    ///
    /// - `alpha_g1`: 64 bytes (G1 uncompressed, x ‖ y)
    /// - `beta_g2`, `gamma_g2`, `delta_g2`: 128 bytes each (G2 uncompressed)
    /// - `ic`: slice of 64-byte G1 chunks - one per public input + 1
    pub fn from_raw_bytes(
        env: &Env,
        alpha_g1: &[u8; 64],
        beta_g2: &[u8; 128],
        gamma_g2: &[u8; 128],
        delta_g2: &[u8; 128],
        ic: &[[u8; 64]],
    ) -> Self {
        let mut ic_vec: Vec<G1Point> = Vec::new(env);
        for chunk in ic {
            ic_vec.push_back(G1Point {
                bytes: BytesN::from_array(env, chunk),
            });
        }
        Self {
            alpha: G1Point {
                bytes: BytesN::from_array(env, alpha_g1),
            },
            beta: G2Point {
                bytes: BytesN::from_array(env, beta_g2),
            },
            gamma: G2Point {
                bytes: BytesN::from_array(env, gamma_g2),
            },
            delta: G2Point {
                bytes: BytesN::from_array(env, delta_g2),
            },
            ic: ic_vec,
        }
    }
}

impl Groth16Proof {
    /// Construct a `Groth16Proof` from raw uncompressed BN254 point bytes.
    ///
    /// - `a`: 64 bytes (G1 uncompressed)
    /// - `b`: 128 bytes (G2 uncompressed)
    /// - `c`: 64 bytes (G1 uncompressed)
    pub fn from_raw_bytes(env: &Env, a: &[u8; 64], b: &[u8; 128], c: &[u8; 64]) -> Self {
        Self {
            a: G1Point {
                bytes: BytesN::from_array(env, a),
            },
            b: G2Point {
                bytes: BytesN::from_array(env, b),
            },
            c: G1Point {
                bytes: BytesN::from_array(env, c),
            },
        }
    }
}
