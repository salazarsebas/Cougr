# Verifiable Chess with ZK Move Validation

A simplified chess implementation on Stellar Soroban that demonstrates **zero-knowledge proof verification** for move legality using the **Cougr-Core** ZK framework. Move validation happens off-chain via circuits, and only compact Groth16 proofs are verified on-chain.

## Why ZK Proofs for Chess?

Traditional on-chain chess requires the contract to validate every move rule:
- Pawn forward movement and capture diagonals
- Knight L-shaped moves
- Bishop diagonal paths
- Rook straight paths
- Queen combined movement
- King single-step movement
- Check and checkmate detection
- Castling, en passant, promotion rules

**This is expensive.** Each move requires complex on-chain computation that scales with rule complexity.

### The ZK Approach

| Aspect | Traditional On-Chain | ZK Proof Verification |
|--------|---------------------|----------------------|
| **Move Validation** | Contract checks all rules | Off-chain circuit proves legality |
| **On-Chain Cost** | O(rule complexity) | O(1) - constant proof verification |
| **Gas Usage** | High, varies per piece | Low, fixed per move |
| **Extensibility** | Requires contract upgrade | Update circuit off-chain |
| **Privacy** | All moves public | Can hide move details (future) |

### Architecture Pattern

```text
Off-Chain (Player)                    On-Chain (Contract)
─────────────────────                 ───────────────────

1. Generate move                      
   (e4 → e5)                          

2. Prove legality                     
   Circuit validates:                 
   - Piece can move this way          
   - Path is clear                    
   - Move doesn't leave king in check 
   - Current board state matches      

3. Generate Groth16 proof             
   Public inputs:                     
   - state_hash (current board)       
   - from (e4)                        
   - to (e5)                          
                                      4. Verify proof
                                         ✓ Proof valid?
                                         ✓ State hash matches?
                                         ✓ Player's turn?

                                      5. Apply move
                                         - Update board
                                         - Compute new state_hash
                                         - Switch turn
```

## Game Flow

### 1. INIT
```rust
new_game(white: Address, black: Address)
```
- Initializes standard chess board
- Computes initial state hash (Poseidon2)
- Sets white as current player

### 2. MOVE (per turn)
```rust
submit_move(player: Address, from: u8, to: u8, proof: Bytes) -> MoveResult
```

**Off-chain (player):**
1. Generate move (e.g., pawn from 12 → 20)
2. Build circuit with current `state_hash`, `from`, `to`
3. Prove move is legal given board state
4. Submit proof to contract

**On-chain (contract):**
1. **TurnSystem**: Validate player and turn order
2. **ProofVerificationSystem**: Verify Groth16 proof
   - Uses `GameCircuit` trait
   - Public inputs: `[state_hash, from, to]`
3. **BoardUpdateSystem**: Apply move if proof valid
4. **TurnSystem**: Switch to next player
5. **EndGameSystem**: Check for king capture (simplified checkmate)

### 3. END
```rust
resign(player: Address)
```
- Player can resign at any time
- Checkmate detected when king is captured

## ECS Architecture

### Components

| Component | Fields | Purpose |
|-----------|--------|---------|
| `BoardState` | `state_hash: BytesN<32>`<br>`pieces: Map<u8, Piece>` | Current board position<br>Hash for proof binding<br>Map for display |
| `Piece` | `kind: enum {King, Queen, Rook, Bishop, Knight, Pawn}`<br>`color: enum {White, Black}` | Piece type and ownership |
| `TurnState` | `current: Address`<br>`move_count: u32`<br>`status: enum {Playing, Checkmate, Draw, Resigned}` | Turn tracking and game status |
| `ProofRecord` | `last_proof: Bytes`<br>`verified: bool` | Audit trail of last proof |

All components implement `cougr_core::component::ComponentTrait` for type-safe serialization.

### Systems

| System | Responsibility |
|--------|---------------|
| **ProofVerificationSystem** | Verifies Groth16 proof using `GameCircuit` trait<br>Validates move against current `state_hash` |
| **BoardUpdateSystem** | Applies verified move to board<br>Computes new `state_hash` (SHA-256) |
| **TurnSystem** | Enforces alternating turns<br>Validates player identity |
| **EndGameSystem** | Detects king capture (simplified checkmate)<br>Accepts resignation |

## ZK Circuit Design

### Move Validation Circuit

Uses `cougr_core::privacy::experimental::CustomCircuitBuilder`:

```rust
let circuit = CustomCircuit::builder(vk)
    .add_bytes32(&state_hash)  // Current board state
    .add_u32(&env, from as u32) // Source position
    .add_u32(&env, to as u32)   // Destination position
    .build();

let valid = circuit.verify(&env, &proof)?;
```

**Public Inputs:**
1. `state_hash` - Binds proof to specific board state (prevents replay)
2. `from` - Source square (0-63)
3. `to` - Destination square (0-63)

**Private Inputs (in circuit, not on-chain):**
- Full board state
- Piece being moved
- Piece type-specific movement rules
- Path obstruction checks

### Circuit Rules (Simplified Subset)

| Piece | Movement Rule | Circuit Constraint |
|-------|--------------|-------------------|
| **Pawn** | Forward 1 (or 2 from start) | `to = from + 8` (white) or `from - 8` (black)<br>No piece at destination |
| **Knight** | L-shape (2+1 or 1+2) | `abs(to_row - from_row) * abs(to_col - from_col) = 2`<br>Can jump over pieces |
| **Rook** | Straight lines | `to_row = from_row` OR `to_col = from_col`<br>Path clear |
| **Bishop** | Diagonals | `abs(to_row - from_row) = abs(to_col - from_col)`<br>Path clear |
| **Queen** | Rook + Bishop | Combined constraints |
| **King** | One square any direction | `max(abs(to_row - from_row), abs(to_col - from_col)) = 1` |

