use soroban_sdk::{contracttype, symbol_short, BytesN, Env, Symbol, Vec};

use crate::zk::experimental::{
    bytes32_to_scalar, field_u32_to_scalar, verify_groth16, FogOfWarCircuit, FogOfWarSnapshot,
    FogOfWarTransition,
};
use crate::zk::types::{Groth16Proof, Scalar, VerificationKey};
use crate::zk::ZKError;

/// Frozen identifier for a pre-built game circuit family.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CircuitId {
    HiddenCards = 1,
    FogOfWar = 2,
    FairDice = 3,
    SealedBid = 4,
}

/// One slot in a frozen public-input layout.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicInputSlot {
    pub name: Symbol,
    pub kind: Symbol,
}

/// Frozen public-input contract for a circuit family.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicInputLayout {
    pub slots: Vec<PublicInputSlot>,
}

impl PublicInputLayout {
    /// Number of public inputs the verifier expects.
    pub fn public_input_count(&self) -> u32 {
        self.slots.len()
    }

    /// Layout for [`CircuitId::HiddenCards`].
    pub fn hidden_cards(env: &Env) -> Self {
        Self {
            slots: Vec::from_array(
                env,
                [
                    PublicInputSlot {
                        name: symbol_short!("deck"),
                        kind: symbol_short!("bytes32"),
                    },
                    PublicInputSlot {
                        name: symbol_short!("hand"),
                        kind: symbol_short!("bytes32"),
                    },
                    PublicInputSlot {
                        name: symbol_short!("player"),
                        kind: symbol_short!("u32"),
                    },
                    PublicInputSlot {
                        name: symbol_short!("deck_sz"),
                        kind: symbol_short!("u32"),
                    },
                    PublicInputSlot {
                        name: symbol_short!("hand_sz"),
                        kind: symbol_short!("u32"),
                    },
                ],
            ),
        }
    }

    /// Layout for [`CircuitId::FogOfWar`] - matches [`FogOfWarCircuit::verify_exploration`].
    pub fn fog_of_war(env: &Env) -> Self {
        Self {
            slots: Vec::from_array(
                env,
                [
                    PublicInputSlot {
                        name: symbol_short!("map"),
                        kind: symbol_short!("bytes32"),
                    },
                    PublicInputSlot {
                        name: symbol_short!("prior"),
                        kind: symbol_short!("bytes32"),
                    },
                    PublicInputSlot {
                        name: symbol_short!("next"),
                        kind: symbol_short!("bytes32"),
                    },
                    PublicInputSlot {
                        name: symbol_short!("orig_x"),
                        kind: symbol_short!("i32"),
                    },
                    PublicInputSlot {
                        name: symbol_short!("orig_y"),
                        kind: symbol_short!("i32"),
                    },
                    PublicInputSlot {
                        name: symbol_short!("tile_x"),
                        kind: symbol_short!("i32"),
                    },
                    PublicInputSlot {
                        name: symbol_short!("tile_y"),
                        kind: symbol_short!("i32"),
                    },
                    PublicInputSlot {
                        name: symbol_short!("radius"),
                        kind: symbol_short!("u32"),
                    },
                ],
            ),
        }
    }

    /// Layout for [`CircuitId::FairDice`].
    pub fn fair_dice(env: &Env) -> Self {
        Self {
            slots: Vec::from_array(
                env,
                [
                    PublicInputSlot {
                        name: symbol_short!("seed"),
                        kind: symbol_short!("bytes32"),
                    },
                    PublicInputSlot {
                        name: symbol_short!("roll"),
                        kind: symbol_short!("u32"),
                    },
                    PublicInputSlot {
                        name: symbol_short!("sides"),
                        kind: symbol_short!("u32"),
                    },
                    PublicInputSlot {
                        name: symbol_short!("nonce"),
                        kind: symbol_short!("u32"),
                    },
                ],
            ),
        }
    }

    /// Layout for [`CircuitId::SealedBid`].
    pub fn sealed_bid(env: &Env) -> Self {
        Self {
            slots: Vec::from_array(
                env,
                [
                    PublicInputSlot {
                        name: symbol_short!("auction"),
                        kind: symbol_short!("bytes32"),
                    },
                    PublicInputSlot {
                        name: symbol_short!("commit"),
                        kind: symbol_short!("bytes32"),
                    },
                    PublicInputSlot {
                        name: symbol_short!("bid"),
                        kind: symbol_short!("u32"),
                    },
                    PublicInputSlot {
                        name: symbol_short!("max_bid"),
                        kind: symbol_short!("u32"),
                    },
                ],
            ),
        }
    }
}

/// Parameter bundle selected by a circuit builder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CircuitParams {
    HiddenCards(u32, u32),
    FogOfWar(u32, u32, u32),
    FairDice(u32, BytesN<32>),
    SealedBid(u32),
}

/// Pre-built game circuit descriptor with frozen layout and Groth16 verifier metadata.
#[derive(Clone, Debug)]
pub struct GameCircuitSpec {
    pub circuit_id: CircuitId,
    pub layout: PublicInputLayout,
    pub verification_key: VerificationKey,
    pub params: CircuitParams,
}

