use soroban_sdk::Env;

use crate::zk::ZKError;

use super::spec::{pipeline_vk, CircuitId, CircuitParams, GameCircuitSpec, PublicInputLayout};

/// Build a hidden-card deal circuit for `deck_size` cards and `hand_size` cards per player.
///
/// Public inputs: `[deck_root, hand_commitment, player_id, deck_size, hand_size]`.
pub fn hidden_cards(env: &Env, deck_size: u32, hand_size: u32) -> Result<GameCircuitSpec, ZKError> {
    if deck_size == 0 || hand_size == 0 || hand_size > deck_size {
        return Err(ZKError::InvalidInput);
    }

    let layout = PublicInputLayout::hidden_cards(env);
    Ok(GameCircuitSpec {
        circuit_id: CircuitId::HiddenCards,
        layout: layout.clone(),
        verification_key: pipeline_vk(env, CircuitId::HiddenCards),
        params: CircuitParams::HiddenCards(deck_size, hand_size),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuits::spec::CircuitParams;
    use crate::zk::types::{G1Point, G2Point, Groth16Proof};
    use soroban_sdk::BytesN;

    #[test]
    fn hidden_cards_rejects_invalid_sizes() {
        let env = Env::default();
        assert!(hidden_cards(&env, 0, 5).is_err());
        assert!(hidden_cards(&env, 52, 0).is_err());
        assert!(hidden_cards(&env, 5, 10).is_err());
    }

    #[test]
    fn hidden_cards_builds_expected_spec() {
        let env = Env::default();
        let spec = hidden_cards(&env, 52, 5).unwrap();
        assert_eq!(spec.circuit_id, CircuitId::HiddenCards);
        assert_eq!(spec.layout.public_input_count(), 5);
        assert_eq!(spec.params, CircuitParams::HiddenCards(52, 5));
    }

    #[test]
    fn hidden_cards_verify_encodes_public_inputs() {
        let env = Env::default();
        let spec = hidden_cards(&env, 52, 5).unwrap();
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

        let deck = BytesN::from_array(&env, &[2u8; 32]);
        let hand = BytesN::from_array(&env, &[3u8; 32]);
        // Encoding path smoke test (production uses VK from `bun run pipeline`).
        let _ = spec.verify_hidden_hand(&env, &proof, &deck, &hand, 1);
    }

    #[test]
    fn hidden_cards_supports_non_default_hand_size() {
        let env = Env::default();
        let spec = hidden_cards(&env, 52, 7).unwrap();
        assert_eq!(spec.circuit_id, CircuitId::HiddenCards);
        assert_eq!(spec.params, CircuitParams::HiddenCards(52, 7));

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

        let deck = BytesN::from_array(&env, &[2u8; 32]);
        let hand = BytesN::from_array(&env, &[3u8; 32]);
        let _ = spec.verify_hidden_hand(&env, &proof, &deck, &hand, 1);
    }
}
