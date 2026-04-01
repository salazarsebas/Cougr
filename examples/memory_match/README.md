# Memory Match Contract

A memory match card game implemented as a Soroban smart contract using the Cougr ECS framework.

## Overview

The Memory Match contract implements a classic memory card game where players flip cards to find matching pairs. The game features:

- 16 cards arranged in a 4x4 grid (8 matching pairs)
- Turn-based gameplay with card revealing mechanics
- Match detection and automatic card hiding for non-matches
- Game state tracking and reset functionality
- Player authorization to prevent unauthorized access

## Architecture

The contract is built using the Cougr ECS (Entity Component System) framework with the following components:

### Components

- **CardComponent**: Represents individual cards with their state (Hidden, Revealed, Matched) and value
- **BoardComponent**: Manages the game board, tracking revealed cards and matched pairs
- **GameStateComponent**: Tracks overall game state including player, moves, and game over status

### World State

The `ECSWorldState` struct encapsulates all game components and provides methods for:
- Card lookup and state updates
- Game state management
- Serialization/deserialization for contract storage

## Contract Functions

### Core Functions

- `init_game(env: Env, player: Address) -> GameState`
  - Initializes a new game with the specified player
  - Returns the initial game state

- `reveal_card(env: Env, player: Address, position: u32) -> RevealInfo`
  - Reveals a card at the specified position
  - Handles match detection and game logic
  - Returns information about the reveal operation

- `get_game_state(env: Env) -> GameState`
  - Returns the current game state
  - Useful for UI updates and state queries

- `reset_game(env: Env, player: Address) -> GameState`
  - Resets the game to initial state
  - Only authorized players can reset

### Game Rules

1. Players can only reveal 2 cards at a time
2. After revealing 2 cards, they are automatically processed:
   - If matching: cards remain revealed and marked as matched
   - If not matching: cards are hidden again
3. The game ends when all 8 pairs are found
4. Only the initializing player can make moves or reset the game

## Card Layout

The game uses a deterministic card layout for testing and consistency:

```
Positions:  0  1  2  3  4  5  6  7
Values:    0  1  2  3  4  5  6  7

Positions:  8  9 10 11 12 13 14 15
Values:    0  1  2  3  4  5  6  7
```

Cards at positions (0,8), (1,9), (2,10), etc., form matching pairs.

## Building and Testing

### Prerequisites

- Rust toolchain
- Soroban CLI tools
- Cougr framework dependencies

### Build

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

### Test Coverage

The contract includes comprehensive tests covering:

- Game initialization
- Card revealing mechanics
- Match detection
- Non-match handling
- Game reset functionality
- Error conditions (unauthorized access, invalid positions, etc.)
- Edge cases and game completion

## Usage Example

```rust
use soroban_sdk::{Env, Address};
use memory_match::MemoryMatchContractClient;

// Initialize environment and contract
let env = Env::default();
let contract_id = env.register(MemoryMatchContract, ());
let client = MemoryMatchContractClient::new(&env, &contract_id);

// Create player and initialize game
let player = Address::generate(&env);
let game_state = client.init_game(&player);

// Reveal first card
let reveal_info = client.reveal_card(&player, &0);
println!("Card value: {}", reveal_info.value);

// Reveal matching card
let match_info = client.reveal_card(&player, &8);
assert!(matches!(match_info.result, RevealResult::MatchFound));
```

## Data Structures

### GameState

```rust
pub struct GameState {
    pub board_state: Vec<u32>,  // 0=Hidden, 1-8=Revealed, 9=Matched
    pub revealed_count: u32,
    pub matched_pairs: u32,
    pub total_pairs: u32,
    pub moves_count: u32,
    pub game_over: bool,
}
```

### RevealInfo

```rust
pub struct RevealInfo {
    pub result: RevealResult,
    pub position: u32,
    pub value: u32,
    pub positions: Vec<u32>,
}
```

### RevealResult

```rust
pub enum RevealResult {
    CardRevealed,
    MatchFound,
    NoMatch,
    GameOver,
}
```

## Storage

The contract uses Soroban's instance storage to persist the `ECSWorldState`. The state is stored under a fixed key and includes all game components, allowing the game to be resumed across contract invocations.

## Security Considerations

- Player authorization ensures only the game creator can make moves
- Input validation prevents invalid card positions
- State consistency is maintained through atomic updates
- No external dependencies or privileged operations

## Future Enhancements

Potential improvements for future versions:

- Multiple game modes (different grid sizes, card counts)
- Score tracking and leaderboards
- Time limits or move counters
- Multiplayer support
- Card shuffling for random layouts
- Visual themes and customization

## License

This contract is part of the Cougr framework examples and follows the same licensing terms.
