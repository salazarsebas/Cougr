# Zero-Knowledge Integration & Protocol 25 X-Ray Research

> Deep research on adding native ZK integrations to the Cougr engine, with special attention to Stellar Protocol 25 (X-Ray).

---

## Table of Contents

1. [Stellar Protocol 25: X-Ray](#1-stellar-protocol-25-x-ray)
2. [Current State of ZK on Stellar](#2-current-state-of-zk-on-stellar)
3. [ZK in On-Chain Gaming](#3-zk-in-on-chain-gaming)
4. [ZK Integration Patterns for ECS](#4-zk-integration-patterns-for-ecs)
5. [Integration Proposal for Cougr](#5-integration-proposal-for-cougr)
6. [References](#6-references)

---

## 1. Stellar Protocol 25: X-Ray

### Overview

Protocol 25, codenamed **X-Ray**, is Stellar's latest protocol upgrade that brings zero-knowledge proof capabilities to the network. It activated on **Testnet on January 7, 2026** and went live on **Mainnet on January 22, 2026**.

X-Ray introduces native host functions for **BN254 elliptic curve operations** and **Poseidon/Poseidon2 hash primitives** - the two fundamental building blocks required for most modern ZK proof systems.

### CAP-0074: BN254 Elliptic Curve Operations

BN254 is the most widely used pairing-friendly curve in today's ZK ecosystem. It powers applications like privacy pools, Lighter, portions of Starknet, and ZK Email. CAP-0074 adds three new host functions:

| Host Function | Purpose | Description |
|---------------|---------|-------------|
| `bn254_g1_add` | Group addition | Adds two points on the BN254 G1 curve |
| `bn254_g1_mul` | Scalar multiplication | Multiplies a G1 point by a scalar |
| `bn254_multi_pairing_check` | Pairing verification | Efficient multi-pairing check for proof verification |

These functions provide **feature parity with Ethereum's EIP-196 and EIP-197 precompiles**, enabling developers to easily migrate or extend existing ZK projects to Stellar. This is significant because it opens Stellar to the entire BN254-based ZK tooling ecosystem.

### CAP-0075: Poseidon/Poseidon2 Hash Primitives

Traditional hash functions like SHA-256 are computationally expensive to model within ZK circuits. Poseidon and Poseidon2 are hash families specifically designed for ZK proof systems - their operations are native to the prime fields used in ZK circuits.

CAP-0075 introduces **permutation primitives** rather than complete hash functions, allowing developers to construct sponge-mode hash functions with parameters matching their specific ZK system requirements.

#### Poseidon Permutation

```
poseidon_permutation(
    input: VecObject,              // Field elements vector, length = t
    field: U32Val,                 // 0 = BLS12-381 Fr, 1 = BN254 Fr
    t: U32Val,                     // State size
    d: U32Val,                     // S-box degree (typically 5)
    rounds_f: U32Val,              // Full rounds (must be even)
    rounds_p: U32Val,              // Partial rounds
    mds: VecObject,                // t x t MDS matrix as Vec<Vec<Scalar>>
    round_constants: VecObject     // (rounds_f + rounds_p) x t constants
) -> VecObject                     // Output vector after permutation
```

#### Poseidon2 Permutation

```
poseidon2_permutation(
    input: VecObject,              // Field elements vector, length = t
    field: U32Val,                 // 0 = BLS12-381 Fr, 1 = BN254 Fr
    t: U32Val,                     // State size
    d: U32Val,                     // S-box degree (3, 5, 7, or 11)
    rounds_f: U32Val,              // Full rounds (must be even)
    rounds_p: U32Val,              // Partial rounds
    mat_internal_diag_m_1: VecObject, // Internal matrix diagonal minus 1
    round_constants: VecObject     // (rounds_f + rounds_p) x t constants
) -> VecObject                     // Output vector after permutation
```

**Key difference**: Poseidon uses a traditional MDS matrix for all rounds, while Poseidon2 uses an optimized internal linear layer (`mat_internal_diag_m_1`) that reduces field multiplications in partial rounds, offering better performance.

#### Supported Scalar Fields

Both functions support two scalar fields:

| Field | ID | Scalar Field Order | Use Case |
|-------|-----|-------------------|----------|
| BLS12-381 Fr | 0 | `0x73eda753...00000001` | Gold-standard 128-bit security |
| BN254 Fr | 1 | `0x30644e72...f0000001` | Ethereum-compatible, most widely used |

#### HADES Design Strategy

Both Poseidon and Poseidon2 use the HADES permutation strategy:

- **Full Rounds**: Apply the S-box (`x^d`) to **all** `t` state elements
- **Partial Rounds**: Apply the S-box to **only one** state element
- **Total rounds**: `rounds_f + rounds_p`
- Full rounds provide diffusion; partial rounds provide efficiency

#### Error Handling

The host function traps (panics) if:
- Input vector length does not equal `t`
- MDS matrix dimensions are not `t x t` (Poseidon only)
- Internal diagonal length does not equal `t` (Poseidon2 only)
- Round constants matrix is not `(rounds_f + rounds_p) x t`
- Field parameter is not 0 or 1

#### Cost Model

New cost types introduced for BN254 operations: `Bn254FrToU256`, `Bn254FrAddSub`, `Bn254FrMul`, `Bn254FrPow`, `Bn254FrInv`. Cost scales **linearly** with rounds and **quadratically** with state size due to matrix operations.

### What X-Ray Enables

1. **Smooth migration** of existing ZK applications from Ethereum to Stellar
2. **Reduced proof generation constraints** through native host function performance
3. **Significantly lower costs** for ZK-based smart contracts
4. **Consistent hash logic** across on-chain and off-chain systems
5. **Gateway to the broader ZK ecosystem** tooling (Circom, Snarkjs, Arkworks)

---

## 2. Current State of ZK on Stellar

### Protocol Evolution

| Protocol | Capability Added | Security Level | Primary Use |
|----------|-----------------|----------------|-------------|
| Protocol 22 | BLS12-381 curve operations | 128-bit (gold standard) | Newer ZK systems, Zcash/Eth2.0 |
| Protocol 25 (X-Ray) | BN254 + Poseidon/Poseidon2 | ~100-bit (legacy) | Ethereum ecosystem compatibility |

### BLS12-381 (Protocol 22)

Added in 2024, BLS12-381 is the modern, security-focused curve:
- **128-bit security** - designed for long-term safety
- Preferred by newer networks (Zcash, Ethereum 2.0)
- Supports Groth16 proof verification natively on Stellar
- Larger field elements than BN254

### BN254 (Protocol 25)

The legacy but most widely-adopted curve:
- **~100-bit security** - lightweight and computationally cheaper
- Used by the vast majority of existing ZK applications
- Feature parity with Ethereum's precompiles (EIP-196/197)
- Now fully supported on Stellar with X-Ray

### Groth16 Verification

Groth16 is the most practical and widely-deployed zk-SNARK proof system:

- **On BLS12-381**: Fully supported since Protocol 22
- **On BN254**: Now possible with Protocol 25 X-Ray
- **Architecture**: Off-chain proof generation → on-chain verification via host functions
- **Cost**: Pairing check is the most expensive operation, handled by native host functions

### Poseidon vs SHA-256 in ZK Circuits

| Property | SHA-256 | Poseidon |
|----------|---------|----------|
| **ZK Circuit Cost** | Very expensive (~25,000 constraints) | Very cheap (~300 constraints) |
| **Design Purpose** | General-purpose hashing | ZK-optimized hashing |
| **Field Compatibility** | Operates on bits | Native to prime fields |
| **Performance in Proofs** | Slow proof generation | Fast proof generation |
| **On-Chain Cost** | Cheap (native host function) | Cheap (native host function with X-Ray) |

### Feature Parity with Ethereum

With Protocol 25, Stellar now has functional parity with Ethereum's ZK precompiles:

| Ethereum Precompile | Stellar Equivalent |
|---------------------|-------------------|
| EIP-196 (BN254 add/mul) | `bn254_g1_add`, `bn254_g1_mul` |
| EIP-197 (BN254 pairing) | `bn254_multi_pairing_check` |
| BLS12-381 precompile | Protocol 22 host functions |
| N/A (no native Poseidon) | `poseidon_permutation`, `poseidon2_permutation` |

Stellar actually **exceeds** Ethereum in Poseidon support - Ethereum doesn't have native Poseidon precompiles, requiring expensive in-contract computation.

---

## 3. ZK in On-Chain Gaming

### The Core Problem

On-chain games face a fundamental tension: **all state is public**. Every player's position, inventory, strategy, and resources are visible to everyone on the blockchain. This eliminates a core element of game design: **hidden information**.

Zero-knowledge proofs solve this by allowing players to prove that their actions are valid without revealing the underlying private state.

### Fog of War

The most iconic use case for ZK in gaming. Regions of the game map are obscured until explored by the player.

**Architecture:**
```
Player knows:     [own position, explored tiles, hidden units]
On-chain stores:  [hash(player_state), public_score, turn_count]
Player proves:    "My move is valid given my hidden state"
                  without revealing positions or unit locations
```

**Reference implementation: Dark Forest** - the first fully on-chain MMO real-time strategy game using zk-SNARKs for fog of war. Players submit proofs that their coordinates are valid against a public hash, without disclosing fleet positions.

### Hidden Information Games

Essential for card games, poker, strategy games, and any game with incomplete information:

- **Card games**: Players prove their hand is valid without revealing cards
- **Poker**: Prove bet is legal given hidden hand
- **Strategy**: Hidden unit compositions verified through proofs
- **Auctions**: Sealed bid verified without revealing amount until resolution

**Proof pattern:**
1. Player commits to hidden state (hash)
2. Player performs action
3. Player generates ZK proof that action is valid given committed state
4. Contract verifies proof and updates public game state
5. Hidden state remains private

### Verifiable Randomness

Generating provably fair random numbers for:
- **Loot boxes**: Contents determined fairly, verifiably
- **Dice rolls**: No manipulation possible
- **Event spawning**: Random encounters verified
- **Shuffling**: Card decks shuffled verifiably

**Commit-reveal with ZK:**
1. Player commits to a random seed (hash)
2. Contract provides its own randomness source
3. Combined randomness generates outcome
4. ZK proof verifies the combination was done correctly

### Anti-Cheat Mechanisms

ZK proofs as cryptographic enforcement of game rules:

- **Movement validation**: Prove new position is within valid bounds and speed limits
- **Damage calculation**: Prove combat results follow game formulas
- **Resource management**: Prove inventory changes are valid (no duplication)
- **Action timing**: Prove actions occurred in valid sequence

**Architecture:**
```
Player performs computation locally
    ↓
Generates ZK proof that computation follows game rules
    ↓
Submits proof + public outputs to smart contract
    ↓
Contract verifies proof (cheap, constant cost)
    ↓
State update applied only if proof is valid
```

### Private State Management

Players maintain encrypted or hidden state locally:

- **Private inventories**: Only reveal items when trading
- **Hidden economies**: Strategy game resource production hidden
- **Secret alliances**: Prove alliance membership without revealing partners
- **Stealth mechanics**: Invisible units with ZK-verified positions

### Commit-Reveal for Turn-Based Games

Essential pattern for simultaneous turns:

1. **Commit phase**: All players submit hash(action + salt)
2. **Reveal phase**: Players reveal action + salt
3. **Verify phase**: ZK proof that reveal matches commitment
4. **Execute phase**: Actions applied simultaneously

This prevents the "last mover advantage" where the last player to submit can see others' actions.

---

## 4. ZK Integration Patterns for ECS

### ZK Components

Zero-knowledge concepts map naturally to ECS components:

```rust
/// A component whose value is hidden - only a commitment is stored on-chain
pub struct ZKHiddenState {
    pub commitment: Hash<32>,     // Hash of the actual value
    pub last_proof_tick: u64,     // When the last valid proof was submitted
}

/// A commitment to a value that can be revealed later
pub struct ZKCommitment {
    pub hash: Hash<32>,           // Commitment hash
    pub committed_at: u64,        // Timestamp
    pub reveal_deadline: u64,     // When reveal must happen
}

/// Marks a component as requiring ZK proof for updates
pub struct ZKProofRequired {
    pub circuit_id: Symbol,       // Which circuit validates this
    pub verification_key: Bytes,  // Stored verification key
}

/// Stores a submitted proof awaiting verification
pub struct ZKProofSubmission {
    pub proof: Bytes,             // The proof data
    pub public_inputs: Vec<Bytes>,// Public inputs for verification
    pub verified: bool,           // Whether proof was verified
}
```

### ZK Systems

Verification as a step in the game loop:

```rust
/// System that verifies all pending ZK proofs
pub struct ZKVerificationSystem;

impl System for ZKVerificationSystem {
    type In = ();
    type Out = Vec<VerificationResult>;

    fn run(&mut self, world: &mut World, _input: ()) -> Vec<VerificationResult> {
        // Query all entities with pending proofs
        // Verify each proof using host functions
        // Update verification status
        // Allow/deny state transitions based on results
    }
}

/// System that manages commit-reveal phases
pub struct CommitRevealSystem;

/// System that updates hidden state based on verified proofs
pub struct HiddenStateUpdateSystem;
```

### Pre-Built Circuits for Common Game Patterns

#### Movement Validation Circuit

```
Public inputs:  [old_position_hash, new_position_hash, max_speed, game_map_hash]
Private inputs: [old_x, old_y, new_x, new_y]
Constraints:
  - hash(old_x, old_y) == old_position_hash
  - hash(new_x, new_y) == new_position_hash
  - distance(old, new) <= max_speed
  - new_position is within map bounds
  - new_position is not an obstacle
```

#### Combat Resolution Circuit

```
Public inputs:  [attacker_hash, defender_hash, damage_result, rng_seed]
Private inputs: [attacker_stats, defender_stats, attacker_buffs]
Constraints:
  - hash(attacker_stats) == attacker_hash
  - damage = formula(attacker_stats, defender_stats, rng_seed)
  - damage_result == damage
  - all stats within valid ranges
```

#### Inventory Verification Circuit

```
Public inputs:  [old_inventory_hash, new_inventory_hash, transaction_type]
Private inputs: [old_items[], new_items[], transaction_details]
Constraints:
  - hash(old_items) == old_inventory_hash
  - hash(new_items) == new_inventory_hash
  - conservation: total_value(old) == total_value(new) (no duplication)
  - all item counts >= 0
  - transaction follows game rules
```

#### Turn Sequencing Circuit

```
Public inputs:  [previous_state_hash, new_state_hash, turn_number]
Private inputs: [player_actions[], game_state]
Constraints:
  - hash(game_state_before) == previous_state_hash
  - all actions are valid for the current turn
  - actions applied in correct order
  - hash(game_state_after) == new_state_hash
```

### Off-Chain Proof Generation + On-Chain Verification

The practical architecture for ZK gaming:

```
┌─────────────────────────────────────┐
│           CLIENT SIDE               │
│                                     │
│  Game State (private) ──────────┐   │
│          │                      │   │
│          ▼                      ▼   │
│  Game Logic Engine    Circuit Compiler│
│          │                      │   │
│          ▼                      ▼   │
│  Action Decision ──→ Proof Generator │
│                              │      │
│                              ▼      │
│                    ZK Proof + Public │
│                    Inputs           │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│         ON-CHAIN (Soroban)          │
│                                     │
│  Proof + Public Inputs              │
│          │                          │
│          ▼                          │
│  bn254_multi_pairing_check()        │
│  (or BLS12-381 verification)        │
│          │                          │
│          ▼                          │
│  Valid? ──→ Update Game State       │
│         ──→ Reject Transaction      │
└─────────────────────────────────────┘
```

### Developer-Friendly Abstractions

```rust
/// High-level trait for game circuits
pub trait GameCircuit {
    type PublicInput;
    type PrivateInput;
    type Output;

    /// Generate a proof off-chain
    fn prove(
        public: &Self::PublicInput,
        private: &Self::PrivateInput,
    ) -> Result<Proof, CircuitError>;

    /// Verify a proof on-chain
    fn verify(
        env: &Env,
        proof: &Proof,
        public_input: &Self::PublicInput,
    ) -> Result<Self::Output, CircuitError>;
}

/// Pre-built movement validation
pub struct MovementCircuit;

impl GameCircuit for MovementCircuit {
    type PublicInput = MovementPublicInput;
    type PrivateInput = MovementPrivateInput;
    type Output = bool;
    // ...
}
```

---

## 5. Integration Proposal for Cougr

### Module Architecture: `src/zk/`

ZK functionality lives as an internal module within the single `cougr-core` crate. This is a **major expansion** of the crate - adding a complete zero-knowledge proof layer with cryptographic primitives, pre-built game circuits, verification systems, and ECS-integrated components. The developer experience stays simple: one dependency, one import path, no workspace juggling.

```
src/zk/
├── mod.rs                  // Module entry point, re-exports all public API
├── traits.rs               // GameCircuit, Verifiable, Committable traits
├── error.rs                // ZKError variants (InvalidProof, CircuitMismatch, etc.)
├── types.rs                // G1Point, G2Point, Scalar, FieldElement, Proof, VerificationKey
├── components/
│   ├── mod.rs              // Re-exports ZK components
│   ├── hidden_state.rs     // ZKHiddenState - commitment-based hidden values
│   ├── commitment.rs       // ZKCommitment - commit-reveal lifecycle
│   ├── proof.rs            // ZKProofRequired, ZKProofSubmission
│   └── verifiable.rs       // ZKVerified marker component (attached after proof passes)
├── systems/
│   ├── mod.rs              // Re-exports ZK systems
│   ├── verification.rs     // ZKVerificationSystem - batch proof verification per tick
│   ├── commit_reveal.rs    // CommitRevealSystem - phase management and deadline enforcement
│   ├── hidden_update.rs    // HiddenStateUpdateSystem - update commitments after verified proofs
│   └── proof_cleanup.rs    // ProofCleanupSystem - remove processed proof submissions
├── circuits/
│   ├── mod.rs              // Re-exports pre-built circuits
│   ├── movement.rs         // Movement validation (position bounds, speed limits, obstacles)
│   ├── combat.rs           // Combat resolution (damage formulas, stat verification)
│   ├── inventory.rs        // Inventory verification (conservation, no duplication)
│   ├── turn.rs             // Turn sequencing (valid order, state transitions)
│   └── custom.rs           // CustomCircuit - developer-defined circuit adapter
├── crypto/
│   ├── mod.rs              // Re-exports crypto wrappers
│   ├── bn254.rs            // BN254 G1 add/mul/pairing - ergonomic wrappers over host functions
│   ├── bls12_381.rs        // BLS12-381 operations - wrappers over Protocol 22 host functions
│   ├── poseidon.rs         // Poseidon hash construction (sponge mode, BN254/BLS12-381 fields)
│   ├── poseidon2.rs        // Poseidon2 hash construction (optimized internal matrix)
│   ├── groth16.rs          // Groth16 verifier (BN254 + BLS12-381)
│   └── commitment.rs       // Commitment schemes (Pedersen, hash-based)
├── merkle/
│   ├── mod.rs              // Merkle tree utilities for fog of war, inventory proofs
│   ├── tree.rs             // Poseidon-based Merkle tree construction
│   ├── proof.rs            // Merkle inclusion/exclusion proofs
│   └── sparse.rs           // Sparse Merkle tree for large state spaces
└── testing.rs              // MockProof, MockCircuit, test_verification_key for unit testing
```

This adds **~25 new source files** to `cougr-core`, representing a complete ZK layer that makes Cougr the first ECS game engine with native zero-knowledge proof support. Developers access everything through `cougr_core::zk::*` - no extra crate to add, no version mismatches to manage.

### ZK Components as First-Class Citizens

ZK-enabled components should integrate seamlessly with the existing ECS:

```rust
// A hidden position component
pub struct HiddenPosition {
    pub commitment: Hash<32>,  // Public: hash of actual position
    // Actual x, y stored only client-side
}

// Usage in a game
let entity = world.spawn_entity();
world.add_component(entity, HiddenPosition {
    commitment: poseidon_hash(x, y),
});

// Player submits move with proof
world.submit_proof(entity, MovementCircuit, proof, public_inputs);

// Verification system processes proofs
zk_verification_system.run(&mut world);
```

### Crypto Wrappers

Ergonomic wrappers around Soroban host functions:

```rust
/// BN254 operations wrapper
pub mod bn254 {
    /// Add two G1 points
    pub fn g1_add(env: &Env, p1: &G1Point, p2: &G1Point) -> G1Point;

    /// Scalar multiply a G1 point
    pub fn g1_mul(env: &Env, point: &G1Point, scalar: &Scalar) -> G1Point;

    /// Multi-pairing check (returns true if valid)
    pub fn pairing_check(env: &Env, pairs: &[(G1Point, G2Point)]) -> bool;
}

/// Poseidon hash construction
pub mod poseidon {
    /// Hash two field elements using Poseidon
    pub fn hash2(env: &Env, a: &FieldElement, b: &FieldElement) -> FieldElement;

    /// Hash a variable number of field elements
    pub fn hash_many(env: &Env, inputs: &[FieldElement]) -> FieldElement;

    /// Poseidon with BN254 scalar field
    pub fn hash_bn254(env: &Env, inputs: &[FieldElement]) -> FieldElement;

    /// Poseidon with BLS12-381 scalar field
    pub fn hash_bls12(env: &Env, inputs: &[FieldElement]) -> FieldElement;
}

/// Groth16 proof verifier
pub mod groth16 {
    /// Verify a Groth16 proof
    pub fn verify(
        env: &Env,
        verification_key: &VerificationKey,
        proof: &Proof,
        public_inputs: &[FieldElement],
    ) -> bool;
}
```

### Merkle Tree Utilities

Built-in Merkle tree support for common ZK gaming patterns (fog of war, inventory proofs, state commitments):

```rust
/// Poseidon-based Merkle tree optimized for ZK circuits
pub struct PoseidonMerkleTree {
    leaves: Vec<FieldElement>,
    root: FieldElement,
    depth: u32,
}

impl PoseidonMerkleTree {
    /// Create a new tree from leaf values
    pub fn new(env: &Env, leaves: Vec<FieldElement>) -> Self;

    /// Get the Merkle root (stored on-chain as commitment)
    pub fn root(&self) -> FieldElement;

    /// Generate an inclusion proof for a leaf
    pub fn prove_inclusion(&self, leaf_index: u32) -> MerkleProof;

    /// Verify an inclusion proof on-chain
    pub fn verify_inclusion(
        env: &Env, root: &FieldElement, leaf: &FieldElement, proof: &MerkleProof,
    ) -> bool;

    /// Update a leaf and return the new root + update proof
    pub fn update_leaf(
        &mut self, env: &Env, index: u32, new_value: FieldElement,
    ) -> (FieldElement, MerkleProof);
}

/// Sparse Merkle tree for large state spaces (e.g., game maps)
pub struct SparseMerkleTree {
    // Efficient for maps where most tiles are empty/default
}
```

### Testing Utilities

```rust
/// Mock proof that always passes verification - for unit testing game logic
pub struct MockProof;

/// Mock circuit for testing without actual ZK computation
pub struct MockCircuit;

impl GameCircuit for MockCircuit {
    type PublicInput = Vec<FieldElement>;
    type PrivateInput = Vec<FieldElement>;
    type Output = bool;

    fn prove(_public: &Self::PublicInput, _private: &Self::PrivateInput) -> Result<Proof, ZKError> {
        Ok(MockProof::valid())
    }

    fn verify(_env: &Env, _proof: &Proof, _public: &Self::PublicInput) -> Result<bool, ZKError> {
        Ok(true)  // Always passes in test mode
    }
}

/// Generate a test verification key for a circuit
pub fn test_verification_key() -> VerificationKey;

// Usage in tests:
#[test]
fn test_hidden_movement() {
    let world = test_world_with_zk();
    let proof = MockProof::valid();
    world.submit_proof(entity, MockCircuit, proof, public_inputs).unwrap();
    zk_verification_system.run(&mut world);
    assert!(world.has_component::<ZKVerified>(entity));
}
```

### Example: Card Game with Hidden Hands

```rust
// Components
pub struct HiddenHand {
    pub commitment: Hash<32>,  // hash(cards)
    pub card_count: u32,       // Public: how many cards
}

pub struct PlayedCard {
    pub card_value: u32,       // Revealed when played
    pub proof: Bytes,          // Proof card was in hand
}

// System: Verify card play
pub fn verify_card_play(world: &mut World, env: &Env) {
    // For each entity with a PlayedCard this tick:
    // 1. Get the player's HiddenHand commitment
    // 2. Verify the ZK proof that the played card was in the hand
    // 3. Update the hand commitment to reflect removal
    // 4. Apply game effects of the played card
}
```

### Example: Strategy Game with Fog of War

```rust
// Components
pub struct HiddenUnitPosition {
    pub commitment: Hash<32>,  // hash(x, y, unit_type)
}

pub struct VisibilityRange {
    pub range: u32,            // How far this entity can see
}

pub struct ExploredTiles {
    pub merkle_root: Hash<32>, // Merkle tree of explored tiles
}

// System: Process movement with fog of war
pub fn process_hidden_movement(world: &mut World, env: &Env) {
    // 1. Player submits new position commitment + proof
    // 2. Verify movement distance is valid
    // 3. Update position commitment
    // 4. Reveal tiles within visibility range
    // 5. Check for combat with revealed enemy units
}
```

### Implementation Roadmap

**Phase 1 - Foundation (Immediate)**
- Wrap BN254 and Poseidon host functions in ergonomic Rust API
- Implement Groth16 verifier using host functions
- Create basic ZK component types
- Example: simple commit-reveal game

**Phase 2 - Game Circuits (Short-term)**
- Build pre-built circuits for movement, combat, inventory
- Create client-side proof generation SDK
- Implement ZKVerificationSystem
- Example: card game with hidden hands

**Phase 3 - Advanced Patterns (Medium-term)**
- Fog of war system with Merkle tree exploration
- Multi-player ZK state channels
- Recursive proof composition for complex game logic
- Example: strategy game with fog of war

**Phase 4 - Ecosystem Integration (Long-term)**
- Integration with Circom/Snarkjs for circuit development
- Integration with Arkworks for Rust-native circuits
- Circuit compiler for game rules → ZK circuits
- Developer SDK for custom circuit development

---

## 6. References

- [Stellar Protocol 25 X-Ray Announcement](https://stellar.org/blog/developers/announcing-stellar-x-ray-protocol-25)
- [Stellar X-Ray Protocol 25 Upgrade Guide](https://stellar.org/blog/developers/stellar-x-ray-protocol-25-upgrade-guide)
- [CAP-0074: BN254 Elliptic Curve Operations](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0074.md)
- [CAP-0075: Poseidon/Poseidon2 Hash Function Primitives](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0075.md)
- [Stellar Discussion: Cryptographic Primitives for Proof Verification](https://github.com/stellar/stellar-protocol/discussions/1500)
- [Deep Dive: ZKP on Stellar (Rumble Fish)](https://www.rumblefish.dev/blog/post/zkp-on-stellar/)
- [Stellar: Prototyping Privacy Pools on Stellar](https://stellar.org/blog/ecosystem/prototyping-privacy-pools-on-stellar)
- [BLS12-381 Support in Soroban (Issue #779)](https://github.com/stellar/rs-soroban-env/issues/779)
- [Stellar Software Versions](https://developers.stellar.org/docs/networks/software-versions)
- [Zero-Knowledge Proofs in Gaming (TokenMinds)](https://tokenminds.co/blog/knowledge-base/zero-knowledge-proofs-in-gaming)
- [Dark Forest: ZK Adventures (HackerNoon)](https://hackernoon.com/zero-knowledge-proofs-adventures-in-the-dark-forest-jij3xyn)
- [Incomplete Information Games on Bitcoin (sCrypt)](https://scryptplatform.medium.com/incomplete-information-games-on-bitcoin-d79408050882)
