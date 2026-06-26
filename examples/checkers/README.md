# Checkers

> **Transitional example**: This example uses an older Cougr pattern and is preserved
> for compatibility reference. For the current recommended approach, see `snake`.

A fully on-chain two-player Checkers game implemented as a Soroban smart contract.

## Purpose and pattern

This example demonstrates a turn-based board game with non-trivial rule enforcement —
diagonal grid movement, mandatory captures, multi-hop chain captures, king promotion,
and win detection — implemented as plain Rust functions over `#[contracttype]` structs.
It showcases the simplest possible Cougr-adjacent pattern: component-shaped data types
with hand-written contract storage access, predating the `ComponentTrait`/`GameApp`
conventions used by newer examples. It is a useful reference for understanding what a
Soroban board-game contract looks like with no ECS framework involvement at all.

## Public contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `init_game` | `player_one: Address`, `player_two: Address` | `Result<(), CheckersError>` | Seeds the standard 8×8 opening position; Player One moves first. Returns `AlreadyInitialised` if called more than once. |
| `submit_move` | `player: Address`, `from_row: u32`, `from_col: u32`, `to_row: u32`, `to_col: u32` | `Result<(), CheckersError>` | Validates and applies a step or capture, resolves chain captures, promotes kings, and advances or holds the turn. Returns one of the `CheckersError` variants on any rule violation. |
| `get_state` | — | `Result<GameState, CheckersError>` | Full snapshot: board, turn, status, and both player addresses. |
| `get_board` | — | `Result<BoardState, CheckersError>` | Raw 64-cell board array, row-major order. |
| `get_current_player` | — | `Result<Address, CheckersError>` | The `Address` of the player whose turn it currently is. |

### Error codes

| Error | Code | Meaning |
|---|---|---|
| `AlreadyInitialised` | 1 | `init_game` called more than once |
| `NotInitialised` | 2 | Any call before `init_game` |
| `NotAPlayer` | 3 | Caller is not `player_one` or `player_two` |
| `WrongTurn` | 4 | Caller is a registered player but it is not their turn |
| `NotYourPiece` | 5 | Source square is empty or owned by the opponent |
| `DestinationOccupied` | 6 | Target square is already occupied |
| `IllegalMove` | 7 | Move is not a legal diagonal step or jump |
| `MustCapture` | 8 | A capture is available but a non-capture move was attempted |
| `GameOver` | 9 | The game has already ended |
| `OutOfBounds` | 10 | Row or column index ≥ 8 |
| `ChainCapturePieceMismatch` | 11 | During a chain capture the origin square was not the chain square |
| `NotDarkSquare` | 12 | Destination square is a light square (row + col is even) |

## Architecture overview

There is no `GameApp` tick loop. `submit_move` runs a fixed validation/update pipeline
synchronously on each call:

```
submit_move
  └─ identify caller, check turn order, bounds, and dark-square rule
  └─ MoveValidationSystem  (systems.rs: legal_steps / legal_captures)
       → confirms the requested move is a legal step or capture for this piece
  └─ forced-capture check   (systems.rs: any_capture_available)
       → rejects a non-capture move if any capture exists for the player
  └─ apply move + remove captured piece
  └─ PromotionSystem        (systems.rs: maybe_promote)
       → crowns a man reaching the opponent's back rank
  └─ chain-capture detection (systems.rs: legal_captures on the landing square)
       → holds the turn and records the active square if another capture is available
  └─ EndConditionSystem     (systems.rs: check_winner)
       → declares a winner on piece-count exhaustion or no legal moves
  └─ TurnSystem
       → advances current_player, or holds it during a chain capture
```

`components.rs` holds the data shapes (`BoardComponent`, `TurnComponent`,
`GameStatusComponent`, `GameStatus`, the public `GameState`/`BoardState` view types, the
internal `ChainCapture` tracker, and the small `SmallVec4` fixed-capacity collection).
`systems.rs` holds the pure, storage-free game-logic functions (move/capture legality,
promotion, win detection, board initialisation). `lib.rs` is the only module that touches
`env.storage()`; it loads components, calls into `systems.rs`, and writes the results
back.

