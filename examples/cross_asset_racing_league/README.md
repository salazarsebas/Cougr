# Cross-Asset Racing League (Stellar + Soroban)

Cross-Asset Racing League is a multi-asset payment-driven racing game example for Soroban that combines:

- payment-gated boost mechanics enabled by cross-asset payment flows
- proof-backed race-state validation through stellar-zk
- league standings and progression tracking
- deterministic race result verification

This example is intentionally contract-only: no frontend, no real-time physics simulation, no multiplayer middleware.

## Why This Is Stellar-Specific

This example demonstrates three Stellar-native patterns working together:

1. **Soroban contract state** for deterministic race progression and league management.
2. **stellar-zk style Groth16 verification** on-chain using Soroban BN254 pairing checks for anti-cheat and race integrity.
3. **x402-style payment credits** represented as settled payment units tied to multi-asset payment flows.

References:

- https://crates.io/crates/stellar-zk
- https://github.com/salazarsebas/stellar-zk
- https://developers.stellar.org/docs/build/apps/x402

## Gameplay Flow

### 1. League Initialization

The owner initializes a racing league, establishing:

- Season ID and tracking metadata
- Initial contract state for race creation and standings

### 2. Race Creation

The owner creates a race instance with:

- Unique race ID
- Duration (in ledger blocks)
- Registration phase for player entry
- Support for up to 10 racers per race

### 3. Player Entry

Players join a race during the registration phase:

- Entrant cap enforced (max 10 racers)
- Vehicle state initialized for each player
- Replay protection via tracking entered players

### 4. Race Activation

Owner starts the race, transitioning it to the active phase:

- Entrants are locked in
- Payment-gated boosts can now be activated
- Race-state proofs can be submitted

### 5. Payment-Gated Boost Activation

Players consume pre-settled payment credits to activate boosts:

- **Standard Boost** (cost: 10 credits) → +10 speed
- **Premium Boost** (cost: 50 credits) → +30 speed
- **Legendary Boost** (cost: 200 credits) → +60 speed

Payment flow:

1. Off-chain settlement process (e.g., via x402 gateway) receives cross-asset payment
2. Owner calls `credit_payment(player, amount, receipt_hash)` with proof of settlement
3. Replay protection via receipt hash prevents double-crediting
4. Player activates boost, consuming credits in-contract

### 6. Proof-Backed Race Validation

Players submit Groth16 proofs to validate race integrity:

- Proof verifies correct race execution state
- Public inputs bound to race commitments
- Nullifier-based replay protection
- BN254 pairing checks ensure anti-cheat properties

### 7. Race Completion and Standings Update

Owner completes the race:

- Race transitions to completed phase
- Points awarded by position: 1st=10, 2nd=6, 3rd=3, rest=1
- League standings updated for each racer

## Architecture

### Components

| Component | Fields | Purpose |
|-----------|--------|---------|
| **LeagueComponent** | season_id, current_race_id, league_active | Tracks league state |
| **RaceComponent** | race_id, season_id, entrants_count, phase, duration | Tracks race instance |
| **VehicleStateComponent** | speed, boost_state_type, boost_active, penalty_count | Tracks racer state |
| **PaymentActionComponent** | player_credits | Tracks payment-gated access |
| **ProofStateComponent** | commitment, verification_status | Tracks proof validation |

### Systems

- **RaceEntrySystem**: Player registration and race entry
- **BoostSystem**: Payment-gated boost activation with credit consumption
- **ProofValidationSystem**: stellar-zk Groth16 proof verification
- **ResultResolutionSystem**: Race completion and scoring
- **StandingsSystem**: League standings and progression tracking

## stellar-zk Integration

### Proof-Backed Race Integrity

This contract uses stellar-zk for a real validation path:

1. **Race Commitment**: Off-chain race state committed via Merkle root
2. **Proof Generation**: Player proves valid race state against commitment
3. **On-Chain Verification**: BN254 pairing checks verify the Groth16 proof
4. **Nullifier Protection**: Prevents replay attacks using commitment hashing

### Proof Structure

```rust
pub struct ProofInput {
    pub proof: BytesN<256>,           // Groth16 proof (compressed)
    pub public_inputs: Bytes,          // Public inputs bound to race state
    pub commitment: BytesN<32>,        // Race state commitment root
    pub race_id: u32,                  // Associated race
    pub player_id: u32,                // Associated player
}
```

### Verification Pattern

