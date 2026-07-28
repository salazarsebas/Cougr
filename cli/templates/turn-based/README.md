# {{crate_name}}

{{description}}

Generated with `cougr new --template {{template_id}}`, based on the canonical
[`{{source_example}}`](https://github.com/salazarsebas/Cougr/tree/main/examples/{{source_example}})
example.

## Purpose and pattern

Two players alternate placing marks on a 3×3 board until one lines up three or
the board fills. It is the reference shape for any turn-based game on Cougr:
exactly one match per contract instance, an explicit turn owner, and every move
validated before it is written to storage.

## Public contract API

| Function | Parameters | Returns | Description |
| --- | --- | --- | --- |
| `init_game` | `player_x: Address`, `player_o: Address` | `GameState` | Start a fresh match, discarding previous state |
| `make_move` | `player: Address`, `position: u32` | `MoveResult` | Place the caller's mark at `0`–`8` |
| `get_state` | — | `GameState` | Full board, players, turn, and status |
| `is_valid_move` | `position: u32` | `bool` | Whether that cell is playable right now |
| `get_winner` | — | `Option<Address>` | Winner's address, or `None` while running or drawn |
| `reset_game` | — | `GameState` | Clear the board, keep the same players |

`MoveResult.status` and `GameState.status` use the constants in
`components.rs`: `0` in progress, `1` X wins, `2` O wins, `3` draw.

## Architecture overview

```
lib.rs         contract entrypoints — load world, validate, write, save world
  ├─ components.rs   Board + Players (rich), TurnState (plain), status constants
  └─ systems.rs      validate_move(), advance(), detect_status() — pure rules
```

`make_move` never applies a rule itself: it loads the world, asks
`systems::validate_move` whether the move is legal, and only then writes. An
illegal move returns `success: false` with a short reason code instead of
panicking, so a client can display the rejection without losing state.

## Storage model

The whole `SimpleWorld` lives in **instance storage** under the `"world"` key,
wired by `impl_soroban_game!({{ContractName}}, "world")`. Because there is one
match per instance, all three components hang off a single fixed entity
(`GAME_ENTITY`), so any call is one storage read and at most one write.

`Board` and `Players` are declared with `impl_rich_component!` because `Vec` and
`Address` need an XDR codec; `TurnState` is fixed-size scalars and uses the
cheaper `impl_component!`.

## Main gameplay flow

1. Someone calls `init_game` with both player addresses — X moves first.
2. X calls `make_move`; the contract checks the match is running, the cell is in
   range and empty, and the caller owns the turn.
3. The mark is written, the move count increases, and `detect_status` re-checks
   all eight lines.
4. Turn ownership flips while the status is still "in progress"; once a player
   wins or the board fills, further moves are rejected with `gameover`.
5. `reset_game` clears the board and keeps both players for a rematch.

## Cougr APIs used

| API | Why |
| --- | --- |
| `impl_rich_component!` | `Board` (`Vec<u32>`) and `Players` (`Address`) need XDR storage |
| `impl_component!` | `TurnState` is three scalars — no codec required |
| `SorobanGame` / `impl_soroban_game!` | Removes hand-written world load/save boilerplate |
| `SimpleWorld` | Holds all three components under one entity |
| `test::GameHarness`, `Scenario` | Drives alternating turns in an integration test |

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

* `make_move` calls `require_auth` on the acting player, but `init_game` and
  `reset_game` are open to any caller — add access control before deploying.
* One match per contract instance. A lobby of concurrent games needs one entity
  per match instead of the fixed `GAME_ENTITY`.
* No draw offers, resignations, timeouts, or stakes.
