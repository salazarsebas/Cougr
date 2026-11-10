use soroban_sdk::{BytesN, Env};

use super::error::ZKError;
use super::traits::{bytes32_to_scalar, i32_to_scalar, u32_to_scalar, GameCircuit};
use super::types::{Groth16Proof, Scalar, VerificationKey};

/// Movement verification circuit interface.
///
/// Verifies that a player's move is valid (within maximum allowed distance)
/// without revealing the full game state. The circuit's public inputs are:
/// `[from_x, from_y, to_x, to_y, max_distance]`.
///
/// # Example
/// ```no_run
/// use cougr_core::zk::{G1Point, G2Point};
/// use cougr_core::zk::experimental::{Groth16Proof, MovementCircuit, VerificationKey};
/// use soroban_sdk::{BytesN, Env, Vec};
///
/// let env = Env::default();
/// let g1 = G1Point { bytes: BytesN::from_array(&env, &[0u8; 64]) };
/// let g2 = G2Point { bytes: BytesN::from_array(&env, &[0u8; 128]) };
/// let mut ic = Vec::new(&env);
/// for _ in 0..6 {
///     ic.push_back(g1.clone());
/// }
/// let vk = VerificationKey {
///     alpha: g1.clone(),
///     beta: g2.clone(),
///     gamma: g2.clone(),
///     delta: g2,
///     ic,
/// };
/// let proof = Groth16Proof { a: g1.clone(), b: vk.beta.clone(), c: g1 };
/// let circuit = MovementCircuit::new(vk, 10);
/// let _valid = circuit.verify_move(&env, &proof, 0, 0, 3, 4)?;
/// # Ok::<(), cougr_core::zk::ZKError>(())
/// ```
pub struct MovementCircuit {
    pub vk: VerificationKey,
    pub max_distance: u32,
}

impl GameCircuit for MovementCircuit {
    fn verification_key(&self) -> &VerificationKey {
        &self.vk
    }
}

impl MovementCircuit {
    /// Create a new movement circuit with the given verification key and max distance.
    pub fn new(vk: VerificationKey, max_distance: u32) -> Self {
        Self { vk, max_distance }
    }

    /// Verify a move from (from_x, from_y) to (to_x, to_y).
    ///
    /// The proof must demonstrate that the move is within `max_distance`.
    /// Public inputs are encoded as: `[from_x, from_y, to_x, to_y, max_distance]`.
    pub fn verify_move(
        &self,
        env: &Env,
        proof: &Groth16Proof,
        from_x: i32,
        from_y: i32,
        to_x: i32,
        to_y: i32,
    ) -> Result<bool, ZKError> {
        let public_inputs = alloc::vec![
            i32_to_scalar(env, from_x),
            i32_to_scalar(env, from_y),
            i32_to_scalar(env, to_x),
            i32_to_scalar(env, to_y),
            u32_to_scalar(env, self.max_distance),
        ];
        self.verify_with_inputs(env, proof, &public_inputs)
    }
}

/// Combat verification circuit interface.
///
/// Verifies damage calculation without revealing hidden player stats.
/// Public inputs: `[attacker_commitment, defender_commitment, damage_result]`.
pub struct CombatCircuit {
    pub vk: VerificationKey,
}

impl GameCircuit for CombatCircuit {
    fn verification_key(&self) -> &VerificationKey {
        &self.vk
    }
}

impl CombatCircuit {
    /// Create a new combat circuit with the given verification key.
    pub fn new(vk: VerificationKey) -> Self {
        Self { vk }
    }

    /// Verify a damage calculation.
    ///
    /// The proof demonstrates that `damage_result` was correctly computed
    /// from the hidden stats of the attacker and defender.
    pub fn verify_damage(
        &self,
        env: &Env,
        proof: &Groth16Proof,
        attacker_commitment: &BytesN<32>,
        defender_commitment: &BytesN<32>,
        damage_result: u32,
    ) -> Result<bool, ZKError> {
        let public_inputs = alloc::vec![
            bytes32_to_scalar(attacker_commitment),
            bytes32_to_scalar(defender_commitment),
            u32_to_scalar(env, damage_result),
        ];
        self.verify_with_inputs(env, proof, &public_inputs)
    }
}

/// Inventory verification circuit interface.
///
/// Proves a player has a specific item without revealing the full inventory.
/// Public inputs: `[inventory_root, item_id]`.
pub struct InventoryCircuit {
    pub vk: VerificationKey,
}

impl GameCircuit for InventoryCircuit {
    fn verification_key(&self) -> &VerificationKey {
        &self.vk
    }
}

impl InventoryCircuit {
    /// Create a new inventory circuit with the given verification key.
    pub fn new(vk: VerificationKey) -> Self {
        Self { vk }
    }