**Not implemented (future extensions):**
- Castling
- En passant
- Pawn promotion
- Check/checkmate validation (currently simplified to king capture)

## Contract API

### Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `new_game` | `white: Address`<br>`black: Address` | - | Initialize new game |
| `submit_move` | `player: Address`<br>`from: u8`<br>`to: u8`<br>`proof: Bytes` | `MoveResult` | Submit move with ZK proof |
| `resign` | `player: Address` | - | Resign the game |
| `get_board` | - | `BoardState` | Get current board state |
| `get_state` | - | `GameState` | Get full game state |
| `set_vk` | `vk: VerificationKey` | - | Set verification key (admin) |

### Board Positions

```text
56 57 58 59 60 61 62 63  ← Black pieces
48 49 50 51 52 53 54 55  ← Black pawns
...
 8  9 10 11 12 13 14 15  ← White pawns
 0  1  2  3  4  5  6  7  ← White pieces
```

### Move Results

```rust
pub enum MoveResult {
    Success,        // Move applied, proof valid
    InvalidProof,   // ZK proof verification failed
    WrongTurn,      // Not player's turn
    GameOver,       // Game already ended
}
```

## Building & Testing

### Prerequisites

| Requirement | Version |
|------------|---------|
| Rust | 1.70.0+ |
| Stellar CLI | 25.0.0+ |

```bash
cargo install stellar-cli
```

### Build

```bash
# Development build
cargo build

# Optimized WASM
stellar contract build
```

### Test

```bash
cargo test
```

**Test Coverage:**

| Category | Tests | Coverage |
|----------|-------|----------|
| Initialization | 3 | Game setup, board layout, state hashing |
| Turn Management | 3 | Turn switching, wrong turn rejection, move counting |
| Piece Movement | 3 | Pawn, knight, rook movement simulation |
| Endgame | 3 | Resignation, checkmate detection, post-game moves |
| Components | 2 | ComponentTrait serialization |
| Validation | 2 | Uninitialized game, proof verification |
| **Total** | **16** | **All passing** |

## Key Constraints

### Off-Chain Legality, On-Chain Verification

The contract **never** checks if a knight can move in an L-shape or if a path is clear. It only verifies the ZK proof that says "this move is legal given the current board state."

### State Hashing

After each move, the board state is hashed (SHA-256). The proof ties the move to this specific hash, preventing:
- **Replay attacks**: Old proofs can't be reused
- **State manipulation**: Proof only valid for exact board state
- **Parallel game attacks**: Each game has unique state progression

### Simplified Rules

This implementation demonstrates the ZK verification pattern with basic movement:
- ✅ Basic piece movement (6 types)
- ✅ Turn enforcement
- ✅ King capture detection
- ❌ Castling (future)
- ❌ En passant (future)
- ❌ Pawn promotion (future)
- ❌ Full check/checkmate (future)

These can be added by extending the circuit without changing the contract.

## Deployment

### Deploy to Testnet

```bash
# Generate funded account
stellar keys generate chess-deployer --network <NETWORK> --fund

# Build contract
stellar contract build

# Deploy
stellar contract deploy \
  --wasm target/wasm32v1-none/release/chess.wasm \
  --source chess-deployer \
  --network <NETWORK>
```

### Set Verification Key

```bash
# Generate VK off-chain (using your circuit compiler)
# Then set it on-chain:

stellar contract invoke \
  --id <CONTRACT_ID> \
  --network <NETWORK> \
  -- set_vk \
  --vk <SERIALIZED_VK>
```

### Play a Game

```bash
# Initialize game
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network <NETWORK> \
  -- new_game \
  --white <WHITE_ADDRESS> \
  --black <BLACK_ADDRESS>

# Submit move (white's turn)
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network <NETWORK> \
  --source white-player \
  -- submit_move \
  --player <WHITE_ADDRESS> \
  --from 12 \
  --to 20 \
  --proof <GROTH16_PROOF_BYTES>

# Get game state
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network <NETWORK> \
  -- get_state
```

## Why This Matters for Complex Games

### Scalability

Traditional on-chain games hit limits quickly:
- **Chess**: 64 squares, 6 piece types, complex rules
- **Go**: 361 squares, ko rule, territory calculation
- **Magic: The Gathering**: Thousands of cards, stack resolution

ZK proofs make these feasible:
- **Constant verification cost** regardless of rule complexity
- **Off-chain computation** scales with player hardware, not blockchain
- **Privacy potential**: Hide moves until reveal phase

### Extensibility

Add new rules without contract upgrades:
1. Update circuit off-chain
2. Generate new verification key
3. Call `set_vk()` on existing contract
4. Players use new circuit for proofs

No redeployment, no migration, no downtime.

### Future Enhancements

- **Fog of war**: Prove valid move without revealing destination
- **Simultaneous turns**: Commit moves, reveal with proofs
- **AI opponents**: Prove AI made legal move without revealing strategy
- **Tournament brackets**: Prove game outcome without replaying all moves

## Resources

- [Cougr Repository](https://github.com/salazarsebas/Cougr)
- [GameCircuit Trait](https://github.com/salazarsebas/Cougr/blob/main/src/zk/traits.rs)
- [Circuit Implementations](https://github.com/salazarsebas/Cougr/blob/main/src/zk/circuits.rs)
- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)
- [Groth16 Paper](https://eprint.iacr.org/2016/260.pdf)

## License

MIT OR Apache-2.0
