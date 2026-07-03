# dice_duel

**Canonical** ZK circuit example demonstrating `circuits::fair_dice` for verifiable dice rolls.

## Purpose and pattern

This example showcases a verifiable on-chain dice rolling game. Players generate random numbers off-chain and submit ZK proofs to prove that their roll result is deterministic, within bounds, and bound to the initial committed seed, preventing manipulation of on-chain randomness.

## Public contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `init_duel` | `sides: u32`, `seed_commitment: BytesN<32>` | `DuelConfig` | Registers the dice parameters and the starting cryptographic seed commitment. |
| `submit_roll` | `player: Address`, `roll_result: u32`, `nonce: u32`, `proof: Groth16Proof` | `bool` | Verifies a dice roll ZK proof and updates the roll record if valid. |
| `roll_record` | `player: Address` | `RollRecord` | Retrieves the stored roll result for a player. |

## Architecture overview

```
                         ┌───────────────┐
                         │    Player     │
                         └──────┬────────┘
                                │ Rolls & Generates ZK Proof
                     ┌──────────▼──────────┐
                     │      DiceDuel       │
                     │ (Soroban Contract)  │
                     └──────────┬──────────┘
                                │ Loads Spec
                     ┌──────────▼──────────┐
                     │ circuits::          │
                     │  fair_dice          │
                     └─────────────────────┘
```

The player commits a random seed and runs a deterministic calculation. They submit a Groth16 proof showing the calculation output matches the result of their roll.

## Storage model

Dice duel config and player roll history are stored in **Instance Storage** on-chain via Soroban instance key-value associations.

## Main gameplay flow

1. **Setup**: Call `init_duel` to bind the dice properties and seed commitment.
2. **Roll**: Players roll dice off-chain and calculate proof.
3. **Submit**: Players call `submit_roll` to verify and record the roll on-chain.

## Cougr APIs used

- `circuits::fair_dice`: Implements the dice roll verification boundary specifications.
- `zk::Groth16Proof`: Holds proof payload.

## Recommended testing approach

Integrate `GameHarness` and `Scenario` runner combined with `test_fixtures::pipeline_proof` to simulate individual rolls and multi-player roll scenarios.

## Build and test commands

```bash
cargo test
stellar contract build
```

## Known limitations

- Simple single-roll mechanics.
- The seed is assumed to be cryptographically secure and generated off-chain.
