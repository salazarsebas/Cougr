# AI Dungeon Master Arena Example

> **Transitional example**: This example uses an older Cougr pattern and is preserved
> for compatibility reference. For the current recommended approach, see [`snake`](../snake/).

This example demonstrates a complex gameplay loop on Stellar/Soroban using:
1.  **x402-style Paid Actions**: Premium encounters or rerolls purchased in-game via a request-driven model.
2.  **stellar-zk (via Cougr-Core)**: Proof-backed validation of sensitive encounter progression and reward eligibility.

## Features

- **Run Initialization**: Start an entity-backed arena run for a player.
- **Turn-based Combat**: Attack or defend in procedural encounters.
- **x402 Hooks**: Premium rerolls or hints appearing at specific progression points.
- **ZK Verification**: Submit Groth16 proofs to validate off-chain state or hidden attributes on-chain.
- **Reward System**: Track and claim rewards earned from verified milestones.

## Architecture

The example follows an ECS-like structure (Entity Component System):
- **RunStateComponent**: Core session data.
- **EncounterComponent**: Current enemy and challenge data.
- **PremiumActionComponent**: Tracks availability of x402-driven premium options.
- **ProofStateComponent**: Stores verification status for ZK-backed progression.
- **RewardComponent**: Management of in-game assets and rewards.

## stellar-zk Integration

This contract uses `cougr-core::zk` which is built for high-performance on-chain verification compatible with `stellar-zk` generated circuits.
- The `verify_run_proof` function takes a `Groth16Proof` and public inputs.
- It validates the proof against a stored `VerificationKey` (which would correspond to the circuit built for your arena logic).
- Successful validation unlocks pending rewards and confirms floor progression.

## x402 Flow

1. Player completes a floor.
2. At every 3rd floor, a `PremiumActionComponent` is attached to their run.
3. This signals an **HTTP 402 Payment Required** state to the frontend/off-chain DM.
4. Player pays the fee (XLM/tokens) to purchase the `purchase_premium_action`.
5. Contract verifies payment and grants a high-tier reward or easier encounter path.

## Development

### Prerequisites

- [Soroban CLI](https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup)
- Rust toolchain with `wasm32-unknown-unknown` target.

### Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

### Test

```bash
cargo test
```

### On-Chain Deployment

1. Deploy the contract: `soroban contract deploy ...`
2. Initialize VK: `soroban contract invoke --id <ID> --fn set_vk --vk <VK_JSON>`
3. Start run: `soroban contract invoke --id <ID> --fn init_run --player <ADDR>`

## Acceptance Criteria

- [x] Includes x402 representation in `purchase_premium_action`.
- [x] Real `stellar-zk` verification path in `verify_run_proof`.
- [x] Deterministic run loop and full test coverage.
- [x] Soroban-compatible build (wasm32v1-none).
