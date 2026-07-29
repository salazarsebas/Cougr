# ADR 0006: Game Circuit Suite (`cougr_core::circuits`)

## Status

Accepted

## Context

Game developers need fog-of-war, hidden cards, fair dice, and sealed-bid mechanics
without authoring Circom circuits and wiring Groth16 verification by hand. Cougr
already exposes low-level verifiers under `zk::experimental`, but the onboarding
path requires weeks of ZK specialization.

## Decision

1. Ship four pre-built circuit builders under `cougr_core::circuits` (always
   available, Experimental maturity):

   | Builder | Public inputs |
   |---------|---------------|
   | `hidden_cards(deck_size, hand_size)` | deck_root, hand_commitment, player_id, deck_size, hand_size |
   | `fog_of_war(w, h, radius)` | map_root, prior/next explored roots, origin, tile, radius |
   | `fair_dice(sides, seed_commitment)` | seed_commitment, roll_result, sides, nonce |
   | `sealed_bid(max_bid)` | auction_id, bid_commitment, revealed_bid, max_bid |

2. Each builder returns `GameCircuitSpec` with a frozen `PublicInputLayout`,
   placeholder VK (correct IC length), and typed verify methods that delegate to
   `zk::experimental::verify_groth16`. Production deploys replace the VK via
   `with_verification_key`.

3. `fog_of_war` reuses `FogOfWarCircuit` in `zk::advanced` — no duplicate
   verification logic.

4. Circom scaffolds and off-chain scripts live in
   `internal/cougr-core-circuits/` (`publish = false`). Rust implementation lives
   in `src/circuits/`.

5. Canonical examples demonstrate each builder:

   - `examples/hidden_hand/`
   - `examples/fog_explorer/`
   - `examples/dice_duel/`
   - `examples/blind_auction/`

## Consequences

- Developers integrate common game privacy patterns in hours, not weeks.
- Public-input layouts are versioned by `CircuitId` and must not change without a
  new ADR.
- On-chain builders ship an unbound VK; load test/production keys from
  `internal/cougr-core-circuits` (`bun run pipeline` → `exported/*_vk.json`).
- Circuits use Poseidon + game constraints (~325–13.6k R1CS); pot14 trusted setup.
- `zk::stable` remains unchanged; all new surface stays Experimental until external
  audit.