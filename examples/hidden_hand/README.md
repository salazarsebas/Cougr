# hidden_hand

**Canonical** ZK circuit example demonstrating `circuits::hidden_cards` for private card deals.

## Purpose and pattern

This example demonstrates how to implement private card dealing (e.g., poker hand setups) on-chain. It uses ZK proofs to verify that a deck was shuffled and a hand of cards was dealt honestly, without revealing the cards to other players or the public on-chain ledger.

## Public contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `init_table` | `deck_size: u32`, `hand_size: u32` | `TableConfig` | Configures the game table with the specified deck and hand size, initializing the ZK verifier. |
| `verify_deal` | `player: Address`, `deck_root: BytesN<32>`, `hand_commitment: BytesN<32>`, `proof: Groth16Proof` | `bool` | Verifies a Groth16 deal proof against the configured hidden-cards layout. |

## Architecture overview

```
                         ┌──────────────┐
                         │ Card Dealer  │
                         └──────┬───────┘
                                │ Generates ZK Proof
                     ┌──────────▼──────────┐
                     │     HiddenHand      │
                     │ (Soroban Contract)  │
                     └──────────┬──────────┘
                                │ Loads Spec
                     ┌──────────▼──────────┐
                     │ circuits::          │
                     │  hidden_cards       │
                     └─────────────────────┘
```

The dealer generates a Groth16 proof off-chain, verifying that the hand commitment was indeed created from a valid slice of the shuffled deck. The contract verifies the proof using the `hidden_cards` circuit spec.

## Storage model

The `TableConfig` containing the card configuration is stored in **Instance Storage**. The ZK verification keys are built dynamically or stored securely within the contract instance.

## Main gameplay flow

1. **Setup**: The contract calls `init_table(deck_size, hand_size)` to prepare the ZK circuit parameters.
2. **Dealing**: The dealer generates a hand commitment and a proof off-chain.
3. **Verification**: The contract calls `verify_deal(player, deck_root, hand_commitment, proof)` to validate that the deal is correct before proceeding with gameplay.

## Cougr APIs used

- `circuits::hidden_cards`: Fetches the circuit layout, verification key, and verify wrapper for Groth16 hidden cards.
- `zk::Groth16Proof`: Data structure representing the cryptographic proof.

## Recommended testing approach

Utilize `GameHarness` and `test_fixtures` to mock ZK inputs and pipeline proof bytes. This allows testing successful proof verification paths without running full proving ceremonies in unit tests.

## Build and test commands

```bash
cargo test
stellar contract build
```

## Known limitations

- Simple single-deck configuration.
- Proving is done entirely off-chain.
