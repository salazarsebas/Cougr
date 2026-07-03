# Reversi

> **Transitional example**: This example uses an older Cougr pattern and is preserved
> for compatibility reference. For the current recommended approach, see `snake`.

An on-chain Reversi (Othello) game built with the [Cougr](../../README.md) ECS framework on Stellar Soroban.

## Purpose and pattern

This example demonstrates a turn-based, perfect-information board game where every move
mutates shared board state and can trigger automatic turn-skipping. It showcases Cougr's
`ComponentTrait` pattern for typed, byte-serializable game state — components are plain
Rust structs annotated with `#[contracttype]` that implement `ComponentTrait` directly,
rather than being scanned through `SimpleWorld`/`SimpleQueryBuilder`. This is a lighter-weight
integration suited to a single fixed-shape game state (one board, two players) with no
dynamic entity population.

## Public contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `init_game` | `player_one: Address`, `player_two: Address` | — | Initializes an 8×8 board with the standard opening position; `player_one` plays Black and moves first. Panics if already initialized. |
| `submit_move` | `player: Address`, `row: u32`, `col: u32` | — | Validates and applies a move (requires `player.require_auth()`), flips bracketed opponent pieces, recomputes score, and advances turn/pass state. Panics on illegal move, wrong turn, or finished game. |
| `get_state` | — | `GameState` | Current player, pass-count state, and active/finished status. |
| `get_board` | — | `BoardState` | Raw 64-cell board array (row-major, 0=empty/1=black/2=white), plus width/height. |
| `get_score` | — | `ScoreState` | Black/white piece counts and winner (0=ongoing, 1=black, 2=white, 3=draw). |

## Architecture overview

There is no `GameApp` tick loop — `submit_move` runs all systems synchronously in a fixed
pipeline on each invocation:

```
submit_move
  └─ MoveValidationSystem   → rejects occupied cells or moves that flip nothing
  └─ FlipResolutionSystem   → places the piece, flips bracketed pieces in all 8 directions
  └─ ScoringSystem          → recomputes black/white piece counts from the board
  └─ TurnSystem/PassSystem  → advances to the opponent, or auto-passes if they have no legal move
  └─ EndConditionSystem     → marks the game finished when both players pass or the board is full
```

Each system is a pure function over `components.rs` types (`BoardComponent`, `TurnComponent`,
`GameStatusComponent`, `ScoreComponent`); `lib.rs` owns the single `ECSWorldState` aggregate
and is the only place that touches contract storage.

## Storage model

The entire game lives under one instance-storage key (`WORLD_KEY`, `symbol_short!("WORLD")`)
as a single `ECSWorldState` struct bundling the board, turn, status, score, and both player
addresses. Instance storage is used — not persistent or temporary — because there is exactly
one game per contract instance and the state must survive for the lifetime of that instance
with no per-entry TTL management.

## Main gameplay flow

1. Deployer calls `init_game(player_one, player_two)`; board is seeded with the four-piece
   opening position, Black (`player_one`) to move, status active.
2. Black calls `submit_move(player, row, col)`. The contract checks turn order and move
   legality (the move must bracket at least one opponent piece in a straight line).
3. On a legal move: the piece is placed, bracketed opponent pieces flip, score is
   recomputed, and turn passes to White — unless White has no legal move, in which case
   Black continues and `pass_count` is set to 1.
4. Players alternate `submit_move` calls. If neither player has a legal move
   (`pass_count` reaches 2) or the board fills, `EndConditionSystem` sets status to finished.
5. Either player calls `get_score` to read the final piece counts and winner.

## Cougr APIs used

- `cougr_core::component::ComponentTrait` — gives each component (`BoardComponent`,
  `TurnComponent`, `GameStatusComponent`, `ScoreComponent`) a `component_type()` symbol and
  byte-level `serialize`/`deserialize`, chosen here because the game has a single fixed set
  of components per instance rather than a dynamic entity population that would benefit from
  `SimpleWorld`/`SimpleQueryBuilder` scanning.

## Build and test commands

```bash
cargo test
stellar contract build
```

## Known limitations

- Does not use `GameApp`, `ScheduleStage`, or `SimpleWorld` — game logic is invoked directly
  from contract entrypoints rather than through a tick-based scheduler, since a turn-based
  game with one decision point per call does not need staged scheduling.
- No timeout/forfeit mechanism for an unresponsive player.
- No spectator or replay API beyond the three read-only getters.
