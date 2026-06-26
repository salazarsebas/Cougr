# Tic Tac Toe On-Chain Game

A fully functional Tic Tac Toe game implemented as a Soroban smart contract on the Stellar blockchain, demonstrating the **Cougr-Core** ECS (Entity Component System) framework for on-chain gaming.

## Cougr-Core ECS Integration

All game components implement `cougr_core::component::ComponentTrait`:

```rust
impl ComponentTrait for BoardComponent {
    fn component_type() -&gt; Symbol {
        symbol_short!("board")
    }

    fn serialize(&self, env: &Env) -&gt; Bytes { /* ... */ }
    fn deserialize(env: &Env, data: &Bytes) -&gt; Option&lt;Self&gt; { /* ... */ }
}


## ECS System Pattern
Game logic is organized into discrete systems:
| System                 | Responsibility                                    |
| ---------------------- | ------------------------------------------------- |
| `validation_system`    | Enforces game rules (turn order, valid positions) |
| `execution_system`     | Applies moves to the board                        |
| `win_detection_system` | Checks all 8 winning patterns                     |
| `turn_system`          | Manages turn transitions                          |


# Features
| Feature              | Description                                            |
| -------------------- | ------------------------------------------------------ |
| Two-player gameplay  | Uses Stellar addresses for player identification       |
| Turn-based mechanics | X always goes first, enforced turn order               |
| Win detection        | All 8 patterns (3 rows, 3 columns, 2 diagonals)        |
| Draw detection       | Recognizes full board with no winner                   |
| Move validation      | Rejects invalid positions, occupied cells, wrong turns |
| Game reset           | Restart with same players                              |


# Prerequisites
| Requirement | Version               |
| ----------- | --------------------- |
| Rust        | 1.70.0+               |
| Stellar CLI | 25.0.0+ (recommended) |
cargo install stellar-cli

## Building
# Build for testing
cargo build

# Build optimized WASM
stellar contract build

## Testing
cargo test
| Test Category  | Count  | Coverage                                        |
| -------------- | ------ | ----------------------------------------------- |
| Initialization | 2      | Game setup, state retrieval                     |
| Valid moves    | 3      | X moves, O moves, position validation           |
| Invalid moves  | 5      | Wrong turn, occupied, out of bounds, non-player |
| Win conditions | 8      | All rows, columns, diagonals                    |
| Draw           | 2      | Full board, post-draw state                     |
| Game over      | 2      | No moves after win/draw                         |
| Reset          | 2      | Mid-game reset, post-win reset                  |
| State          | 3      | Persistence, move counting, winner retrieval    |
| **Total**      | **33** | **All passing**                                 |


## Contract API
# Functions
| Function        | Parameters                             | Returns           | Description             |
| --------------- | -------------------------------------- | ----------------- | ----------------------- |
| `init_game`     | `player_x: Address, player_o: Address` | `GameState`       | Initialize new game     |
| `make_move`     | `player: Address, position: u32`       | `MoveResult`      | Make a move (0-8)       |
| `get_state`     | —                                      | `GameState`       | Get current state       |
| `is_valid_move` | `position: u32`                        | `bool`            | Check if move is valid  |
| `get_winner`    | —                                      | `Option<Address>` | Get winner's address    |
| `reset_game`    | —                                      | `GameState`       | Reset with same players |


## Board Positions
 0 | 1 | 2
-----------
 3 | 4 | 5
-----------
 6 | 7 | 8

 ## Data Structures
 # GameState
 | Field        | Type       | Description                            |
| ------------ | ---------- | -------------------------------------- |
| `cells`      | `Vec<u32>` | Board state (0=Empty, 1=X, 2=O)        |
| `player_x`   | `Address`  | Player X's address                     |
| `player_o`   | `Address`  | Player O's address                     |
| `is_x_turn`  | `bool`     | True if X's turn                       |
| `move_count` | `u32`      | Total moves made                       |
| `status`     | `u32`      | 0=InProgress, 1=XWins, 2=OWins, 3=Draw |

# MoveResult
| Field        | Type        | Description            |
| ------------ | ----------- | ---------------------- |
| `success`    | `bool`      | Whether move succeeded |
| `game_state` | `GameState` | Updated state          |
| `message`    | `Symbol`    | Status code            |

# Error Messages
| Code       | Meaning                          |
| ---------- | -------------------------------- |
| `ok`       | Move successful                  |
| `invalid`  | Position out of bounds (not 0-8) |
| `occupied` | Cell already has a mark          |
| `notturn`  | Not the player's turn            |
| `notplay`  | Address is not a player          |
| `gameover` | Game has already ended           |


## Architecture
ECSWorldState
├── BoardComponent     (entity_id: 0)
│   └── cells: Vec<u32> [9 cells]
├── PlayerComponent    (entity_id: 1)
│   ├── player_x: Address
│   └── player_o: Address
├── GameStateComponent (entity_id: 2)
│   ├── is_x_turn: bool
│   ├── move_count: u32
│   └── status: u32
└── next_entity_id: u32

## Deployment
# Deploy to Testnet
# Generate funded account
stellar keys generate deployer --network <NETWORK> --fund

# Build contract
stellar contract build

# Deploy
stellar contract deploy \
  --wasm target/tic_tac_toe.wasm \
  --source <ACCOUNT> \
  --network <NETWORK>

  # Interact with Deployed Contract
  # Initialize a game
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network <NETWORK> \
  -- init_game \
  --player_x <PLAYER_X_ADDRESS> \
  --player_o <PLAYER_O_ADDRESS>

# Make a move
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network <NETWORK> \
  -- make_move \
  --player <PLAYER_ADDRESS> \
  --position 4

# Get game state
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network <NETWORK> \
  -- get_state



## Resources
  Cougr Repository
Soroban Documentation
Stellar CLI Reference


---

## 5. Verification Script: `scripts/verify_hygiene.sh`

```bash
#!/usr/bin/env bash
#
# verify_hygiene.sh — Verify all hygiene standards from #225 are met
#

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EXAMPLES_DIR="$REPO_ROOT/examples"
EXIT_CODE=0