## Storage model

All state lives in **persistent storage**, under five top-level `Symbol` keys:

| Key | Contents |
|---|---|
| `BOARD` | `BoardComponent` — 64-cell flat grid |
| `TURN` | `TurnComponent` — current player and move number |
| `STATUS` | `GameStatusComponent` — active/finished and winner |
| `P1` | `Address` of Player One |
| `P2` | `Address` of Player Two |
| `CHAIN` | `ChainCapture` — present only while a multi-hop capture sequence is in progress |

Persistent storage (rather than instance storage) is used because each key is read and
rewritten independently inside `submit_move` — the board, turn, and status are not bundled
into one aggregate struct in this example, so per-key persistent entries map directly onto
that access pattern. The `CHAIN` key is removed as soon as a turn fully ends; its absence
is what tells `submit_move` that no multi-hop sequence is active, so it doubles as both
data and a lightweight state flag.

## Main gameplay flow

1. Deployer calls `init_game(player_one, player_two)`. The board is seeded with the
   standard 12-piece-per-side opening position; Player One moves first.
2. Player One calls `submit_move` with a `(from_row, from_col)` → `(to_row, to_col)` pair.
   The contract checks turn order, bounds, the dark-square rule, piece ownership, and
   move geometry (diagonal step or jump).
3. If any capture is available for the mover anywhere on the board, a non-capture move is
   rejected with `MustCapture` — captures are mandatory, not optional.
4. On a legal capture, the jumped piece is removed. If the landing square has a further
   capture available, the turn is **held**: the same player must continue jumping with the
   same piece (tracked via the `CHAIN` key) until no further capture exists.
5. A man reaching the opponent's back rank is promoted to a king immediately after the
   move that lands it there, before chain-capture continuation is checked.
6. After each move, `EndConditionSystem` checks whether either side has no pieces left or
   no legal move available; if so, `STATUS` flips to `Finished` with a recorded winner.
7. Players alternate `submit_move` calls until a winner is recorded; `get_state`,
   `get_board`, and `get_current_player` provide read-only views at any point.

## Cougr APIs used

None. `Cargo.toml` declares `cougr-core` as a dependency, but the contract source does
not import or call into it — no `ComponentTrait`, `impl_component!`, `GameApp`,
`SimpleWorld`/`SimpleQueryBuilder`, `auth`, or `privacy` APIs are used anywhere in
`components.rs`, `systems.rs`, or `lib.rs`. Per the API usage guidance in
`EXAMPLE_STANDARD.md` §8, none of those APIs apply here: there is exactly one fixed-shape
game state per contract instance (no dynamic entity population to scan with
`SimpleWorld`/`SimpleQueryBuilder`), one synchronous validation pipeline per call (no need
for `GameApp`/`ScheduleStage` ordering across stages), no session-key or multi-device flow
(no need for `auth`), and no hidden information or proof submission (no need for
`privacy::stable` or `privacy::experimental`). This example is kept as a transitional
reference precisely because it predates Cougr's component/ECS conventions — see `snake`
for the current recommended approach to structuring an example around `GameApp` and Cougr
components.

## Build and test commands

```bash
cargo test
stellar contract build
```

## Known limitations

- Does not use `GameApp`, `ScheduleStage`, `SimpleWorld`, or any other `cougr-core` API —
  game logic is invoked directly from contract entrypoints over plain `#[contracttype]`
  structs, since a two-player, one-decision-per-call board game does not require a tick
  scheduler or entity queries.
- No timeout/forfeit mechanism for an unresponsive player.
- No draw detection (e.g., repetition or no-progress rules); a game can only end by piece
  exhaustion or a player having no legal move.
- No spectator or replay API beyond the three read-only getters.
