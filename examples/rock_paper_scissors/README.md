# Rock Paper Scissors with Commit-Reveal

A two-player Rock Paper Scissors game demonstrating the **commit-reveal pattern** using cryptographic hashing on Stellar Soroban. This is the simplest example of zero-knowledge cryptography in the Cougr framework.

## What is Commit-Reveal?

The commit-reveal pattern solves a fundamental problem in blockchain games: **how can players make simultaneous secret choices when all transactions are public?**

### The Problem

In a naive implementation:
1. Player A submits "Rock" → visible on-chain
2. Player B sees A's choice, submits "Paper" → B always wins!

### The Solution: Commit-Reveal

```
COMMIT PHASE (hide choices)
├─ Player A: hash(Rock + secret_salt) → 0x7a3f...
├─ Player B: hash(Paper + secret_salt) → 0x9b2e...
└─ Both hashes stored on-chain (choices hidden)

REVEAL PHASE (prove choices)
├─ Player A: reveals (Rock, secret_salt)
├─ Contract: verify hash(Rock + secret_salt) == 0x7a3f... ✓
├─ Player B: reveals (Paper, secret_salt)
├─ Contract: verify hash(Paper + secret_salt) == 0x9b2e... ✓
└─ Contract: compare choices → B wins!
```

**Key Properties:**
- ✅ **Binding**: Can't change choice after committing (hash locks it in)
- ✅ **Hiding**: Opponent can't see choice until reveal
- ✅ **Order-independent**: Neither player gains advantage by going first/second

## Game Flow

### 1. Initialize Match
```rust
new_match(player_a, player_b, best_of: 3)
```
Creates a best-of-N match (1, 3, 5, etc.)

### 2. Commit Phase
Both players compute and submit hashes:

```rust
// Off-chain: Player A
let salt = random_32_bytes();
let hash = sha256(choice || salt);

// On-chain
commit(player_a, hash)
```

When both players commit → automatically transitions to Reveal phase

### 3. Reveal Phase
Players reveal their choices with salts:

```rust
reveal(player_a, choice, salt)
```

Contract verifies: `sha256(choice || salt) == stored_hash`

When both players reveal → automatically resolves round

### 4. Resolution
```
Rock > Scissors
Scissors > Paper  
Paper > Rock
Same choice = Draw
```

Updates scoreboard, checks if match winner determined (best-of-N), or starts next round.

### 5. Timeout Protection
If a player refuses to reveal after committing:

```rust
claim_timeout(honest_player)
```

After 100 ledgers, the honest player who revealed wins by forfeit.

## Contract API

| Function | Parameters | Description |
|----------|-----------|-------------|
| `new_match` | `player_a: Address`<br>`player_b: Address`<br>`best_of: u32` | Initialize new match |
| `commit` | `player: Address`<br>`hash: BytesN<32>` | Submit commitment hash |
| `reveal` | `player: Address`<br>`choice: u32`<br>`salt: BytesN<32>` | Reveal choice (0=Rock, 1=Paper, 2=Scissors) |
| `claim_timeout` | `player: Address` | Claim win if opponent doesn't reveal |
| `get_state` | - | Get current match state |
| `get_score` | - | Get scoreboard |

## Data Structures

### Choice
```rust
enum Choice {
    Rock = 0,
    Paper = 1,
    Scissors = 2,
}
```

### Phase
```rust
enum Phase {
    Committing,  // Waiting for both commitments
    Revealing,   // Waiting for both reveals
    Resolved,    // Match complete
}
```

### MatchState
```rust
struct MatchState {
    phase: Phase,
    winner: Option<Address>,
    round: u32,
}
```

### ScoreBoard
```rust
struct ScoreBoard {
    wins_a: u32,
    wins_b: u32,
    draws: u32,
    best_of: u32,
}
```

## Building & Testing

### Prerequisites
- Rust 1.70.0+
- Stellar CLI 25.0.0+ (optional)

```bash
cargo install stellar-cli
```

### Build
```bash
# Development build
cargo build

# Optimized WASM
cargo build --release --target wasm32v1-none
```

### Test
```bash
cargo test
```

**Test Coverage (15 tests):**
- ✅ Match initialization
- ✅ Commit phase transitions
- ✅ All 9 choice combinations (RR, RP, RS, PR, PP, PS, SR, SP, SS)
- ✅ Hash mismatch rejection
- ✅ Best-of-3 match flow
- ✅ Double commit prevention
- ✅ Premature reveal prevention
- ✅ Component trait serialization

## Example Usage

### Off-Chain (Player)
```rust
use soroban_sdk::{Bytes, BytesN, Env};

// Generate random salt
let salt = BytesN::from_array(&env, &[42u8; 32]);

// Choose Rock (0)
let choice = 0u32;

// Compute commitment hash
let mut data = Bytes::new(&env);
data.append(&Bytes::from_array(&env, &choice.to_be_bytes()));
for i in 0..32 {
    data.push_back(salt.get(i).unwrap());
}
let hash = env.crypto().sha256(&data);

// Submit commitment
client.commit(&player, &hash.into());

// Later, reveal
client.reveal(&player, &choice, &salt);
```

