# Minesweeper On-Chain Game

A fully functional Minesweeper game implemented as a Soroban smart contract on the Stellar blockchain, demonstrating the **Cougr-Core** ECS (Entity Component System) framework for on-chain gaming.

|                 |                                                                                                                                       |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------- |

## Why Cougr-Core?

Cougr-Core provides an ECS architecture that simplifies on-chain game development with modular, testable systems.

### ComponentTrait Integration

All game components implement `cougr_core::component::ComponentTrait`:

```rust
impl ComponentTrait for BoardComponent {
    fn component_type() -> Symbol {
        symbol_short!("board")
    }

    fn serialize(&self, env: &Env) -> Bytes { /* ... */ }
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> { /* ... */ }
}
```

### ECS System Pattern

Game logic is organized into discrete systems:

| System               | Responsibility                                          |
| -------------------- | ------------------------------------------------------- |
| `RevealSystem`       | Handles cell revelation and mine detection              |
| `AdjacencySystem`    | Calculates adjacent mine count for each cell            |
| `LossSystem`         | Detects mine reveal and triggers game over              |
| `CompletionSystem`   | Checks if all safe cells are revealed (win condition)   |

## Features

| Feature                | Description                                                    |
| ---------------------- | -------------------------------------------------------------- |
| Deterministic layout   | Fixed mine positions for verifiability                         |
| Cell reveal mechanics  | Reveal cells with adjacent mine count feedback                 |
| Adjacency counting     | Shows number of mines in 8 neighboring cells                   |
| Win detection          | Tracks when all safe cells are revealed                        |
| Loss detection         | Game ends immediately on mine reveal                           |
| Compact board state    | 9×9 grid optimized for on-chain storage                        |
| Proof-friendly         | Deterministic layout enables verification                      |

## Prerequisites

| Requirement | Version               |
| ----------- | --------------------- |
| Rust        | 1.70.0+               |
| Stellar CLI | 25.0.0+ (recommended) |

```bash
cargo install stellar-cli
```

## Building

```bash
# Build for testing
cargo build

# Build optimized WASM
stellar contract build
```

## Testing

```bash
cargo test
```

| Test Category              | Count | Coverage                                              |
| -------------------------- | ----- | ----------------------------------------------------- |
| Initialization             | 2     | Game setup, board retrieval                           |
| Safe cell reveal           | 3     | Basic reveal, adjacent count, counter increment       |
| Mine reveal                | 2     | Hit mine, game over                                   |
| Repeated reveals           | 2     | Already revealed, multiple different cells            |
| Out of bounds              | 1     | Invalid coordinates                                   |
| Win condition              | 2     | State tracking, after loss                            |
| Visible cell state         | 4     | Hidden, revealed, mine revealed, out of bounds        |
| Reset game                 | 1     | Mid-game reset                                        |
| Adjacent count verification| 1     | Multiple cells across board                           |
| Edge cases                 | 3     | Corners, game over, deterministic placement           |
| **Total**                  | **21**| **All passing**                                        |

## Contract API

### Functions

| Function          | Parameters              | Returns            | Description                          |
| ----------------- | ----------------------- | ------------------ | ------------------------------------ |
| `init_game`       | -                       | `GameState`        | Initialize new game with fixed mines |
| `reveal_cell`     | `row: u32, col: u32`    | `RevealResult`     | Reveal cell at position              |
| `get_state`       | -                       | `GameState`        | Get current game state               |
| `get_visible_cell`| `row: u32, col: u32`    | `VisibleCellState` | Get state of specific cell           |
| `is_finished`     | -                       | `bool`             | Check if game is over                |
| `get_board`       | -                       | `Vec<u32>`         | Get full board state (debug)         |
| `reset_game`      | -                       | `GameState`        | Reset game with new layout           |

### Board Layout

```
9×9 Grid (81 cells total, 10 mines)

    0  1  2  3  4  5  6  7  8
   ┌──────────────────────────┐
0  │ .  .  .  .  .  .  .  .  . │
1  │ .  *  .  .  .  *  .  .  . │
2  │ .  .  .  .  .  .  .  *  . │
3  │ .  .  .  *  .  .  .  .  * │
4  │ .  .  .  .  .  .  *  .  . │
5  │ .  .  *  .  *  .  .  *  . │
6  │ .  .  .  .  .  .  .  .  . │
7  │ *  .  .  .  .  *  .  .  . │
8  │ .  .  .  .  .  .  .  .  . │
   └──────────────────────────┘

Legend:
  . = Hidden cell
  * = Mine (hidden until revealed)
  0-8 = Revealed cell with adjacent mine count
```

### Cell States

| Value | Meaning |
|-------|---------|
| 0-8   | Revealed cell with N adjacent mines |
| 9     | Hidden cell |
| 10    | Mine (revealed) |

### Data Structures

**GameState**
| Field                | Type    | Description                                    |
| -------------------- | ------- | ---------------------------------------------- |
| `rows`               | `u32`   | Number of rows (9)                               |
| `cols`               | `u32`   | Number of columns (9)                            |
| `total_mines`        | `u32`   | Total mines on board (10)                        |
| `status`             | `u32`   | 0=Playing, 1=Won, 2=Lost                         |
| `revealed_count`     | `u32`   | Number of safe cells revealed                    |
| `safe_cells_remaining`| `u32`  | Safe cells yet to be revealed                    |

**RevealResult**
| Field              | Type      | Description                           |
| ------------------ | --------- | ------------------------------------- |
| `success`          | `bool`    | Whether reveal succeeded              |
| `is_mine`          | `bool`    | True if cell contains mine            |
| `adjacent_mines`   | `u32`     | Count of mines in 8 neighboring cells |
| `message`          | `Symbol`  | Status code                           |

