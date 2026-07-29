# {{crate_name}}

{{description}}

Generated with `cougr new --template {{template_id}}`, based on the canonical
[`{{source_example}}`](https://github.com/salazarsebas/Cougr/tree/main/examples/{{source_example}})
example.

> **Experimental**: `cougr_core::circuits` is an experimental API. The circuit
> spec ships with the Cougr development verification key — replace it with your
> own trusted-setup key via `GameCircuitSpec::with_verification_key` before any
> real deployment.

## Purpose and pattern

Players take a seat at a table, then prove — without revealing a card — that the
hand they hold was really dealt from the committed deck. The contract never
learns the hand: it checks a Groth16 proof against public commitments and counts
the deals that verified. This is the reference shape for poker, deck-builders,
and anything else where players hold private state the chain must still trust.

## Public contract API

| Function | Parameters | Returns | Description |
| --- | --- | --- | --- |
| `init_table` | `deck_size: u32`, `hand_size: u32` | `TableConfig` | Open a table and freeze the circuit layout |
| `join_table` | `player: Address` | `u32` | Take a seat; returns the zero-based seat number |
| `seat_of` | `player: Address` | `Option<u32>` | The seat a player joined with |
| `table` | — | `Option<TableConfig>` | The frozen table configuration |
| `verify_deal` | `player: Address`, `deck_root: BytesN<32>`, `hand_commitment: BytesN<32>`, `proof: Groth16Proof` | `bool` | Verify a deal proof for that player's seat |
| `deals_verified` | `player: Address` | `u32` | Deals that player has proven so far |

## Architecture overview

```
lib.rs         contract entrypoints — seat lookup, proof verification, counters
  ├─ components.rs   TableConfig, DealsVerified, storage keys, seat → entity map
  └─ systems.rs      validate_config(), circuit_for(), verify_deal() — pure rules
```

The seat number is the hinge of the whole design. It is a public input to the
circuit, so a proof is bound to one seat: replaying another player's proof under
your own address fails verification rather than being caught by an extra check.

## Storage model

| Storage | Contents | Why |
| --- | --- | --- |
| Instance — `table` | `TableConfig` | One tiny value read by every call |
| Instance — `(seat, Address)` | Seat number | Direct address → seat lookup, no scan |
| Instance — `world` | `SimpleWorld` with a `DealsVerified` per seat | Per-seat gameplay counters live in the ECS |

Deck order and hands are deliberately **not** stored. Only the deck root and
hand commitment appear on chain, and only as arguments to a verification call.

## Main gameplay flow

1. A host calls `init_table(52, 5)`, freezing the circuit layout.
2. Each player calls `join_table` and receives seat `0`, `1`, `2`, …
3. Off chain, the dealer commits to a shuffled deck and produces a Merkle root
   plus, per player, a hand commitment and a Groth16 proof.
4. A player submits `verify_deal` with the deck root, their hand commitment, and
   the proof. The contract rebuilds the circuit spec and verifies.
5. On success the seat's `DealsVerified` counter increases; on failure the call
   returns `false` and nothing is written.

## Cougr APIs used

| API | Why |
| --- | --- |
| `circuits::hidden_cards` | Pre-built public-input layout for a hidden-card deal — no hand-rolled circuit wiring |
| `zk::Groth16Proof` | Proof accepted directly as a contract argument |
| `impl_component!` | `DealsVerified` is a single scalar per seat |
| `SorobanGame` / `impl_soroban_game!` | Removes hand-written world load/save boilerplate |
| `test::GameHarness`, `circuits::test_fixtures` | Real pipeline proofs in an integration test |

## Build and test

```bash
cargo test
stellar contract build
```

`stellar contract build` needs the WASM target and the Stellar CLI:

```bash
rustup target add wasm32v1-none
cargo install --locked stellar-cli
```

The build writes `target/wasm32v1-none/release/{{module_name}}.wasm`, which is
what you deploy with `stellar contract deploy`.

## Known limitations

* Ships the Cougr development verification key. Anyone who holds that proving
  key can forge a proof — swap in your own before deploying.
* Nobody is bound to committing the deck: `verify_deal` trusts whatever
  `deck_root` the caller passes. A real game stores the root at deal time.
* Seats are never released, and there is no cap on how many players can join.
* Groth16 verification is expensive. Budget for it before adding per-turn proofs.
