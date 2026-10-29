use soroban_sdk::{BytesN, Env};

use crate::zk::ZKError;

use super::spec::{pipeline_vk, CircuitId, CircuitParams, GameCircuitSpec, PublicInputLayout};

/// Build a fair dice circuit for `sides` faces bound to `seed_commitment`.
///
/// Public inputs: `[seed_commitment, roll_result, sides, nonce]`.
pub fn fair_dice(
    env: &Env,
    sides: u32,
    seed_commitment: &BytesN<32>,
) -> Result<GameCircuitSpec, ZKError> {
    if sides < 2 {
        return Err(ZKError::InvalidInput);
    }

    let layout = PublicInputLayout::fair_dice(env);
    Ok(GameCircuitSpec {
        circuit_id: CircuitId::FairDice,
        layout: layout.clone(),
        verification_key: pipeline_vk(env, CircuitId::FairDice),
        params: CircuitParams::FairDice(sides, seed_commitment.clone()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zk::ZKError;

    #[test]
    fn fair_dice_rejects_single_sided_die() {
        let env = Env::default();
        let seed = BytesN::from_array(&env, &[1u8; 32]);
        assert!(fair_dice(&env, 1, &seed).is_err());
    }

    #[test]
    fn fair_dice_rejects_out_of_range_roll() {
        let env = Env::default();
        let seed = BytesN::from_array(&env, &[1u8; 32]);
        let spec = fair_dice(&env, 6, &seed).unwrap();
        let g1 = crate::zk::types::G1Point {
            bytes: BytesN::from_array(&env, &[0u8; 64]),
        };
        let g2 = crate::zk::types::G2Point {
            bytes: BytesN::from_array(&env, &[0u8; 128]),
        };
        let proof = crate::zk::types::Groth16Proof {
            a: g1.clone(),
            b: g2,
            c: g1,
        };

        assert_eq!(
            spec.verify_dice_roll(&env, &proof, 7, 1),
            Err(ZKError::InvalidInput)
        );
    }
}