**VisibleCellState**
| Field              | Type      | Description                           |
| ------------------ | --------- | ------------------------------------- |
| `is_revealed`      | `bool`    | Whether cell is revealed              |
| `is_mine`          | `bool`    | Whether cell is a mine (if revealed)  |
| `adjacent_mines`   | `u32`     | Adjacent mine count (if revealed)     |

### Error Messages

| Code       | Meaning                                      |
| ---------- | -------------------------------------------- |
| `ok`       | Reveal successful                            |
| `invalid`  | Coordinates out of bounds (not 0-8)          |
| `revealed` | Cell already revealed                        |
| `boom`     | Hit mine - game over                         |
| `over`     | Game already finished                        |

## Architecture

```text
ECSWorldState
├── BoardComponent           (entity_id: 0)
│   └── cells: Vec<u32> [81 cells - visible state]
├── MineLayoutComponent      (entity_id: 1)
│   └── mines: Vec<u32> [81 cells - hidden mine positions]
├── GameStateComponent       (entity_id: 2)
│   ├── status: u32
│   ├── revealed_count: u32
│   └── entity_id: u32
└── next_entity_id: u32
```

### Adjacency Calculation

The contract calculates adjacent mines by checking all 8 neighbors:

```rust
fn count_adjacent_mines(&self, env: &Env, row: u32, col: u32) -> u32 {
    let mut count = 0;
    
    // Check all 8 neighbors
    for dr in -1i32..=1 {
        for dc in -1i32..=1 {
            if dr == 0 && dc == 0 { continue; }
            
            let nr = row as i32 + dr;
            let nc = col as i32 + dc;
            
            if nr >= 0 && nr < ROWS as i32 && nc >= 0 && nc < COLS as i32 {
                if self.has_mine(env, nr as u32, nc as u32) {
                    count += 1;
                }
            }
        }
    }
    
    count
}
```

### Win/Loss Conditions

**Loss:** Player reveals a cell containing a mine  
**Win:** All safe cells revealed (total cells - mines = 71 safe cells)

## Gameplay Example

```text
Initial State (all hidden):
┌──────────────────────────┐
│ #  #  #  #  #  #  #  #  # │
│ #  #  #  #  #  #  #  #  # │
│ #  #  #  #  #  #  #  #  # │
│ #  #  #  #  #  #  #  #  # │
│ #  #  #  #  #  #  #  #  # │
│ #  #  #  #  #  #  #  #  # │
│ #  #  #  #  #  #  #  #  # │
│ #  #  #  #  #  #  #  #  # │
│ #  #  #  #  #  #  #  #  # │
└──────────────────────────┘

After revealing (0,0) - shows 1 adjacent mine:
┌──────────────────────────┐
│ 1  #  #  #  #  #  #  #  # │
│ #  *  #  #  #  #  #  #  # │
│ #  #  #  #  #  #  #  #  # │
│ #  #  #  #  #  #  #  #  # │
│ #  #  #  #  #  #  #  #  # │
│ #  #  #  #  #  #  #  #  # │
│ #  #  #  #  #  #  #  #  # │
│ #  #  #  #  #  #  #  #  # │
│ #  #  #  #  #  #  #  #  # │
└──────────────────────────┘

Player hits mine at (1,1) - GAME OVER:
┌──────────────────────────┐
│ 1  💥  #  #  #  #  #  #  # │
│ #  #  #  #  #  #  #  #  # │
...
Status: Lost
```

## Deployment

### Deploy to Testnet

```bash
# Generate funded account
stellar keys generate deployer --network <NETWORK> --fund

# Build contract
stellar contract build

# Deploy
stellar contract deploy \
  --wasm target/wasm32v1-none/release/minesweeper.wasm \
  --source deployer \
  --network <NETWORK>
```

### Interact with Deployed Contract

```bash
# Initialize a game
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network <NETWORK> \
  -- init_game

# Reveal cell at row 0, col 0
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network <NETWORK> \
  -- reveal_cell \
  --row 0 \
  --col 0

# Get game state
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network <NETWORK> \
  -- get_state

# Check if game is finished
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network <NETWORK> \
  -- is_finished
```

## Strategy Tips

1. **Start from corners**: Corners have fewer neighbors, making them safer
2. **Use adjacent counts**: Numbers reveal information about surrounding cells
3. **Avoid random guesses**: Each reveal should be informed by adjacent counts
4. **Track revealed cells**: Don't waste moves on already-revealed cells

## Design Decisions

### Deterministic Mine Placement

Unlike traditional Minesweeper with random layouts, this implementation uses a **fixed, deterministic mine pattern** for several reasons:

1. **Proof-friendly**: Players can verify the layout is fair
2. **Reproducible**: Same inputs always produce same results
3. **On-chain friendly**: No need for randomness oracles
4. **Auditability**: Anyone can verify the mine positions

### Compact 9×9 Board

- **81 cells total** - manageable on-chain storage
- **10 mines (~12% density)** - beginner-friendly difficulty
- **71 safe cells** - reasonable completion goal

### No Flagging System

Flags omitted to keep scope focused on core reveal mechanics. The game emphasizes:
- Safe cell discovery
- Adjacent count interpretation
- Risk assessment

## Resources

- [Cougr Repository](https://github.com/salazarsebas/Cougr)
- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)
- [Stellar CLI Reference](https://developers.stellar.org/docs/tools/cli)
- [Minesweeper Wikipedia](https://en.wikipedia.org/wiki/Minesweeper_(video_game))
