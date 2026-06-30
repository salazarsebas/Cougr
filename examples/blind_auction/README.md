# blind_auction

**Canonical** ZK circuit example demonstrating `circuits::sealed_bid` for sealed-bid reveals.

## Purpose and pattern

This example demonstrates how to run a blind auction on-chain. Bidders commit their encrypted bids, and during the reveal phase, submit a ZK proof to verify their bid value matches the original commitment and is below the maximum allowed bid limits, without revealing bids early or leaking bid range details.

## Public contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `init_auction` | `max_bid: u32`, `auction_id: BytesN<32>` | `AuctionConfig` | Starts a new auction config with maximum bid limits and an auction identifier. |
| `reveal_bid` | `bidder: Address`, `bid_commitment: BytesN<32>`, `revealed_bid: u32`, `proof: Groth16Proof` | `bool` | Verifies a bidder's reveal proof, storing the bid record if valid. |
| `bid_reveal` | `bidder: Address` | `BidReveal` | Retrieves the revealed bid details for a bidder. |

## Architecture overview

```
                         ┌───────────────┐
                         │    Bidder     │
                         └──────┬────────┘
                                │ Submits Bid & ZK Proof
                     ┌──────────▼──────────┐
                     │    BlindAuction     │
                     │ (Soroban Contract)  │
                     └──────────┬──────────┘
                                │ Loads Spec
                     ┌──────────▼──────────┐
                     │ circuits::          │
                     │  sealed_bid         │
                     └─────────────────────┘
```

The bidder submits their bid value and proof that verifies their bid corresponds to the committed hash. The contract runs the `sealed_bid` verifier logic to register the bid.

## Storage model

Auction configuration records and active bid listings are stored in **Instance Storage** on-chain.

## Main gameplay flow

1. **Setup**: Call `init_auction` to setup maximum bid constraints and the auction ID.
2. **Commit Phase**: Bidders record hash commitments of their bids (handled off-chain or via standard storage).
3. **Reveal Phase**: Bidders call `reveal_bid` with their bids and ZK proofs to unlock and register their bid weights.

## Cougr APIs used

- `circuits::sealed_bid`: Handles verify calculations for sealed on-chain bidding logic.
- `zk::Groth16Proof`: Contains the proof struct representation.

## Recommended testing approach

Use `GameHarness` and standard `test_fixtures` to execute happy-path reveals and invalid proof rejection flows.

## Build and test commands

```bash
cargo test
stellar contract build
```

## Known limitations

- Simple single-item auction model.
- Winner computation logic is not included (focuses on verification of bid validity).
