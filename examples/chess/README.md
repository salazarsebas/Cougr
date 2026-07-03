# Verifiable Chess with ZK Move Validation

> **Transitional example**: This example uses an older Cougr pattern and is preserved
> for compatibility reference. For the current recommended approach to ZK-backed game
> logic specifically, see `hidden_hand`, which uses `cougr_core::circuits`' pre-built
> circuit builders instead of constructing a `CustomCircuit` by hand.

A simplified chess implementation on Stellar Soroban that demonstrates **zero-knowledge proof
verification** for move legality using the [Cougr](../../README.md) ZK framework. Move
legality is checked off-chain by a circuit; the contract only verifies a compact Groth16
proof and applies the move.

## Purpose and pattern

This example demonstrates how to keep move-validation logic entirely off-chain while still
enforcing it trustlessly: instead of the contract checking piece-movement rules, path
obstruction, and check/checkmate conditions on every move, the player generates a Groth16
proof off-chain that the move is legal given the current board state, and the contract
verifies that proof in constant time. It showcases `cougr_core::privacy::experimental`'s
generic `CustomCircuit` builder for binding a proof's public inputs (`state_hash`, `from`,
`to`) to a verification key, alongside the same `ComponentTrait` pattern used by the other
ECS-lite examples for typed, byte-serializable game state.

## Public contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `new_game` | `white: Address`, `black: Address` | — | Initializes the standard 8x8 opening position, computes the initial `state_hash`, and sets White to move first. |
| `submit_move` | `player: Address`, `from: u32`, `to: u32`, `proof: Bytes` | `MoveResult` | Requires `player.require_auth()`. Checks turn order and game status, verifies the supplied proof against the stored verification key and current `state_hash`, then applies the move, recomputes the hash, and switches turns. |
| `resign` | `player: Address` | — | Marks the game `Resigned` if called by either registered player. |
| `get_board` | — | `BoardState` | Current piece map and `state_hash`. |
| `get_state` | — | `GameState` | Full game state: both players, board, turn state, and last proof record. |
| `set_vk` | `vk: VerificationKey` | — | Admin function to set/replace the Groth16 verification key used for move proofs. |

## Architecture overview

There is no `GameApp` tick loop. `submit_move` runs a fixed pipeline synchronously on each
invocation:

```
submit_move
  └─ TurnSystem              → rejects calls from the wrong player or a finished game
  └─ ProofVerificationSystem → verifies the Groth16 proof against state_hash/from/to via CustomCircuit
  └─ BoardUpdateSystem       → applies the move to the piece map, recomputes state_hash
  └─ TurnSystem              → increments move_count, switches current player
  └─ EndGameSystem           → marks Checkmate if either king is no longer on the board
```

`components.rs` defines the data (`Piece`, `BoardState`, `TurnState`, `ProofRecord`,
`GameState`, `MoveResult`) and the `ComponentTrait` implementations for `BoardState` and
`TurnState`. `systems.rs` holds the pure, storage-free game logic (`init_board`,
`compute_state_hash`, `apply_move`, `check_endgame`). `zk.rs` holds the proof-verification
wrapper (`build_move_circuit`, `decode_proof`) built on `cougr_core::privacy::experimental`.
`lib.rs` owns the single `GameState` aggregate, is the only module that touches contract
storage, and wires the systems and the ZK verification step together inside `submit_move`.

## Storage model

The entire game lives under one instance-storage key (`GAME_KEY`,
`symbol_short!("GAME")`) as a single `GameState` struct bundling both player addresses, the
board, turn state, and the last proof record. The verification key is stored separately
under its own instance-storage key (`VK_KEY`, `symbol_short!("VK")`) so it can be rotated
with `set_vk` independently of game state. Instance storage is used for both — not persistent
or temporary — because there is exactly one game and one verification key per contract
instance, with no per-entry TTL management needed.

## Main gameplay flow

1. Deployer calls `set_vk(vk)` once to register the Groth16 verification key matching the
   off-chain move-validation circuit.
2. Deployer (or either player) calls `new_game(white, black)`; the board is seeded with the
   standard opening position, White to move, and the initial `state_hash` is computed.
3. Off-chain, White picks a move (`from`, `to`), runs the move-validation circuit against the
   current `state_hash`, and produces a Groth16 proof that the move is legal.
4. White calls `submit_move(white, from, to, proof)`. The contract checks turn order, verifies
   the proof via `CustomCircuit`, and on success applies the move, recomputes `state_hash`,
   records the proof, and switches the turn to Black.
5. Players alternate steps 3-4. If a king is ever captured, `EndGameSystem` sets the status to
   `Checkmate`; either player may also call `resign` at any time to end the game early.
6. Any caller reads `get_board` or `get_state` to render the current position or check status.

## Cougr APIs used

- `cougr_core::component::ComponentTrait` — gives `BoardState` and `TurnState` a
  `component_type()` symbol and byte-level `serialize`/`deserialize`, chosen because chess has
  a single fixed-shape game state per contract instance rather than a dynamic entity
  population that would benefit from `SimpleWorld`/`SimpleQueryBuilder` scanning.
- `cougr_core::privacy::experimental` (`CustomCircuit`) — chosen specifically because this
  example demonstrates Groth16 proof submission: the contract never evaluates chess rules
  itself, it only binds a proof's public inputs (`state_hash`, `from`, `to`) to a
  verification key and checks the proof verifies. This is the canonical use case the
  `privacy::experimental` guidance in the Cougr API table calls out (Groth16 proof submission
  / pairing-based verification), as opposed to `privacy::stable`, which is for commit-reveal
  and Merkle-proof patterns this example does not use. Newer examples (`hidden_hand`,
  `fog_explorer`, `dice_duel`, `blind_auction`) use the higher-level `cougr_core::circuits`
  pre-built builders instead of hand-rolling a `CustomCircuit`; this example predates that
  module and is kept as a reference for the lower-level builder API.
- `cougr_core::privacy::{Groth16Proof, VerificationKey}` — the concrete proof and key types
  the `CustomCircuit` builder and verifier operate on; stored on-chain (`VerificationKey`) or
  passed per-call (`Groth16Proof`, decoded from the `proof: Bytes` argument) so the contract
  has everything it needs to verify a move without re-deriving any chess logic.

## Build and test commands

```bash
cargo test
stellar contract build
```

## Known limitations

- Proof decoding (`zk::decode_proof`) is not implemented — it intentionally panics, since
  wiring up a real off-chain Groth16 prover/serializer is out of scope for this example. Tests
  exercise the turn/resign/game-over paths that return before proof verification is reached.
- The move-validation circuit itself (piece movement rules, path obstruction, check/checkmate
  detection) lives off-chain and is not part of this repository; only the on-chain
  verification wrapper is shown.
- Checkmate detection is simplified to "a king is missing from the board" rather than full
  check/checkmate/stalemate rules. Castling, en passant, and pawn promotion are not modeled.
- Does not use `GameApp`, `ScheduleStage`, or `SimpleWorld` — `submit_move` runs its pipeline
  directly from the contract entrypoint since each call has exactly one decision point and
  does not need staged scheduling.
