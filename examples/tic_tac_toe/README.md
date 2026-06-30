# tic_tac_toe

**Canonical** example demonstrating turn-based logic, `impl_rich_component!` (for Address and Vec fields), and `impl_soroban_game!`.

## Purpose and pattern

This example implements a standard Tic Tac Toe game on-chain. It showcases how to store complex types like `Address` and `Vec` using `impl_rich_component!` and standard components via `impl_component!`. It demonstrates player-turn validation, win condition detection, and game state management.

## Public contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `init_game` | `player_x: Address`, `player_o: Address` | `GameState` | Starts a new game with the two specified players. |
| `make_move` | `player: Address`, `position: u32` | `MoveResult` | Places the player's mark at `position` (0-8) if valid. |
| `get_state` | - | `GameState` | Retrieves the current board state, players, and turn info. |
| `is_valid_move` | `position: u32` | `bool` | Checks if a cell is empty and inside the grid boundaries. |
| `get_winner` | - | `Option<Address>` | Returns the winner's Address, if any player has won. |
| `reset_game` | - | `GameState` | Resets the game cells and turn count. |

## Architecture overview

```
                    ┌────────────────────────┐
                    │   tic_tac_toe Client   │
                    └───────────┬────────────┘
                                │ Calls
                     ┌──────────▼──────────┐
                     │ TicTacToeContract   │
                     │ (Soroban Contract)  │
                     └──────────┬──────────┘
                                │ Loads / Saves
                     ┌──────────▼──────────┐
                     │     SimpleWorld     │
                     └──────────┬──────────┘
                                │ Stores
               ┌────────────────┼────────────────┐
        ┌──────▼──────┐   ┌──────▼──────┐   ┌──────▼──────┐
        │    Board    │   │   Players   │   │  TurnState  │
        │   (Rich)    │   │   (Rich)    │   │ (Standard)  │
        └─────────────┘   └─────────────┘   └─────────────┘
```

When `make_move` is called, the contract loads the board, player registry, and turn states. It verifies turn order, checks position occupancy, updates cells, evaluates win paths, and saves the updated state.

## Storage model

Game state components are stored in Soroban **Instance Storage** via `SimpleWorld`. Using instance storage is optimal for this low-footprint turn-based contract since the full state is read and written in a single batch.

## Main gameplay flow

1. **Initialization**: Call `init_game(player_x, player_o)` to register player addresses and spawn the game entity.
2. **Play Loop**: Players alternate calling `make_move(player, position)` (0-8) to place marks.
3. **End Phase**: The game reaches status 1 (X wins), 2 (O wins), or 3 (draw). No further moves are allowed.

## Cougr APIs used

- `impl_rich_component!`: Used for `Board` and `Players` which store `Vec` and `Address` types that require XDR serialization.
- `impl_component!`: Used for flat, primitive fields inside `TurnState` to bypass XDR overhead.
- `impl_soroban_game!`: Attaches `load_world` and `save_world` loader utilities to the contract.

## Recommended testing approach

Scenario-based testing with `GameHarness` and `Scenario` is the recommended method. The tests generate mock players, simulate player turns sequentially, and verify intermediate states (e.g., mark placement, illegal out-of-turn moves, and game resolution).

## Build and test commands

```bash
cargo test
stellar contract build
```

## Known limitations

- Only supports one concurrent active game per contract deployment (uses a single global game entity ID).
- No stake/escrow mechanism (pure game logic only).
