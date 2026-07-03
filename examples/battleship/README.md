# Battleship with Hidden Board

A two-player Battleship game demonstrating **hidden information** using commit-reveal pattern and Merkle proofs on Stellar Soroban. Players commit their board layouts cryptographically, then prove hit/miss results without revealing unattacked positions.

This example is Cougr's **canonical** hidden-information reference. It intentionally leans on the stable privacy surface in `cougr_core::privacy::stable` instead of re-defining Merkle verification inside the example.

## Status

**Canonical** — maintained reference implementation for commit-reveal + selective disclosure on Soroban. Uses `cougr-core = "1.1.0"`, `privacy::stable` Merkle primitives, and `impl_component!` macros for standardized serialization.

## The Hidden Information Problem

Traditional on-chain games face a challenge: **all data is public**. In Battleship, if boards are stored directly on-chain, opponents can see ship positions and cheat.

### Solution: Commit-Reveal + Merkle Proofs

```
SETUP PHASE (hide boards)
├─ Player A: commitment = SHA256(board || salt)
├─ Player A: merkle_root = MerkleTree(board).root
├─ Submit (commitment, merkle_root) on-chain
└─ Player B: same process

ATTACK PHASE (selective reveal)
├─ Attacker: attack(x, y)
├─ Defender: reveal_cell(x, y, value, merkle_proof)
├─ Contract: verify proof against merkle_root
└─ Record hit/miss (other cells remain hidden)

END PHASE (anti-cheat)
├─ Winner declared when all ships sunk
└─ Full board can be verified against commitment
```

**Key Properties:**
- ✅ **Hiding**: Unattacked cells remain secret
- ✅ **Binding**: Can't change board after commitment
- ✅ **Selective Reveal**: Prove one cell without revealing others
- ✅ **Verifiable**: Merkle proofs ensure honesty

## Game Flow

### 1. Setup Phase
```rust
new_game(player_a, player_b)
```

Each player computes off-chain:
```rust
// 1. Create 10x10 board (0=water, 1=ship)
let board = [0u32; 100];
board[0] = 1; // Ship at (0,0)

// 2. Compute commitment
let commitment = SHA256(board || salt);

// 3. Build Merkle tree
let merkle_root = MerkleTree::new(board).root();

// 4. Submit on-chain
commit_board(player, commitment, merkle_root)
```

### 2. Attack Phase
Alternating turns:

**Attacker:**
```rust
attack(attacker, x, y)
```

**Defender:**
```rust
// Off-chain: get Merkle proof for cell (x,y)
let proof = merkle_tree.get_proof(x, y);

// On-chain: reveal with proof
reveal_cell(defender, x, y, value, proof)
```

Contract verifies:
1. Proof is valid against stored `merkle_root`
2. Records hit (value=1) or miss (value=0)
3. Updates ship count
4. Switches turn

### 3. Win Condition
Game ends when one player's ships are all sunk (17 hits total: 5+4+3+3+2).

## Contract API

| Function | Parameters | Description |
|----------|-----------|-------------|
| `new_game` | `player_a: Address`<br>`player_b: Address` | Initialize game |
| `commit_board` | `player: Address`<br>`commitment: BytesN<32>`<br>`merkle_root: BytesN<32>` | Commit board layout |
| `attack` | `attacker: Address`<br>`x: u32, y: u32` | Attack coordinates (0-9) |
| `reveal_cell` | `defender: Address`<br>`x: u32, y: u32`<br>`value: u32`<br>`proof: OnChainMerkleProof` | Reveal cell with Merkle proof |
| `get_state` | - | Get current game state |

## Data Structures

### Phase
```rust
enum Phase {
    Setup,    // Waiting for board commitments
    Attack,   // Game in progress
    Finished, // Winner declared
}
```

### CellResult
```rust
enum CellResult {
    Unknown, // Not yet attacked
    Miss,    // Attacked, no ship
    Hit,     // Attacked, ship present
}
```

### BoardCommitment
```rust
struct BoardCommitment {
    commitment: BytesN<32>,  // SHA256(board || salt)
    merkle_root: BytesN<32>, // Root of Merkle tree
}
```

## Stable Merkle Verification

The contract uses Cougr's stable SHA256 Merkle proof contract:

```rust
use cougr_core::privacy::stable::{MerkleProofVerifier, Sha256MerkleProofVerifier};

let verifier = Sha256MerkleProofVerifier;
assert!(verifier.verify(&env, &proof, &merkle_root)?);
```

The leaf payload still binds `index || value`, but the inclusion proof format and verification rules come from Cougr's stable privacy API.

## When to Use Commit-Reveal vs ZK Circuits

| Pattern | Best For | Cost | Complexity |
|---------|----------|------|------------|
| **Commit-Reveal + Merkle** | Hidden boards, card hands, fog-of-war | O(log n) proof verification | Low — uses standard SHA256 |
| **ZK Circuits (Groth16/Poseidon)** | Private game logic evaluation, hidden card deals | Single on-chain verification | High — requires circuit compilation |

Use **commit-reveal** when you need to hide state but reveal it incrementally with verifiable proofs. Use **ZK circuits** when the game logic itself must remain private (e.g., proving a move is valid without revealing the move).

For a reference ZK implementation, see [hidden_hand](../hidden_hand/), which demonstrates circuit-based hidden card dealing with Groth16 proofs.

## Storage Model

### Committed State

Stored on-chain during setup:
- `commitment_a/b` — SHA256 hash of each player's board with random salt
- `merkle_root_a/b` — root of the Merkle tree built from cell hashes
- `has_commitment_a/b` — flags tracking which players have committed

