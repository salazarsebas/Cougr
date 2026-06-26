# Memory Match

> **Transitional example**: This example uses an older Cougr pattern and is preserved
> for compatibility reference. For the current recommended approach, see `snake`.

An on-chain Memory Match card game built with the [Cougr](../../README.md) ECS framework on Stellar Soroban.

## Purpose and pattern

This example demonstrates a single-player, turn-based reveal-and-match game where pairs of
cards are flipped and compared against each other rather than against fixed rules. It
showcases Cougr's `ComponentTrait` pattern for typed, byte-serializable game state: cards,
the board, and game progress are each modeled as a `#[contracttype]` struct with manual
`serialize`/`deserialize`, rather than being scanned through `SimpleWorld`/`SimpleQueryBuilder`.
This is a lighter-weight integration suited to a single fixed-shape game state (16 cards, one
board, one player) with no dynamic entity population.

## Public contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `init_game` | `player: Address` | `GameState` | Seeds a fresh 16-card board (8 deterministic pairs) for `player` and returns the initial state. |
| `reveal_card` | `player: Address`, `position: u32` | `RevealInfo` | Reveals the card at `position`. Resolves a pair once two cards are revealed (match, no-match, or game-over). Panics if not initialized, the caller isn't the registered player, the game is over, two cards are already revealed, the position is out of range, or the card is already revealed/matched. |
| `get_game_state` | — | `GameState` | Current board view, revealed/matched counts, move count, and game-over flag. |
| `reset_game` | `player: Address` | `GameState` | Hides all cards and clears progress, keeping the same board layout. Panics if not initialized or the caller isn't the registered player. |

## Architecture overview

There is no `GameApp` tick loop — `reveal_card` runs a fixed pipeline synchronously on each
invocation:

```
reveal_card
  └─ ValidationSystem     → checks caller, game-over state, two-card limit, position bounds
  └─ RevealSystem          → flips the card to Revealed, increments the move counter
  └─ (when 2 cards are revealed)
       └─ PairResolutionSystem → compares values; marks Matched + checks for game-over,
                                  or hides both cards again on a mismatch
```

`reset_game` runs a separate `ResetSystem` that rehides every card and zeroes progress
counters without regenerating the board layout. `components.rs` defines the data
(`CardComponent`, `BoardComponent`, `GameStateComponent`, the `ECSWorldState` aggregate, and
the public `GameState`/`RevealInfo`/`RevealResult` types); `systems.rs` holds the pure
validation, reveal, resolution, reset, and projection functions; `lib.rs` owns the single
`ECSWorldState` and is the only module that touches contract storage.

## Storage model

The entire game lives under one instance-storage key (`WORLD_KEY`, `symbol_short!("WORLD")`)
as a single `ECSWorldState` struct bundling all 16 card components, the board component, and
the game-state component. Instance storage is used — not persistent or temporary — because
there is exactly one game per contract instance and the state must survive for the lifetime
of that instance with no per-entry TTL management.

## Main gameplay flow

1. Deployer calls `init_game(player)`; 16 cards are created with a deterministic layout
   (positions 0–7 and 8–15 hold matching values 0–7), all starting `Hidden`.
2. Player calls `reveal_card(player, position)` for a hidden card; it flips to `Revealed`
   and the move counter increments.
3. Once two cards are revealed, the contract compares their values:
   - **Match**: both cards become `Matched`, `matched_pairs` increments, and reveals are
     re-enabled. If all 8 pairs are now matched, the game is marked over.
   - **No match**: both cards flip back to `Hidden` and reveals are re-enabled.
4. Player repeats step 2–3 until `matched_pairs` reaches 8 and `game_over` is `true`.
5. Player (or anyone reading state) calls `get_game_state` at any point to read the board,
   or calls `reset_game` to rehide all cards and start over on the same layout.

## Cougr APIs used

- `cougr_core::component::ComponentTrait` — gives each component (`CardComponent`,
  `BoardComponent`, `GameStateComponent`) a `component_type()` symbol and explicit byte-level
  `serialize`/`deserialize`. This is the only Cougr API the example uses: the game has one
  fixed-shape `ECSWorldState` per contract instance (16 cards, one board, one player) rather
  than a dynamic entity population, so `SimpleWorld`/`SimpleQueryBuilder` scanning would add
  overhead with no benefit. There is a single decision point per call (reveal or reset), so
  `GameApp`/`ScheduleStage` staged ticking is not needed either, and the game has no hidden
  information, multi-device auth, or proof-verification requirements that would call for
  `auth`, `privacy::stable`, `privacy::experimental`, or `ops` standards.

## Build and test commands

```bash
cargo test
stellar contract build
```

## Known limitations

- Does not use `GameApp`, `ScheduleStage`, or `SimpleWorld` — game logic is invoked directly
  from contract entrypoints rather than through a tick-based scheduler, since a single-player
  game with one decision point per call does not need staged scheduling.
- The card layout is fixed and deterministic (not shuffled), so the pairing is the same for
  every game; randomized layouts are out of scope for this example.
- `reset_game` reuses the same fixed layout rather than generating a new one.
- No timeout/forfeit mechanism, scoring across multiple games, or multiplayer support.
