# Proof of Hunt (Stellar + Soroban)

A hidden-map treasure discovery game demonstrating **zero-knowledge proof-backed exploration** on Stellar Soroban. This example combines:

- Hidden world state committed off-chain via Merkle roots
- Proof-backed exploration using Groth16 verification and BN254 pairing checks
- Premium actions modeled for x402 settlement flows
- Nullifier-based replay prevention

## Status

**Transitional** — uses `cougr-core = "1.1.0"` and Stellar-zk style Groth16 verification. See [battleship](../battleship/) for the canonical commit-reveal reference, and [hidden_hand](../hidden_hand/) for the canonical ZK circuits reference.

## Why This Is Stellar-Specific

This example demonstrates three Stellar-native patterns working together:

1. Soroban contract state for deterministic gameplay and progression
2. Stellar-zk style Groth16 verifier flow on-chain using Soroban BN254 pairing checks
3. x402-style premium action credits represented as settled payment units before hint consumption

References:
- https://crates.io/crates/stellar-zk
- https://github.com/salazarsebas/stellar-zk
- https://developers.stellar.org/docs/build/apps/x402

## When to Use Commit-Reveal vs ZK Circuits

Use **commit-reveal with Merkle proofs** (e.g., [battleship](../battleship/), [treasure_hunt](../treasure_hunt/)) when you need to hide state and reveal it incrementally with verifiable proofs. This is simpler and sufficient for most hidden-information games.

Use **ZK circuits** (this example and [hidden_hand](../hidden_hand/)) when:
- The game logic itself must remain private (e.g., proving a valid move without revealing it)
- You need constant-size proofs regardless of map size
- You want nullifier-based replay prevention without exposing cell indices

## Hidden State and Proof Flow

### Off-chain committed data

The hidden map is represented off-chain and committed by root hash:
- Map commitment root (`BytesN<32>`)
- Map dimensions (`width`, `height`)
- Implicit treasure distribution encoded in proof public inputs

### What is proven on-chain

For each exploration:
- `(x, y)` belongs to a valid proof statement bound to the same commitment root
- Leaf + sibling path resolves to the committed root
- Groth16 proof verifies through BN254 pairing checks
- Nullifier has not been replayed

### Anti-cheat and privacy properties

- Players cannot claim arbitrary discoveries because proof public inputs are tied to coordinates and root
- Replay is blocked by nullifier storage
- Full hidden map remains off-chain; only commitment and selective proof metadata are revealed

## Contract API

### Public Functions

| Function | Parameters | Description |
|----------|-----------|-------------|
| `init_game` | `player: Address`, `map_commitment: BytesN<32>`, `width: u32`, `height: u32` | Initialize game with committed map |
| `explore` | `player: Address`, `x: u32`, `y: u32`, `proof: ProofInput` | Explore a cell with ZK proof |
| `purchase_hint` | `player: Address`, `hint_type: u32` | Buy hint (0) or scan (1) using credits |
| `get_state` | — | Return `GameState` |
| `is_finished` | — | Check if game is won or lost |
| `set_verification_key` | `owner: Address`, `vk_bytes: Bytes` | Set Groth16 verification key |
| `credit_x402_payment` | `owner: Address`, `player: Address`, `units: u32`, `receipt_hash: BytesN<32>` | Credit x402 payment units |

### Data Types

```rust
enum GameStatus { Active, Won, Lost }

struct ProofInput {
    proof: BytesN<256>,
    public_inputs: Bytes,
    nullifier: BytesN<32>,
    leaf_hash: BytesN<32>,
    sibling_hash: BytesN<32>,
    sibling_on_left: bool,
}

struct PlayerState {
    position_x: u32,
    position_y: u32,
    score: i128,
    health: u32,
    discoveries: u32,
}

struct GameState {
    player: Address,
    map_commitment: BytesN<32>,
    width: u32,
    height: u32,
    treasure_count: u32,
    discovered_cells: u32,
    status: GameStatus,
    player_state: PlayerState,
    hint_usage: HintUsage,
    x402_credits: u32,
}
```

## Storage Model

### Committed State

Stored on-chain during initialization:
- `map_commitment` — Merkle root of the hidden map
- Map dimensions (`width`, `height`)
- Derived `treasure_count = max(1, (width * height) / 8)`
- Verification key for Groth16 proofs

### Revealed State

Updated during exploration:
- `player_state` — position, score, health, discoveries
- `discovered_cells` — count of explored cells
- `explored_cell(cell_idx)` — persistent storage for replay prevention
- Game status (`Active`/`Won`/`Lost`)

### Proven State

The `explore` function performs multi-layer verification:
1. **Public input validation**: Checks that `(x, y)` in proof inputs match claimed coordinates
2. **Merkle path verification**: Reconstructs root from leaf + sibling path
3. **Groth16 verification**: Verifies BN254 pairing check via `env.crypto().bn254().pairing_check()`
4. **Nullifier check**: Rejects if nullifier was already used (replay prevention)

## x402 Premium Action Model

`purchase_hint(player, hint_type)` consumes pre-settled premium credits.

This maps to an x402 backend flow where a payment gateway verifies and settles payment off-chain, then credits the user in-contract via `credit_x402_payment(...)`.

- `hint_type = 0`: hint action (cost 1 credit)
- `hint_type = 1`: scan action (cost 2 credits)

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

**Test Coverage:**
- ✅ Game initialization
- ✅ Valid exploration proof acceptance
- ✅ Invalid proof rejection
- ✅ Progression updates after valid exploration
- ✅ Premium hint action flow (x402 credits)
- ✅ Game completion (win condition)
- ✅ Failure condition (health depletion)
- ✅ Nullifier replay prevention
- ✅ Coordinate bounds validation
- ✅ Timeout and claim logic

## Security Considerations

### ✅ Secure
- **Proof-backed exploration**: Cannot claim arbitrary discoveries without valid Groth16 proof
- **Nullifier replay prevention**: Each cell can only be explored once
- **BN254 pairing verification**: Cryptographic proof of correct computation
- **Public input binding**: Coordinates are bound to proof statement

### ⚠️ Important
- **Zero-proof test path**: CI tests use deterministic zero-proof path (`#[cfg(test)]`) — production requires real Groth16 proofs
- **Verification key**: Must be set correctly before exploration; invalid VK causes verification failures

## Deployment

```bash
stellar keys generate proof-of-hunt-deployer --network testnet --fund
stellar contract deploy \
  --wasm target/wasm32v1-none/release/proof_of_hunt.wasm \
  --source proof-of-hunt-deployer \
  --network testnet
```

## Resources

- [Cougr Repository](https://github.com/salazarsebas/Cougr)
- [stellar-zk](https://github.com/salazarsebas/stellar-zk)
- [battleship — canonical commit-reveal example](../battleship/)
- [hidden_hand — canonical ZK circuit example](../hidden_hand/)
- [Soroban BN254 Documentation](https://developers.stellar.org/docs/build/smart-contracts)

## License

MIT OR Apache-2.0