impl GameCircuitSpec {
    /// Replace the verification key with a production key from the Circom pipeline.
    pub fn with_verification_key(mut self, vk: VerificationKey) -> Self {
        self.verification_key = vk;
        self
    }

    /// Access the frozen public-input layout.
    pub fn public_input_layout(&self) -> &PublicInputLayout {
        &self.layout
    }

    /// Verify a proof against raw public inputs (layout must match `ic.len() - 1`).
    pub fn verify(
        &self,
        env: &Env,
        proof: &Groth16Proof,
        public_inputs: &[Scalar],
    ) -> Result<bool, ZKError> {
        if public_inputs.len() as u32 != self.layout.public_input_count() {
            return Err(ZKError::InvalidPublicInput);
        }
        let expected_ic = self.layout.public_input_count().saturating_add(1);
        if self.verification_key.ic.len() != expected_ic {
            return Err(ZKError::InvalidVerificationKey);
        }
        verify_groth16(env, &self.verification_key, proof, public_inputs)
    }

    /// Verify a hidden-card deal proof.
    pub fn verify_hidden_hand(
        &self,
        env: &Env,
        proof: &Groth16Proof,
        deck_root: &BytesN<32>,
        hand_commitment: &BytesN<32>,
        player_id: u32,
    ) -> Result<bool, ZKError> {
        let CircuitParams::HiddenCards(deck_size, hand_size) = self.params else {
            return Err(ZKError::CircuitMismatch);
        };

        let public_inputs = alloc::vec![
            bytes32_to_scalar(deck_root),
            bytes32_to_scalar(hand_commitment),
            field_u32_to_scalar(env, player_id),
            field_u32_to_scalar(env, deck_size),
            field_u32_to_scalar(env, hand_size),
        ];
        self.verify(env, proof, &public_inputs)
    }

    /// Verify a fog-of-war exploration proof (delegates to [`FogOfWarCircuit`]).
    pub fn verify_fog_exploration(
        &self,
        env: &Env,
        proof: &Groth16Proof,
        snapshot: &FogOfWarSnapshot,
        transition: &FogOfWarTransition,
    ) -> Result<bool, ZKError> {
        let CircuitParams::FogOfWar(_, _, visibility_radius) = self.params else {
            return Err(ZKError::CircuitMismatch);
        };
        let circuit = FogOfWarCircuit::new(self.verification_key.clone(), visibility_radius);
        circuit.verify_exploration(env, proof, snapshot, transition)
    }

    /// Verify a fair dice roll proof.
    pub fn verify_dice_roll(
        &self,
        env: &Env,
        proof: &Groth16Proof,
        roll_result: u32,
        nonce: u32,
    ) -> Result<bool, ZKError> {
        let CircuitParams::FairDice(sides, ref seed_commitment) = self.params else {
            return Err(ZKError::CircuitMismatch);
        };

        if roll_result == 0 || roll_result > sides {
            return Err(ZKError::InvalidInput);
        }

        let public_inputs = alloc::vec![
            bytes32_to_scalar(seed_commitment),
            field_u32_to_scalar(env, roll_result),
            field_u32_to_scalar(env, sides),
            field_u32_to_scalar(env, nonce),
        ];
        self.verify(env, proof, &public_inputs)
    }

    /// Verify a sealed-bid reveal proof.
    pub fn verify_bid_reveal(
        &self,
        env: &Env,
        proof: &Groth16Proof,
        auction_id: &BytesN<32>,
        bid_commitment: &BytesN<32>,
        revealed_bid: u32,
    ) -> Result<bool, ZKError> {
        let CircuitParams::SealedBid(max_bid) = self.params else {
            return Err(ZKError::CircuitMismatch);
        };

        if revealed_bid == 0 || revealed_bid > max_bid {
            return Err(ZKError::InvalidInput);
        }

        let public_inputs = alloc::vec![
            bytes32_to_scalar(auction_id),
            bytes32_to_scalar(bid_commitment),
            field_u32_to_scalar(env, revealed_bid),
            field_u32_to_scalar(env, max_bid),
        ];
        self.verify(env, proof, &public_inputs)
    }
}

/// Load the pipeline verification key for a circuit family.
pub(crate) fn pipeline_vk(env: &Env, circuit_id: CircuitId) -> VerificationKey {
    super::embedded::pipeline_verification_key(env, circuit_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zk::types::{G1Point, G2Point};

    #[test]
    fn pipeline_vk_matches_layout_ic_length() {
        let env = Env::default();
        let layout = PublicInputLayout::hidden_cards(&env);
        let vk = pipeline_vk(&env, CircuitId::HiddenCards);
        assert_eq!(vk.ic.len(), layout.public_input_count() + 1);
    }

    #[test]
    fn verify_rejects_wrong_public_input_count() {
        let env = Env::default();
        let layout = PublicInputLayout::fair_dice(&env);
        let spec = GameCircuitSpec {
            circuit_id: CircuitId::FairDice,
            layout,
            verification_key: pipeline_vk(&env, CircuitId::FairDice),
            params: CircuitParams::FairDice(6, BytesN::from_array(&env, &[1u8; 32])),
        };

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

        let inputs = alloc::vec![field_u32_to_scalar(&env, 1)];
        assert_eq!(
            spec.verify(&env, &proof, &inputs),
            Err(ZKError::InvalidPublicInput)
        );
    }
}