### On-Chain (Contract)
```rust
// Verify hash matches
let computed = sha256(choice || salt);
if computed != stored_hash {
    panic!("Hash mismatch");
}

// Resolve round
if choice_a.beats(choice_b) {
    wins_a += 1;
}
```

## Why SHA256 Instead of Poseidon2?

This example uses SHA256 for simplicity and immediate usability. Poseidon2 is a ZK-friendly hash function that's more efficient in zero-knowledge circuits (~300 constraints vs SHA256's ~28,000), but requires the `hazmat-crypto` feature flag.

**For production ZK applications**, use Poseidon2:
```rust
use cougr_core::privacy::experimental::poseidon2_hash;

let hash = poseidon2_hash(&env, &params, &choice_u256, &salt_u256);
```

**For this educational example**, SHA256 is:
- ✅ Built into Soroban SDK
- ✅ No feature flags needed
- ✅ Demonstrates commit-reveal pattern clearly
- ✅ Cryptographically secure for this use case

## Security Considerations

### Secure
- **Commitment binding**: Hash function is collision-resistant
- **Choice hiding**: Preimage resistance prevents guessing
- **Replay protection**: Each round requires new commitments
- **Timeout protection**: Prevents griefing by non-revealing players

### ️ Important
- **Salt randomness**: Use cryptographically secure random salts (32 bytes)
- **Salt uniqueness**: Never reuse salts across rounds
- **Timeout value**: 100 ledgers (~8 minutes on Stellar) - adjust for your needs

### Best Practices
```rust
// ✅ Good: Random salt per round
let salt = generate_random_bytes(32);

// ❌ Bad: Predictable salt
let salt = BytesN::from_array(&env, &[0u8; 32]);

// ❌ Bad: Reused salt
let salt = player_address.to_bytes();
```

## ECS Architecture

### Components

| Component | Fields | Purpose |
|-----------|--------|---------|
| `PlayerCommitment` | `hash: BytesN<32>`<br>`revealed: bool` | Stores commitment hash |
| `MatchState` | `phase: Phase`<br>`winner: Option<Address>`<br>`round: u32` | Game phase tracking |
| `ScoreBoard` | `wins_a: u32`<br>`wins_b: u32`<br>`draws: u32`<br>`best_of: u32` | Match scoring |

All components implement `cougr_core::component::ComponentTrait` for type-safe serialization.

### Systems

| System | Responsibility |
|--------|---------------|
| **CommitSystem** | Accepts hashes, transitions to reveal when both committed |
| **RevealSystem** | Verifies `sha256(choice || salt) == hash`, rejects mismatches |
| **ResolveSystem** | Compares choices, updates scoreboard |
| **MatchSystem** | Checks best-of-N threshold, declares winner or starts next round |

## Deployment

### Deploy to Testnet
```bash
# Generate funded account
stellar keys generate rps-deployer --network <NETWORK> --fund

# Build contract
cargo build --release --target wasm32v1-none

# Deploy
stellar contract deploy \
  --wasm target/wasm32v1-none/release/rock_paper_scissors.wasm \
  --source rps-deployer \
  --network <NETWORK>
```

### Play a Game
```bash
CONTRACT_ID=<your_contract_id>

# Initialize match
stellar contract invoke \
  --id $CONTRACT_ID \
  --network <NETWORK> \
  -- new_match \
  --player_a <PLAYER_A_ADDRESS> \
  --player_b <PLAYER_B_ADDRESS> \
  --best_of 3

# Player A commits (compute hash off-chain first)
stellar contract invoke \
  --id $CONTRACT_ID \
  --network <NETWORK> \
  --source player-a \
  -- commit \
  --player <PLAYER_A_ADDRESS> \
  --hash <HASH_BYTES>

# Player B commits
stellar contract invoke \
  --id $CONTRACT_ID \
  --network <NETWORK> \
  --source player-b \
  -- commit \
  --player <PLAYER_B_ADDRESS> \
  --hash <HASH_BYTES>

# Player A reveals
stellar contract invoke \
  --id $CONTRACT_ID \
  --network <NETWORK> \
  --source player-a \
  -- reveal \
  --player <PLAYER_A_ADDRESS> \
  --choice 0 \
  --salt <SALT_BYTES>

# Player B reveals
stellar contract invoke \
  --id $CONTRACT_ID \
  --network <NETWORK> \
  --source player-b \
  -- reveal \
  --player <PLAYER_B_ADDRESS> \
  --choice 1 \
  --salt <SALT_BYTES>

# Check results
stellar contract invoke \
  --id $CONTRACT_ID \
  --network <NETWORK> \
  -- get_score
```

## Learning Path

This example is the **entry point** for understanding Cougr's cryptographic primitives:

1. **Start here**: Commit-reveal with SHA256 (this example)
2. **Next**: Upgrade to Poseidon2 hashing (ZK-friendly)
3. **Advanced**: Full ZK proofs with circuits (see `examples/chess/`)

## Resources

- [Cougr Repository](https://github.com/salazarsebas/Cougr)
- [Commit-Reveal Schemes](https://en.wikipedia.org/wiki/Commitment_scheme)
- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)
- [Poseidon Hash Function](https://www.poseidon-hash.info/)

## License

MIT OR Apache-2.0