    /// Verify that an inventory contains a specific item.
    ///
    /// The proof demonstrates knowledge of a Merkle path from the item
    /// to the inventory root.
    pub fn verify_has_item(
        &self,
        env: &Env,
        proof: &Groth16Proof,
        inventory_root: &BytesN<32>,
        item_id: u32,
    ) -> Result<bool, ZKError> {
        let public_inputs = alloc::vec![
            bytes32_to_scalar(inventory_root),
            u32_to_scalar(env, item_id),
        ];
        self.verify_with_inputs(env, proof, &public_inputs)
    }
}

/// Turn sequence verification circuit interface.
///
/// Proves a sequence of game actions was executed in valid order
/// with valid state transitions.
/// Public inputs: `[initial_state_hash, final_state_hash, action_count]`.
pub struct TurnSequenceCircuit {
    pub vk: VerificationKey,
}

impl GameCircuit for TurnSequenceCircuit {
    fn verification_key(&self) -> &VerificationKey {
        &self.vk
    }
}

impl TurnSequenceCircuit {
    /// Create a new turn sequence circuit with the given verification key.
    pub fn new(vk: VerificationKey) -> Self {
        Self { vk }
    }

    /// Verify a sequence of turns.
    pub fn verify_sequence(
        &self,
        env: &Env,
        proof: &Groth16Proof,
        initial_state: &BytesN<32>,
        final_state: &BytesN<32>,
        action_count: u32,
    ) -> Result<bool, ZKError> {
        let public_inputs = alloc::vec![
            bytes32_to_scalar(initial_state),
            bytes32_to_scalar(final_state),
            u32_to_scalar(env, action_count),
        ];
        self.verify_with_inputs(env, proof, &public_inputs)
    }
}

/// Developer-defined circuit that wraps a VK and pre-encoded public inputs.
///
/// Use this when you have a custom circuit not covered by the pre-built ones.
///
/// # Example
/// ```no_run
/// use cougr_core::zk::{G1Point, G2Point};
/// use cougr_core::zk::experimental::{
///     bytes32_to_scalar, u32_to_scalar, CustomCircuit, GameCircuit, Groth16Proof, VerificationKey,
/// };
/// use soroban_sdk::{BytesN, Env, Vec};
///
/// let env = Env::default();
/// let g1 = G1Point { bytes: BytesN::from_array(&env, &[0u8; 64]) };
/// let g2 = G2Point { bytes: BytesN::from_array(&env, &[0u8; 128]) };
/// let mut ic = Vec::new(&env);
/// for _ in 0..3 {
///     ic.push_back(g1.clone());
/// }
/// let vk = VerificationKey {
///     alpha: g1.clone(),
///     beta: g2.clone(),
///     gamma: g2.clone(),
///     delta: g2,
///     ic,
/// };
/// let root = BytesN::from_array(&env, &[9u8; 32]);
/// let inputs = vec![u32_to_scalar(&env, 42), bytes32_to_scalar(&root)];
/// let circuit = CustomCircuit::new(vk, inputs);
/// let proof = Groth16Proof { a: g1.clone(), b: circuit.verification_key().beta.clone(), c: g1 };
/// let _valid = circuit.verify_with_inputs(&env, &proof, circuit.public_inputs())?;
/// # Ok::<(), cougr_core::zk::ZKError>(())
/// ```
pub struct CustomCircuit {
    vk: VerificationKey,
    public_inputs: alloc::vec::Vec<Scalar>,
}

impl GameCircuit for CustomCircuit {
    fn verification_key(&self) -> &VerificationKey {
        &self.vk
    }
}

impl CustomCircuit {
    /// Create a custom circuit with pre-encoded public inputs.
    pub fn new(vk: VerificationKey, public_inputs: alloc::vec::Vec<Scalar>) -> Self {
        Self { vk, public_inputs }
    }

    /// Start building a custom circuit with a fluent API.
    pub fn builder(vk: VerificationKey) -> CustomCircuitBuilder {
        CustomCircuitBuilder {
            vk,
            inputs: alloc::vec::Vec::new(),
        }
    }

    /// Get the pre-encoded public inputs.
    pub fn public_inputs(&self) -> &[Scalar] {
        &self.public_inputs
    }

    /// Verify the proof using the stored public inputs.
    pub fn verify(&self, env: &Env, proof: &Groth16Proof) -> Result<bool, ZKError> {
        self.verify_with_inputs(env, proof, &self.public_inputs)
    }
}

/// Builder for constructing `CustomCircuit` public inputs fluently.
pub struct CustomCircuitBuilder {
    vk: VerificationKey,
    inputs: alloc::vec::Vec<Scalar>,
}

impl CustomCircuitBuilder {
    /// Add a raw scalar to the public inputs.
    pub fn add_scalar(mut self, scalar: Scalar) -> Self {
        self.inputs.push(scalar);
        self
    }

    /// Add a u32 value as a scalar.
    pub fn add_u32(mut self, env: &Env, val: u32) -> Self {
        self.inputs.push(u32_to_scalar(env, val));
        self
    }

    /// Add an i32 value as a scalar.
    pub fn add_i32(mut self, env: &Env, val: i32) -> Self {
        self.inputs.push(i32_to_scalar(env, val));
        self
    }

