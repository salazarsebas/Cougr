use soroban_sdk::Env;

use crate::zk::ZKError;

use super::spec::{pipeline_vk, CircuitId, CircuitParams, GameCircuitSpec, PublicInputLayout};

/// Build a sealed-bid auction circuit capped at `max_bid`.
///
/// Public inputs: `[auction_id, bid_commitment, revealed_bid, max_bid]`.
pub fn sealed_bid(env: &Env, max_bid: u32) -> Result<GameCircuitSpec, ZKError> {
    if max_bid == 0 {
        return Err(ZKError::InvalidInput);
    }

    let layout = PublicInputLayout::sealed_bid(env);
    Ok(GameCircuitSpec {
        circuit_id: CircuitId::SealedBid,
        layout: layout.clone(),
        verification_key: pipeline_vk(env, CircuitId::SealedBid),
        params: CircuitParams::SealedBid(max_bid),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zk::ZKError;
    use soroban_sdk::BytesN;

    #[test]
    fn sealed_bid_rejects_zero_cap() {
        let env = Env::default();
        assert!(sealed_bid(&env, 0).is_err());
    }

    #[test]
    fn sealed_bid_rejects_over_cap_reveal() {
        let env = Env::default();
        let spec = sealed_bid(&env, 1000).unwrap();
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
        let auction = BytesN::from_array(&env, &[4u8; 32]);
        let commit = BytesN::from_array(&env, &[5u8; 32]);

        assert_eq!(
            spec.verify_bid_reveal(&env, &proof, &auction, &commit, 1001),
            Err(ZKError::InvalidInput)
        );
    }
}
