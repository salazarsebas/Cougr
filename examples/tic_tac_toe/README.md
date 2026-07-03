# Tic Tac Toe

> **Transitional example**: This example uses an older Cougr pattern and is preserved
> for compatibility reference. For the current recommended approach, see `snake`.

An on-chain Tic Tac Toe game built with the [Cougr](../../README.md) ECS framework on
Stellar Soroban. Note: although this example already uses cougr-core's newest macro-based
component pattern (`impl_rich_component!` / `impl_component!` / `impl_soroban_game!`), it is
not on the canonical examples list, so it is marked transitional per the project standard —
see "Cougr APIs used" below for why these specific macros were chosen over the manual
`ComponentTrait` implementations used in other transitional examples like `reversi`.

## Purpose and pattern

This example demonstrates a two-player, perfect-information board game with a single shared
entity for all game state. It showcases Cougr's `SimpleWorld` entity/component storage driven
through the macro-generated `RichComponentTrait`/`ComponentTrait` implementations
(`impl_rich_component!`, `impl_component!`) and the `SorobanGame` trait
(`impl_soroban_game!`), which together remove the need to hand-write `serialize`/`deserialize`
or repeat the storage key in every contract function.

## Public contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `init_game` | `player_x: Address`, `player_o: Address` | `GameState` | Spawns the single game entity, seeds an empty 9-cell board, and sets X to move first. Overwrites any previous game. |
| `make_move` | `player: Address`, `position: u32` | `MoveResult` | Validates and applies a move at `position` (0–8). Returns `success: false` with a status `message` symbol instead of panicking on illegal input. |
| `get_state` | — | `GameState` | Current board, both player addresses, whose turn it is, move count, and status. |
| `is_valid_move` | `position: u32` | `bool` | Whether `position` is a legal move right now (in range, empty cell, game still in progress). |
| `get_winner` | — | `Option<Address>` | The winning player's address, or `None` if the game is in progress or drawn. |
| `reset_game` | — | `GameState` | Re-initializes the board with the same two players, discarding moves. |

### Status codes

`GameState.status`: `0` = in progress, `1` = X wins, `2` = O wins, `3` = draw.

`MoveResult.message`: `ok`, `invalid` (out of range), `occupied`, `notturn`, `notplay`,
`gameover`.

## Architecture overview

```
src/
├── lib.rs          # #[contract] struct, SorobanGame wiring, #[contractimpl] entrypoints
├── components.rs    # Board, Players, TurnState component structs + macro invocations
└── systems.rs       # detect_winner: pure win/draw detection over board cells
```

`make_move` runs a fixed validation/execution sequence synchronously on each call — there is
no `GameApp` tick loop:

```
make_move
  └─ load Players, TurnState, Board from the single game entity
  └─ validate: game not over, position in range, caller is a player, caller's turn, cell empty
  └─ apply the mark, call systems::detect_winner over the updated board
  └─ persist updated Board and TurnState, save the world
```

`components.rs` owns the three `SimpleWorld` components and the shared `GAME_ENTITY`
constant; `systems.rs` holds the one pure function (`detect_winner`) that contains no
storage access; `lib.rs` owns the `#[contract]` struct, the `SorobanGame` wiring, and is the
only module that loads/saves the `SimpleWorld`.

## Storage model

All state lives in **instance storage**, managed transparently by `impl_soroban_game!`
through `SorobanGame::load_world`/`save_world` under a single key (`"ttt_world"`). Within
that one `SimpleWorld`, all three components (`Board`, `Players`, `TurnState`) are attached
to a single fixed entity id (`GAME_ENTITY = 1`), since there is exactly one game per contract
instance and no dynamic entity population to query. Instance storage is appropriate here
because the game state must live for the lifetime of the contract instance with no per-entry
TTL management.

## Main gameplay flow

1. Deployer calls `init_game(player_x, player_o)`; a new entity is spawned, the board is
   seeded with 9 empty cells, and `TurnState` is set to X's turn, move count 0, status
   in-progress.
2. X calls `make_move(player_x, position)`. The contract checks the game isn't over, the
   position is in range, the caller is a registered player, and it's their turn.
3. On a legal move, the cell is marked, `systems::detect_winner` re-checks all 8 winning
   lines plus the draw condition, and turn state is updated (status, move count, whose turn
   is next).
4. Players alternate `make_move` calls until `detect_winner` reports a win (status `1` or
   `2`) or a draw (status `3`), after which further moves return `gameover`.
5. Either player can call `get_state` or `get_winner` to read the outcome, or `reset_game` to
   start over with the same two players.

## Cougr APIs used

- `cougr_core::{impl_rich_component!}` — used for `Board` (holds a `Vec<u32>`) and `Players`
  (holds two `Address` values). Both fields require Soroban's XDR codec rather than fixed-size
  byte packing, so `impl_rich_component!` was chosen to get `RichComponentTrait` for free from
  the `#[contracttype]` derive, avoiding a hand-written `serialize`/`deserialize` pair for
  `Vec<u32>` and `Address` like the one `reversi`'s `components.rs` still carries.
- `cougr_core::{impl_component!}` — used for `TurnState`, which is three fixed-size plain
  fields (`bool`, `u32`, `u32`). `impl_component!` generates a compact, fully typed
  `ComponentTrait` implementation (byte-packed, not XDR) since there are no `Address`/`Vec`
  fields needing the heavier rich-component codec.
- `cougr_core::game::SorobanGame` / `impl_soroban_game!` — generates `load_world`/
  `save_world` for the `#[contract]` struct so every entrypoint can read and persist the
  `SimpleWorld` without repeating the instance-storage key (`"ttt_world"`) or its
  get/set boilerplate.
- `cougr_core::simple_world::SimpleWorld` — used as the single source of truth for the game's
  three components, attached to one fixed entity rather than a dynamic population, since
  tic-tac-toe has exactly one game per contract instance.

This example does not use `GameApp`, `ScheduleStage`, `SimpleQueryBuilder`, `auth`, or
`privacy` — see Known limitations.

## Build and test commands

```bash
cargo test
stellar contract build
```

## Known limitations

- Does not use `GameApp` or `ScheduleStage` — validation and win detection run synchronously
  inside `make_move` since a turn-based game with one decision point per call has no need for
  staged scheduling.
- Does not use `SimpleQueryBuilder` — there is exactly one game entity per contract instance,
  so there is no entity population to scan by component type.
- No timeout/forfeit mechanism for an unresponsive player.
- No spectator or replay API beyond the read-only getters.