    /// Add a BytesN<32> as a scalar.
    pub fn add_bytes32(mut self, val: &BytesN<32>) -> Self {
        self.inputs.push(bytes32_to_scalar(val));
        self
    }

    /// Build the CustomCircuit.
    pub fn build(self) -> CustomCircuit {
        CustomCircuit {
            vk: self.vk,
            public_inputs: self.inputs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::traits;
    use super::*;
    use soroban_sdk::{BytesN, Env, Vec};

    use super::super::types::{G1Point, G2Point};

    fn make_vk(env: &Env, ic_count: u32) -> VerificationKey {
        let g1 = G1Point {
            bytes: BytesN::from_array(env, &[0u8; 64]),
        };
        let g2 = G2Point {
            bytes: BytesN::from_array(env, &[0u8; 128]),
        };
        let mut ic = Vec::new(env);
        for _ in 0..ic_count {
            ic.push_back(g1.clone());
        }
        VerificationKey {
            alpha: g1,
            beta: g2.clone(),
            gamma: g2.clone(),
            delta: g2,
            ic,
        }
    }

    #[test]
    fn test_movement_circuit_creation() {
        let env = Env::default();
        let vk = make_vk(&env, 6); // 5 public inputs + 1
        let circuit = MovementCircuit::new(vk, 10);
        assert_eq!(circuit.max_distance, 10);
    }

    #[test]
    fn test_movement_circuit_wrong_ic_length() {
        let env = Env::default();
        let vk = make_vk(&env, 1); // wrong: needs 6 for 5 inputs
        let circuit = MovementCircuit::new(vk, 10);

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

        let result = circuit.verify_move(&env, &proof, 0, 0, 3, 4);
        assert_eq!(result, Err(ZKError::InvalidVerificationKey));
    }

    #[test]
    fn test_combat_circuit_creation() {
        let env = Env::default();
        let vk = make_vk(&env, 4);
        let circuit = CombatCircuit::new(vk);
        assert_eq!(circuit.vk.ic.len(), 4);
    }

    #[test]
    fn test_inventory_circuit_creation() {
        let env = Env::default();
        let vk = make_vk(&env, 3);
        let circuit = InventoryCircuit::new(vk);
        assert_eq!(circuit.vk.ic.len(), 3);
    }

    #[test]
    fn test_turn_sequence_circuit_creation() {
        let env = Env::default();
        let vk = make_vk(&env, 4);
        let circuit = TurnSequenceCircuit::new(vk);
        assert_eq!(circuit.vk.ic.len(), 4);
    }

    #[test]
    fn test_scalar_encoding_u32() {
        let env = Env::default();
        let scalar = traits::u32_to_scalar(&env, 42);
        assert_eq!(scalar.bytes.len(), 32);
    }

    #[test]
    fn test_scalar_encoding_i32() {
        let env = Env::default();
        let scalar = traits::i32_to_scalar(&env, -1);
        assert_eq!(scalar.bytes.len(), 32);
    }

    #[test]
    fn test_game_circuit_trait_on_movement() {
        let env = Env::default();
        let vk = make_vk(&env, 1); // wrong IC for 5 inputs
        let circuit = MovementCircuit::new(vk, 10);

        // Verify GameCircuit trait methods work
        assert_eq!(circuit.verification_key().ic.len(), 1);

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

        // verify_with_inputs should fail with wrong IC length
        let inputs = alloc::vec![traits::u32_to_scalar(&env, 1)];
        let result = circuit.verify_with_inputs(&env, &proof, &inputs);
        assert_eq!(result, Err(ZKError::InvalidVerificationKey));
    }

    #[test]
    fn test_custom_circuit_creation() {
        let env = Env::default();
        let vk = make_vk(&env, 3); // 2 public inputs + 1
        let inputs = alloc::vec![
            traits::u32_to_scalar(&env, 10),
            traits::u32_to_scalar(&env, 20),
        ];
        let circuit = CustomCircuit::new(vk, inputs);
        assert_eq!(circuit.public_inputs().len(), 2);
    }

    #[test]
    fn test_custom_circuit_builder() {
        let env = Env::default();
        let vk = make_vk(&env, 4); // 3 public inputs + 1
        let root = BytesN::from_array(&env, &[0xABu8; 32]);

        let circuit = CustomCircuit::builder(vk)
            .add_u32(&env, 42)
            .add_i32(&env, -5)
            .add_bytes32(&root)
            .build();

        assert_eq!(circuit.public_inputs().len(), 3);
        assert_eq!(circuit.verification_key().ic.len(), 4);
    }

    #[test]
    fn test_custom_circuit_verify_wrong_ic() {
        let env = Env::default();
        let vk = make_vk(&env, 1); // wrong IC
        let circuit = CustomCircuit::builder(vk)
            .add_u32(&env, 42)
            .add_u32(&env, 99)
            .build();

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

        let result = circuit.verify(&env, &proof);
        assert_eq!(result, Err(ZKError::InvalidVerificationKey));
    }
}