Both commitments must be submitted before the game transitions to `Phase::Attack`.

### Revealed State

Updated during the attack phase:
- `attack_grid_a/b` — maps cell indices to `CellResult` (hit/miss)
- `ship_status` — tracks remaining ship cells per player (starts at 17)
- `turn_state` — tracks current player, phase, and pending reveals

### Proven State

The `reveal_cell` function verifies that disclosed cell values match the original commitment:
1. Constructs the expected leaf hash from `(index, value)` using the same SHA256 scheme as the commit phase
2. Validates the `OnChainMerkleProof` against the stored `merkle_root` using `Sha256MerkleProofVerifier`
3. Only if the proof verifies is the hit/miss recorded on-chain

## Component Serialization

`BoardCommitment` uses the `impl_component!` macro for standardized serialization:

```rust
impl_component!(BoardCommitment, "board", Table, {
    commitment: bytes32,
    merkle_root: bytes32
});
```

This replaces manual byte-level serialization with a type-safe macro that handles big-endian encoding/decoding automatically.

## Building & Testing

### Prerequisites
- Rust 1.70.0+
- Stellar CLI 25.0.0+ (optional)

### Build
```bash
cargo build
cargo build --release --target wasm32v1-none
```

### Test
```bash
cargo test
```


**Test Coverage (12 tests):**

**Recommended Testing Approach:**
For verification of complex setup commitment sequences and attack/reveal turn-based interactions, utilize `GameHarness` and `Scenario` (see [sandbox_tests.rs](src/sandbox_tests.rs)). This ensures cryptographic commitments and Merkle proofs verify correctly across turns.

**Test Coverage (10 tests):**

- ✅ Game initialization
- ✅ Board commitment
- ✅ Attack and reveal (miss)
- ✅ Attack and reveal (hit)
- ✅ Invalid proof rejection
- ✅ Cannot attack same cell twice
- ✅ Turn enforcement
- ✅ Win condition
- ✅ Component trait serialization (via `impl_component!`)
- ✅ Turn switching
- ✅ Reveal without pending attack
- ✅ Attack before commit phase

## Example Usage

### Off-Chain (Player)
```rust
// Build Merkle tree from hashed cell payloads
let mut leaves = Vec::new();
for (idx, &value) in board.iter().enumerate() {
    let leaf = sha256(idx || value);
    leaves.push(leaf);
}
let tree = MerkleTree::from_leaves(&env, &leaves)?;
let root = tree.root();

// Get proof for specific cell
let proof = to_on_chain_proof(&tree.proof(x * 10 + y)?, &env);

// Submit on-chain
client.reveal_cell(&player, &x, &y, &value, &proof);
```

### On-Chain (Contract)
```rust
let verifier = Sha256MerkleProofVerifier;
assert!(verifier.verify(&env, &proof, &stored_merkle_root)?);
```

## Security Considerations

### Secure
- **Commitment binding**: SHA256 prevents changing board
- **Selective reveal**: Merkle proofs reveal only attacked cells
- **Proof verification**: Invalid proofs rejected
- **Turn enforcement**: Players alternate attacks

### ️ Important
- **Salt randomness**: Use 32 cryptographically random bytes
- **Merkle tree depth**: 7 levels for 100 cells (padded to 128)
- **Proof ordering**: Siblings must be in correct order

### Best Practices
```rust
// ✅ Good: Random salt
let salt = generate_random_bytes(32);

// ❌ Bad: Predictable salt
let salt = BytesN::from_array(&env, &[0u8; 32]);

// ✅ Good: Verify proof before revealing
if !verify_proof(root, index, value, proof) {
    panic!("Invalid proof");
}
```

## ECS Architecture

### Components

| Component | Fields | Purpose |
|-----------|--------|---------|
| `BoardCommitment` | `commitment: BytesN<32>`<br>`merkle_root: BytesN<32>` | Cryptographic board commitment |
| `AttackGrid` | `cells: Map<u32, CellResult>` | Public record of attacks |
| `ShipStatus` | `remaining_a: u32`<br>`remaining_b: u32` | Ship cell counts |
| `TurnState` | `current_player: Address`<br>`phase: Phase`<br>`has_pending: bool` | Game state management |

### Systems

| System | Responsibility |
|--------|---------------|
| **CommitSystem** | Validates and stores commitments |
| **AttackSystem** | Records attack coordinates |
| **RevealSystem** | Verifies stable `OnChainMerkleProof`, updates grid |
| **WinConditionSystem** | Detects when all ships sunk |

## Why Merkle Proofs?

Merkle trees enable **selective disclosure**:

| Approach | Reveal Cost | Privacy |
|----------|------------|---------|
| **Full board on-chain** | O(1) | ❌ None |
| **Reveal entire board per attack** | O(n) | ❌ None |
| **Merkle proof** | O(log n) | ✅ Only attacked cells |

For a 10x10 board:
- Full reveal: 100 cells
- Merkle proof: ~7 hashes (log₂ 128)

## Deployment

```bash
# Deploy to testnet
stellar keys generate battleship-deployer --network <NETWORK> --fund
stellar contract deploy \
  --wasm target/wasm32v1-none/release/battleship.wasm \
  --source battleship-deployer \
  --network <NETWORK>
```

## Resources

- [Cougr Repository](https://github.com/salazarsebas/Cougr)
- [Merkle Trees](https://en.wikipedia.org/wiki/Merkle_tree)
- [Commitment Schemes](https://en.wikipedia.org/wiki/Commitment_scheme)
- [hidden_hand — ZK circuit example](../hidden_hand/)
- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)

## License

MIT OR Apache-2.0
