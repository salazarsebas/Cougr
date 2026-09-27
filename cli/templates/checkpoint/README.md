# {{crate_name}}

{{description}}

Generated with `cougr new --template {{template_id}}`, demonstrating the
honest pattern for approximating real-time games on Soroban.

## The on-chain / off-chain boundary

Real-time games (Snake, Pong, Asteroids) cannot put every frame on chain.
Stellar's 3–5 second block time is orders of magnitude slower than a 60 fps
game loop, and even at fractions of a cent per transaction the volume would be
prohibitive.

The **checkpoint pattern** moves ticks off chain and uses the contract only
for what it is good at: ordering, timestamping, and adjudication.

| Off chain | On chain |
| --- | --- |
| Physics simulation, collision detection | `commit_checkpoint` — tick index, state hash, score |
| Frame-by-frame game state | Dispute window and outcome |
| Sound, animation, rendering | Match start / end authority |

## Public contract API

| Function | Parameters | Returns | Description |
| --- | --- | --- | --- |
| `start_match` | `player: Address` | `MatchState` | Open a new match; the caller becomes the sole player |
| `commit_checkpoint` | `player: Address`, `tick: u32`, `state_hash: u64`, `score: u32` | `MatchState` | Commit a hash of the simulated state at the given tick |
| `dispute_checkpoint` | `challenger: Address`, `tick: u32`, `claimed_hash: u64` | `MatchState` | Challenge a committed checkpoint within the dispute window |
| `finalize_match` | `player: Address` | `MatchState` | Close a match whose dispute window has passed |
| `get_state` | — | `MatchState` | Current match state |

`MatchState.status` constants (in `components.rs`): `0` running, `1` finalised,
`2` disputed.

## Architecture overview

```
lib.rs         contract entrypoints — load world, validate, write, save world
  ├─ components.rs   MatchConfig + MatchRecord (rich), CheckpointState (plain)
  └─ systems.rs      validate_commit(), validate_dispute(), window_open() — pure rules
```

Rules live in `systems.rs` as pure functions over component values.
`lib.rs` owns storage; the rules never touch it.

## Storage model

The `SimpleWorld` lives in **instance storage** under the `"world"` key, wired
by `impl_soroban_game!({{ContractName}}, "world")`. One match per contract
instance; all components hang off a fixed entity (`MATCH_ENTITY = 1`).

`MatchConfig` and `MatchRecord` use `impl_rich_component!` because they carry
`Address` fields. `CheckpointState` holds only scalars and uses the cheaper
`impl_component!`.

## Typical flow

1. Call `start_match(player)` — opens the match and records the ledger
   timestamp.
2. The client simulates the game locally. Every N ticks it calls
   `commit_checkpoint(player, tick, hash, score)` with a hash of its local
   state vector.
3. If a challenger disagrees with a committed hash, they call
   `dispute_checkpoint(challenger, tick, claimed_hash)` within the dispute
   window.  The contract marks the match disputed and stops accepting further
   checkpoints.
4. Once the window has elapsed, any address can call `finalize_match` to close
   the match and lock the last committed score.

## Dispute window

The window is `DISPUTE_WINDOW_LEDGERS` ledgers from the time the checkpoint was
committed. The default is 1000 ledgers (~83 minutes at ~5 s/ledger). A dispute
after the window is rejected with `toolate`.

## Cougr APIs used

| API | Why |
| --- | --- |
| `impl_rich_component!` | `MatchConfig` / `MatchRecord` carry `Address` — needs XDR storage |
| `impl_component!` | `CheckpointState` is all scalars — no codec needed |
| `SorobanGame` / `impl_soroban_game!` | Removes hand-written world load/save boilerplate |
| `SimpleWorld` | Holds all components under one entity |
| `test::GameHarness` | Registers the contract in a fresh `Env` |

## Build and test

```bash
cougr check
cargo test
stellar contract build
```

`stellar contract build` needs the WASM target and the Stellar CLI:

```bash
rustup target add wasm32v1-none
cargo install --locked stellar-cli
```

## Known limitations

* `commit_checkpoint` trusts the caller. Proofs of computation (ZK or
  interactive bisection) are out of scope for this template — see
  `cougr-core`'s ZK primitives if you need verifiable off-chain execution.
* One match per contract instance. A leaderboard of concurrent matches needs
  per-match entities instead of the fixed `MATCH_ENTITY`.
* No stake or slash logic. Dispute outcomes are recorded but do not move
  tokens.
