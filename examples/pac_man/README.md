# Pac-Man On-Chain Game

A practical example demonstrating how to use the `cougr-core` package to implement on-chain game logic on the Stellar blockchain via Soroban.

## Overview

This example implements a complete Pac-Man game as a Soroban smart contract. The focus is exclusively on the smart contract logic (without graphical interface), showcasing how `cougr-core` simplifies on-chain game development.

### Features

- 10x10 maze with walls, pellets, and power pellets
- Pac-Man movement with direction control
- 4 ghosts with simplified AI (chase and frightened modes)
- Score tracking and lives system
- Win/lose conditions
- Complete test coverage

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Pac-Man Contract                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  GameState  │  │   Maze      │  │    Ghosts[4]        │  │
│  │  - score    │  │  - cells[]  │  │  - position         │  │
│  │  - lives    │  │  - 10x10    │  │  - direction        │  │
│  │  - game_over│  │             │  │  - mode             │  │
│  │  - won      │  │             │  │  - frightened_timer │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Pac-Man   │  │  Constants  │  │   Storage Keys      │  │
│  │  - position │  │  - PELLET=10│  │  - GameState        │  │
│  │  - direction│  │  - POWER=50 │  │  - Initialized      │  │
│  │  - start_pos│  │  - GHOST=200│  │                     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                    Contract Functions                       │
│  ┌──────────────┐ ┌────────────────┐ ┌──────────────────┐  │
│  │  init_game   │ │change_direction│ │   update_tick    │  │
│  └──────────────┘ └────────────────┘ └──────────────────┘  │
│  ┌──────────────┐ ┌────────────────┐ ┌──────────────────┐  │
│  │  eat_pellet  │ │   get_score    │ │ check_game_over  │  │
│  └──────────────┘ └────────────────┘ └──────────────────┘  │
│  ┌──────────────┐ ┌────────────────┐ ┌──────────────────┐  │
│  │  get_lives   │ │get_pacman_pos  │ │   get_maze       │  │
│  └──────────────┘ └────────────────┘ └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Maze Layout

The game uses a fixed 10x10 maze:

```
##########
#P......P#
#.##.##..#
#.#...#..#
#...#....#
#.#.#.##.#
#.#......#
#.##.###.#
#P......P#
##########

Legend:
  # = Wall
  . = Pellet (10 points)
  P = Power Pellet (50 points)
```

## Prerequisites

### Required Tools

1. **Rust** (1.70.0 or later)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup update
   ```

2. **Stellar CLI**
   ```bash
   cargo install stellar-cli --locked
   stellar --version
   ```

3. **WASM Target**
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

### Verify Installation

```bash
rustc --version    # Should be 1.70.0 or later
cargo --version
stellar --version  # Should show Stellar CLI version
```

## Setup

### 1. Clone the Repository

```bash
git clone https://github.com/salazarsebas/Cougr.git
cd Cougr/examples/pac_man
```

### 2. Update Dependencies

```bash
cargo update
```

### 3. Build the Contract

```bash
# Debug build
cargo build

# Release build (optimized for WASM)
cargo build --release
```

### 4. Build for Soroban

```bash
stellar contract build
```

This generates the WASM file at:
```
target/wasm32-unknown-unknown/release/pac_man.wasm
```

## Testing

### Run All Tests

```bash
cargo test
```

### Run Tests with Output

```bash
cargo test -- --nocapture
```

### Run Specific Test

```bash
cargo test test_init_game
cargo test test_pellet_collection
cargo test test_ghost_ai
```

### Expected Output

```
running 25 tests
test test::test_init_game ... ok
test test::test_maze_layout ... ok
test test::test_ghosts_initialized ... ok
test test::test_change_direction_up ... ok
test test::test_change_direction_down ... ok
...
test result: ok. 25 passed; 0 failed; 0 ignored
```

## Code Walkthrough

### 1. Contract Types

The game uses Soroban-compatible types with `#[contracttype]`:

```rust
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct GameState {
    pub pacman_pos: Position,
    pub pacman_dir: Direction,
    pub ghosts: Vec<Ghost>,
    pub maze: Vec<CellType>,
    pub score: u32,
    pub lives: u32,
    pub game_over: bool,
    // ...
}
```

### 2. Storage Pattern

Game state is stored using Soroban's instance storage:

```rust
#[contracttype]
pub enum DataKey {
    GameState,
    Initialized,
}

// Writing state
env.storage().instance().set(&DataKey::GameState, &state);

// Reading state
let state: GameState = env.storage()
    .instance()
    .get(&DataKey::GameState)
    .expect("Game not initialized");
```

### 3. Cougr-Core Integration

The contract imports cougr-core for ECS patterns:

```rust
use cougr_core::*;
```

This provides access to:
- Component patterns for game entities
- Event system for game actions
- Storage utilities optimized for Soroban

### 4. Game Loop

The `update_tick` function implements the main game loop:

```rust
pub fn update_tick(env: Env) -> GameState {
    let mut state = Self::get_state(&env);

    // 1. Move Pac-Man
    Self::move_pacman(&env, &mut state);

    // 2. Check pellet collection
    Self::check_pellet_collection(&env, &mut state);

    // 3. Move ghosts
    Self::move_ghosts(&env, &mut state);

    // 4. Check collisions
    Self::check_ghost_collisions(&env, &mut state);

    // 5. Update timers
    if state.power_mode_timer > 0 {
        state.power_mode_timer -= 1;
    }

    // 6. Check win condition
    if state.pellets_remaining == 0 {
        state.game_over = true;
        state.won = true;
    }

    env.storage().instance().set(&DataKey::GameState, &state);
    state
}
```

## Deployment

### 1. Generate a Stellar Identity

```bash
stellar keys generate --global pacman-deployer --network testnet
```

### 2. Fund the Account

```bash
stellar keys fund pacman-deployer --network testnet
```

Or use Friendbot:
```bash
curl "https://friendbot.stellar.org?addr=$(stellar keys address pacman-deployer)"
```

### 3. Deploy the Contract

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/pac_man.wasm \
  --source pacman-deployer \
  --network testnet
```

Save the returned contract ID (e.g., `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC`).

### 4. Initialize the Game

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source pacman-deployer \
  --network testnet \
  -- \
  init_game
```

### 5. Play the Game

Change direction:
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source pacman-deployer \
  --network testnet \
  -- \
  change_direction \
  --direction 2  # 0=Up, 1=Down, 2=Left, 3=Right
```

Update game state:
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source pacman-deployer \
  --network testnet \
  -- \
  update_tick
```

### 6. Query Game State

Get score:
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  -- \
  get_score
```

Get full state:
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  -- \
  get_game_state
```

Check if game is over:
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  -- \
  check_game_over
```

## Troubleshooting

### Common Errors

#### "Game not initialized"
Call `init_game` before any other function.

#### "Game already initialized"
Each contract instance can only be initialized once. Deploy a new instance to start a fresh game.

#### "Game is over"
The game has ended. Deploy a new contract to play again.

#### Rust Version Errors
```bash
rustup update
rustup default stable
```

#### WASM Build Fails
```bash
rustup target add wasm32-unknown-unknown
```

#### Dependency Resolution Issues
```bash
cargo clean
cargo update
cargo build
```

#### Stellar CLI Not Found
```bash
cargo install stellar-cli --locked
```

### Debug Tips

1. **Verbose Build**
   ```bash
   cargo build --verbose
   ```

2. **Check Contract Size**
   ```bash
   ls -la target/wasm32-unknown-unknown/release/pac_man.wasm
   ```
   Should be under 64KB.

3. **Simulate Before Deploy**
   ```bash
   stellar contract invoke \
     --id <CONTRACT_ID> \
     --source pacman-deployer \
     --network testnet \
     --sim \
     -- \
     init_game
   ```

## Game Mechanics

### Scoring
| Action | Points |
|--------|--------|
| Regular Pellet | 10 |
| Power Pellet | 50 |
| Eat Ghost | 200 |

### Lives
- Start with 3 lives
- Lose 1 life when caught by a ghost in chase mode
- Game over when lives reach 0

### Power Mode
- Lasts 10 ticks
- All ghosts enter frightened mode
- Eating a ghost awards 200 points and respawns the ghost

### Ghost AI
- **Chase Mode**: Ghosts move toward Pac-Man using Manhattan distance
- **Frightened Mode**: Ghosts move away from Pac-Man

### Win Condition
Collect all pellets (regular and power) to win.

## Testnet Deployment

The contract has been successfully deployed to Stellar Testnet:

**Contract ID**: `CDWERKYRRWD5N6Q7RKCVWT7BNNS5ADRRTM2VCG45AYRE52ABP5NUBJ3C`

**Explorer Link**: https://stellar.expert/explorer/testnet/contract/CDWERKYRRWD5N6Q7RKCVWT7BNNS5ADRRTM2VCG45AYRE52ABP5NUBJ3C

### Verified Invocations

1. **init_game** - Successfully initialized with:
   - Score: 0
   - Lives: 3
   - Pellets: 47
   - 4 ghosts in chase mode

2. **change_direction** - Successfully changed Pac-Man direction to Down

3. **update_tick** - Successfully processed game tick:
   - Pac-Man moved from (1,1) to (1,2)
   - Collected pellet (+10 points)
   - Score updated to 10
   - Ghosts moved toward Pac-Man

4. **get_score** - Read-only query returned: 10

## References

- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)
- [Soroban Hello World Tutorial](https://developers.stellar.org/docs/build/smart-contracts/getting-started/hello-world)
- [Stellar CLI Documentation](https://developers.stellar.org/docs/tools/cli)
- [Cougr Repository](https://github.com/salazarsebas/Cougr)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)

## License

MIT OR Apache-2.0
