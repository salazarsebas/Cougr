# Sudoku Match Log - Simulated Testnet Walkthrough

- **Contract:** `CCF5MENLO56M4S72BF4H5KMU7XYO45VTB7GJ2O3BZFG7EWY6X3X2DBQO`
- **Network:** Testnet
- **Player:** `sudoku_player`

---

## Puzzle Fixture

```
     0   1   2   3   4   5   6   7   8
  0  5   3   .   .   7   .   .   .   .
  1  6   .   .   1   9   5   .   .   .
  2  .   9   8   .   .   .   .   6   .
  3  8   .   .   .   6   .   .   .   3
  4  4   .   .   8   .   3   .   .   1
  5  7   .   .   .   2   .   .   .   6
  6  .   6   .   .   .   .   2   8   .
  7  .   .   .   4   1   9   .   .   5
  8  .   .   .   .   8   .   .   7   9
```

30 fixed cells · 51 editable cells

---

## Result

| | |
|---|---|
| **Status** | **SOLVED** |
| **Moves** | **51** |

---

## Deployment

```bash
stellar keys generate sudoku_player
stellar keys fund sudoku_player --network testnet

CONTRACT_ID=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/sudoku.wasm \
  --network testnet \
  --source sudoku_player)

stellar contract invoke --id $CONTRACT_ID --network testnet --source sudoku_player \
  -- init_game \
  --puzzle '[5,3,0,0,7,0,0,0,0,6,0,0,1,9,5,0,0,0,0,9,8,0,0,0,0,6,0,8,0,0,0,6,0,0,0,3,4,0,0,8,0,3,0,0,1,7,0,0,0,2,0,0,0,6,0,6,0,0,0,0,2,8,0,0,0,0,4,1,9,0,0,5,0,0,0,0,8,0,0,7,9]'
```

---

## Move Log

| Move | Row | Col | Value | Cumulative moves |
|------|-----|-----|-------|-----------------|
| 1  | 0 | 2 | 4 | 1 |
| 2  | 0 | 3 | 6 | 2 |
| 3  | 0 | 5 | 8 | 3 |
| 4  | 0 | 6 | 9 | 4 |
| 5  | 0 | 7 | 1 | 5 |
| 6  | 0 | 8 | 2 | 6 |
| 7  | 1 | 1 | 7 | 7 |
| 8  | 1 | 2 | 2 | 8 |
| 9  | 1 | 6 | 3 | 9 |
| 10 | 1 | 7 | 4 | 10 |
| 11 | 1 | 8 | 8 | 11 |
| 12 | 2 | 0 | 1 | 12 |
| 13 | 2 | 3 | 3 | 13 |
| 14 | 2 | 4 | 4 | 14 |
| 15 | 2 | 5 | 2 | 15 |
| 16 | 2 | 6 | 5 | 16 |
| 17 | 2 | 8 | 7 | 17 |
| 18 | 3 | 1 | 5 | 18 |
| 19 | 3 | 2 | 9 | 19 |
| 20 | 3 | 3 | 7 | 20 |
| 21 | 3 | 5 | 1 | 21 |
| 22 | 3 | 6 | 4 | 22 |
| 23 | 3 | 7 | 2 | 23 |
| 24 | 4 | 1 | 2 | 24 |
| 25 | 4 | 2 | 6 | 25 |
| 26 | 4 | 4 | 5 | 26 |
| 27 | 4 | 6 | 7 | 27 |
| 28 | 4 | 7 | 9 | 28 |
| 29 | 5 | 1 | 1 | 29 |
| 30 | 5 | 2 | 3 | 30 |
| 31 | 5 | 3 | 9 | 31 |
| 32 | 5 | 5 | 4 | 32 |
| 33 | 5 | 6 | 8 | 33 |
| 34 | 5 | 7 | 5 | 34 |
| 35 | 6 | 0 | 9 | 35 |
| 36 | 6 | 2 | 1 | 36 |
| 37 | 6 | 3 | 5 | 37 |
| 38 | 6 | 4 | 3 | 38 |
| 39 | 6 | 5 | 7 | 39 |
| 40 | 6 | 8 | 4 | 40 |
| 41 | 7 | 0 | 2 | 41 |
| 42 | 7 | 1 | 8 | 42 |
| 43 | 7 | 2 | 7 | 43 |
| 44 | 7 | 6 | 6 | 44 |
| 45 | 7 | 7 | 3 | 45 |
| 46 | 8 | 0 | 3 | 46 |
| 47 | 8 | 1 | 4 | 47 |
| 48 | 8 | 2 | 5 | 48 |
| 49 | 8 | 3 | 2 | 49 |
| 50 | 8 | 5 | 6 | 50 |
| 51 | 8 | 6 | 1 | **51 → SOLVED** |

---

## Final Board

```
     0   1   2   3   4   5   6   7   8
  0  5   3   4   6   7   8   9   1   2
  1  6   7   2   1   9   5   3   4   8
  2  1   9   8   3   4   2   5   6   7
  3  8   5   9   7   6   1   4   2   3
  4  4   2   6   8   5   3   7   9   1
  5  7   1   3   9   2   4   8   5   6
  6  9   6   1   5   3   7   2   8   4
  7  2   8   7   4   1   9   6   3   5
  8  3   4   5   2   8   6   1   7   9
```

---

## Key Contract Calls

```bash
# Submit a value
stellar contract invoke --id $CONTRACT_ID --network testnet --source sudoku_player \
  -- submit_value --row <row> --col <col> --value <value>

# Read a single cell
stellar contract invoke --id $CONTRACT_ID --network testnet --source sudoku_player \
  -- get_cell --row <row> --col <col>

# Read overall state
stellar contract invoke --id $CONTRACT_ID --network testnet --source sudoku_player \
  -- get_state

# Check completion
stellar contract invoke --id $CONTRACT_ID --network testnet --source sudoku_player \
  -- is_solved
```

---

## Notes

- All 51 empty cells were filled in row-major order (top-left to bottom-right).
- `submit_value` panics on any constraint violation - the transaction fails and the board is not updated.
- Move 51 at (8,6)=1 triggered `completion_system`, setting `status=1` (solved).
