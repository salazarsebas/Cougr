#!/usr/bin/env bash
# play.sh - Build, deploy, and solve the Sudoku contract on Stellar Testnet.
# Usage: ./scripts/play.sh
# Prereqs: stellar CLI installed, Rust with wasm32v1-none target

set -euo pipefail

NETWORK=testnet
SOURCE=sudoku_player

# ── 1. Build ──────────────────────────────────────────────────────────────────

echo "==> Building WASM..."
stellar contract build

# ── 2. Identity ───────────────────────────────────────────────────────────────

echo "==> Setting up identity..."
if ! stellar keys show "$SOURCE" &>/dev/null; then
  stellar keys generate "$SOURCE"
fi
stellar keys fund "$SOURCE" --network "$NETWORK"

# ── 3. Deploy ─────────────────────────────────────────────────────────────────

echo "==> Deploying contract..."
CONTRACT_ID=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/sudoku.wasm \
  --network "$NETWORK" \
  --source "$SOURCE")

echo "CONTRACT_ID=$CONTRACT_ID"

# ── 4. Init ───────────────────────────────────────────────────────────────────

PUZZLE='[5,3,0,0,7,0,0,0,0,6,0,0,1,9,5,0,0,0,0,9,8,0,0,0,0,6,0,8,0,0,0,6,0,0,0,3,4,0,0,8,0,3,0,0,1,7,0,0,0,2,0,0,0,6,0,6,0,0,0,0,2,8,0,0,0,0,4,1,9,0,0,5,0,0,0,0,8,0,0,7,9]'

echo "==> Initialising puzzle..."
stellar contract invoke --id "$CONTRACT_ID" --network "$NETWORK" --source "$SOURCE" \
  -- init_game --puzzle "$PUZZLE"

stellar contract invoke --id "$CONTRACT_ID" --network "$NETWORK" --source "$SOURCE" \
  -- get_state

# ── 5. Play all 51 moves ──────────────────────────────────────────────────────

INVOKE="stellar contract invoke --id $CONTRACT_ID --network $NETWORK --source $SOURCE -- submit_value"

echo "==> Submitting 51 moves..."
$INVOKE --row 0 --col 2 --value 4   # move 1
$INVOKE --row 0 --col 3 --value 6   # move 2
$INVOKE --row 0 --col 5 --value 8   # move 3
$INVOKE --row 0 --col 6 --value 9   # move 4
$INVOKE --row 0 --col 7 --value 1   # move 5
$INVOKE --row 0 --col 8 --value 2   # move 6
$INVOKE --row 1 --col 1 --value 7   # move 7
$INVOKE --row 1 --col 2 --value 2   # move 8
$INVOKE --row 1 --col 6 --value 3   # move 9
$INVOKE --row 1 --col 7 --value 4   # move 10
$INVOKE --row 1 --col 8 --value 8   # move 11
$INVOKE --row 2 --col 0 --value 1   # move 12
$INVOKE --row 2 --col 3 --value 3   # move 13
$INVOKE --row 2 --col 4 --value 4   # move 14
$INVOKE --row 2 --col 5 --value 2   # move 15
$INVOKE --row 2 --col 6 --value 5   # move 16
$INVOKE --row 2 --col 8 --value 7   # move 17
$INVOKE --row 3 --col 1 --value 5   # move 18
$INVOKE --row 3 --col 2 --value 9   # move 19
$INVOKE --row 3 --col 3 --value 7   # move 20
$INVOKE --row 3 --col 5 --value 1   # move 21
$INVOKE --row 3 --col 6 --value 4   # move 22
$INVOKE --row 3 --col 7 --value 2   # move 23
$INVOKE --row 4 --col 1 --value 2   # move 24
$INVOKE --row 4 --col 2 --value 6   # move 25
$INVOKE --row 4 --col 4 --value 5   # move 26
$INVOKE --row 4 --col 6 --value 7   # move 27
$INVOKE --row 4 --col 7 --value 9   # move 28
$INVOKE --row 5 --col 1 --value 1   # move 29
$INVOKE --row 5 --col 2 --value 3   # move 30
$INVOKE --row 5 --col 3 --value 9   # move 31
$INVOKE --row 5 --col 5 --value 4   # move 32
$INVOKE --row 5 --col 6 --value 8   # move 33
$INVOKE --row 5 --col 7 --value 5   # move 34
$INVOKE --row 6 --col 0 --value 9   # move 35
$INVOKE --row 6 --col 2 --value 1   # move 36
$INVOKE --row 6 --col 3 --value 5   # move 37
$INVOKE --row 6 --col 4 --value 3   # move 38
$INVOKE --row 6 --col 5 --value 7   # move 39
$INVOKE --row 6 --col 8 --value 4   # move 40
$INVOKE --row 7 --col 0 --value 2   # move 41
$INVOKE --row 7 --col 1 --value 8   # move 42
$INVOKE --row 7 --col 2 --value 7   # move 43
$INVOKE --row 7 --col 6 --value 6   # move 44
$INVOKE --row 7 --col 7 --value 3   # move 45
$INVOKE --row 8 --col 0 --value 3   # move 46
$INVOKE --row 8 --col 1 --value 4   # move 47
$INVOKE --row 8 --col 2 --value 5   # move 48
$INVOKE --row 8 --col 3 --value 2   # move 49
$INVOKE --row 8 --col 5 --value 6   # move 50
$INVOKE --row 8 --col 6 --value 1   # move 51 → SOLVED

# ── 6. Verify ─────────────────────────────────────────────────────────────────

echo "==> Checking result..."
stellar contract invoke --id "$CONTRACT_ID" --network "$NETWORK" --source "$SOURCE" \
  -- get_state

stellar contract invoke --id "$CONTRACT_ID" --network "$NETWORK" --source "$SOURCE" \
  -- is_solved

echo ""
echo "Done! Contract ID: $CONTRACT_ID"
echo "Update MATCH_LOG.md line 3 with:"
echo "  - **Contract:** \`$CONTRACT_ID\`"