```
1. Extract public inputs from proof
2. Verify proof structure and length
3. Hash commitment for nullifier uniqueness
4. Check nullifier not previously used (replay protection)
5. Perform BN254 pairing checks (Groth16 verifier)
6. Mark nullifier as consumed
```

## Contract API

### Initialization

```rust
fn init_league(env: Env, owner: Address)
```

Initializes the racing league. Requires owner authorization.

### Race Management

```rust
fn create_race(env: Env, owner: Address, duration: u32) -> u32
```

Creates a new race with specified duration. Returns race ID. Owner-only.

```rust
fn start_race(env: Env, owner: Address, race_id: u32)
```

Transitions race from registration to active phase. Owner-only.

```rust
fn complete_race(env: Env, owner: Address, race_id: u32)
```

Completes the race, updates standings, and awards points. Owner-only.

### Player Actions

```rust
fn enter_race(env: Env, player: Address, race_id: u32)
```

Player enters a race during registration phase. Requires player authorization.

```rust
fn activate_boost(env: Env, player: Address, race_id: u32, boost_type: u32)
```

Player activates a boost by spending payment credits. Boost types: 1=Standard, 2=Premium, 3=Legendary.

### Payment Integration

```rust
fn credit_payment(env: Env, owner: Address, player: Address, amount: u32, receipt_hash: BytesN<32>)
```

Owner credits player with payment settlement. Receipt hash prevents double-crediting. Owner-only.

```rust
fn get_player_credits(env: Env, player: Address) -> u32
```

Query player's available payment credits.

### Proof Validation

```rust
fn submit_race_proof(env: Env, player: Address, proof: ProofInput) -> bool
```

Player submits proof for race-state validation. Returns true if proof is valid. Implements replay protection via nullifier.

### Query Functions

```rust
fn get_game_state(env: Env) -> GameState
```

Returns current league state.

```rust
fn get_race(env: Env, race_id: u32) -> Race
```

Returns race details.

```rust
fn get_player_standing(env: Env, season_id: u32, player: Address) -> PlayerStanding
```

Returns player's season standings with points, completion count, and best finish.

## Payment Model and Cross-Asset Integration

This example models payment as abstract credited-unit settlement:

### Off-Chain Settlement (Illustrative)

1. Player initiates cross-asset trade (e.g., USDC → stablecoin or asset)
2. Payment gateway settles transaction
3. Settlement proof returned to owner with receipt hash

### On-Chain Credit

Owner calls `credit_payment(player, units, receipt_hash)`:

- Units represent settled payment value
- Receipt hash ensures idempotent settlement
- Credits are consumed by boost activation

### Multi-Asset Representation

In production, payment units could represent:

- USDC stablecoins
- Native Stellar lumens (XLM)
- Custom issued assets
- Cross-chain bridged assets

The contract treats all as abstract credits, allowing flexible payment backends.

## Testing

Run tests with:

```sh
cargo test
```

### Test Coverage

- League initialization
- Race creation and player entry
- Boost activation with various types
- Payment credit system
- Proof validation and replay protection
- Race completion and standings
- Multiple races and cumulative standings
- Error conditions and edge cases

## Building and Validation

### Build the contract

```sh
cargo build --release
```

### Format and lint

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

### Run tests

```sh
cargo test
```

### Build to WASM

```sh
stellar contract build
```

## Design Decisions

### Why Abstract Payments?

The contract doesn't directly integrate with specific payment backends (Stripe, PayPal), but instead accepts settled credits. This allows flexibility:

- The same contract can work with different payment gateways
- Payment logic can be upgraded off-chain without contract changes
- Anti-censorship: payment flow doesn't depend on specific custodian

### Why Proof-Based Anti-Cheat?

Groth16 proofs enable:

- **Hidden tuning**: Players prove valid race parameters without revealing them
- **Deterministic fairness**: Proof ties race outcome to cryptographic commitment
- **Privacy**: Only commitment revealed on-chain, detailed race state stays off-chain

### Why Ledger-Based Timing?

Race duration uses ledger sequence numbers, not wall-clock time:

- Stellar ledgers close ~every 5 seconds (deterministic)
- No oracle dependency for timing
- Replay protection via sequence-based race ID

## Files

- `src/lib.rs`: Main contract implementation
- `src/test.rs`: Comprehensive test suite
- `README.md`: This file
- `Cargo.toml`: Soroban SDK dependencies

## Compatibility

- Soroban SDK: 25.1.0
- Edition: 2021
- Target: `wasm32v1-none`

## License

MIT OR Apache-2.0