echo "=== Hygiene Verification (#225) ==="
echo ""

# --- Check 1: No tracked target/ directories ---
echo "Check 1: No tracked target/ directories"
tracked_targets=$(git -C "$REPO_ROOT" ls-files 'examples/**/target/**' 2>/dev/null || true)
if [ -z "$tracked_targets" ]; then
    echo "  PASS: No target/ files tracked"
else
    echo "  FAIL: Tracked target/ files found:"
    echo "$tracked_targets" | sed 's/^/    /'
    EXIT_CODE=1
fi
echo ""

# --- Check 2: No .wasm files tracked ---
echo "Check 2: No tracked .wasm files"
tracked_wasm=$(git -C "$REPO_ROOT" ls-files 'examples/**/*.wasm' 2>/dev/null || true)
if [ -z "$tracked_wasm" ]; then
    echo "  PASS: No .wasm files tracked"
else
    echo "  FAIL: Tracked .wasm files found:"
    echo "$tracked_wasm" | sed 's/^/    /'
    EXIT_CODE=1
fi
echo ""

# --- Check 3: No hardcoded contract IDs in READMEs ---
echo "Check 3: No hardcoded contract IDs in README.md files"
contract_ids=$(grep -rE 'C[A-Z2-7]{55}' "$EXAMPLES_DIR"/*/README.md 2>/dev/null || true)
if [ -z "$contract_ids" ]; then
    echo "  PASS: No hardcoded contract IDs found"
else
    echo "  FAIL: Hardcoded contract IDs found:"
    echo "$contract_ids" | sed 's/^/    /'
    EXIT_CODE=1
fi
echo ""

# --- Check 4: .gitignore exists in every example ---
echo "Check 4: .gitignore in every example directory"
missing_gitignore=0
for example_dir in "$EXAMPLES_DIR"/*/; do
    if [ ! -f "$example_dir/.gitignore" ]; then
        echo "  FAIL: Missing .gitignore in $(basename "$example_dir")"
        missing_gitignore=$((missing_gitignore + 1))
        EXIT_CODE=1
    fi
done
if [ $missing_gitignore -eq 0 ]; then
    echo "  PASS: All examples have .gitignore"
fi
echo ""

# --- Check 5: .gitignore excludes target/ ---
echo "Check 5: .gitignore excludes target/"
missing_target_ignore=0
for gitignore in "$EXAMPLES_DIR"/*/.gitignore; do
    if [ ! -f "$gitignore" ]; then
        continue
    fi
    if ! grep -q "^target/" "$gitignore" && ! grep -q "^/target/" "$gitignore"; then
        echo "  FAIL: $(dirname "$gitignore")/.gitignore does not exclude target/"
        missing_target_ignore=$((missing_target_ignore + 1))
        EXIT_CODE=1
    fi
done
if [ $missing_target_ignore -eq 0 ]; then
    echo "  PASS: All .gitignore files exclude target/"
fi
echo ""

# --- Check 6: cargo metadata --no-deps succeeds ---
echo "Check 6: cargo metadata --no-deps succeeds for all examples"
metadata_failed=0
for example_dir in "$EXAMPLES_DIR"/*/; do
    if [ ! -d "$example_dir" ]; then
        continue
    fi
    example_name=$(basename "$example_dir")
    if (cd "$example_dir" && cargo metadata --no-deps --format-version 1 >/dev/null 2>&1); then
        :
    else
        echo "  FAIL: cargo metadata failed for $example_name"
        metadata_failed=$((metadata_failed + 1))
        EXIT_CODE=1
    fi
done
if [ $metadata_failed -eq 0 ]; then
    echo "  PASS: cargo metadata succeeds for all examples"
fi
echo ""

# --- Summary ---
if [ $EXIT_CODE -eq 0 ]; then
    echo "=== ALL CHECKS PASSED ==="
else
    echo "=== SOME CHECKS FAILED ==="
fi

exit $EXIT_CODE