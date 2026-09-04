# Reversi Match Log - Testnet

- **Contract:** `CAQCSRHZZ4O3LN3LEQVF3JQ3H7UI7XPR65WWSCWKUWDKITSEFVRHF2OW`
- **Network:** Testnet
- **Black (1):** `reversi_black` → `GB3TUPV2HHBZFXHUBLRTJFVOLHFWEVPB7PW33YMY6OVLEY7AT7YDO56D`
- **White (2):** `reversi_white` → `GC33TZ4XTBFNRG3O4NQYVW5PIMH63KVCWPV5QLXFTKM76TTGQUYMYA`

---

## Result

| | Black | White |
|---|---|---|
| **Final score** | **36** | **28** |
| **Winner** | **BLACK** | |

---

## Deployment

```bash
# Generate identities and fund via Friendbot
stellar keys generate reversi_black
stellar keys generate reversi_white
stellar keys fund reversi_black --network testnet
stellar keys fund reversi_white --network testnet

# Build WASM
cd examples/reversi && stellar contract build

# Deploy
CONTRACT_ID=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/reversi.wasm \
  --network testnet \
  --source reversi_black)
# → CAQCSRHZZ4O3LN3LEQVF3JQ3H7UI7XPR65WWSCWKUWDKITSEFVRHF2OW

# Initialise
stellar contract invoke --id $CONTRACT_ID --network testnet --source reversi_black \
  -- init_game \
  --player_one reversi_black \
  --player_two reversi_white
```

---

## Move Log

Opening board: Black=(3,4),(4,3) / White=(3,3),(4,4)

| Move | Player | Row | Col | Score after (B/W) |
|------|--------|-----|-----|-------------------|
| 1  | Black | 3 | 2 | 4 / 1 |
| 2  | White | 2 | 4 | 3 / 3 |
| 3  | Black | 4 | 5 | 5 / 2 |
| 4  | White | 5 | 4 | 4 / 4 |
| 5  | Black | 5 | 5 | 6 / 3 |
| 6  | White | 4 | 2 | 5 / 5 |
| 7  | Black | 3 | 5 | 8 / 3 |
| 8  | White | 3 | 6 | 7 / 5 |
| 9  | Black | 4 | 6 | 9 / 4 |
| 10 | White | 5 | 6 | 7 / 7 |
| 11 | Black | 6 | 5 | 10 / 5 |
| 12 | White | 6 | 4 | 6 / 10 |
| 13 | Black | 6 | 6 | 9 / 8 |
| 14 | White | 7 | 6 | 7 / 11 |
| 15 | Black | 7 | 4 | 10 / 9 |
| 16 | White | 2 | 5 | 7 / 13 |
| 17 | Black | 1 | 4 | 10 / 11 |
| 18 | White | 1 | 5 | 8 / 14 |
| 19 | Black | 1 | 6 | 11 / 12 |
| 20 | White | 5 | 3 | 8 / 16 |
| 21 | Black | 7 | 5 | 13 / 12 |
| 22 | White | 7 | 3 | 9 / 17 |
| 23 | Black | 4 | 1 | 13 / 14 |
| 24 | White | 0 | 5 | 9 / 19 |
| 25 | Black | 6 | 7 | 13 / 16 |
| 26 | White | 4 | 0 | 8 / 22 |
| 27 | Black | 5 | 2 | 14 / 17 |
| 28 | White | 4 | 7 | 12 / 20 |
| 29 | Black | 2 | 6 | 18 / 15 |
| 30 | White | 6 | 3 | 14 / 20 |
| 31 | Black | 6 | 2 | 20 / 15 |
| 32 | White | 3 | 7 | 13 / 23 |
| 33 | Black | 2 | 3 | 18 / 19 |
| 34 | White | 3 | 1 | 13 / 25 |
| 35 | Black | 2 | 7 | 18 / 21 |
| 36 | White | 6 | 1 | 14 / 26 |
| 37 | Black | 5 | 1 | 21 / 20 |
| 38 | White | 1 | 3 | 17 / 25 |
| 39 | Black | 3 | 0 | 24 / 19 |
| 40 | White | 1 | 7 | 18 / 26 |
| 41 | Black | 6 | 0 | 23 / 22 |
| 42 | White | 5 | 7 | 19 / 27 |
| 43 | Black | 0 | 6 | 24 / 23 |
| 44 | White | 0 | 7 *(corner)* | 20 / 28 |
| 45 | Black | 1 | 2 | 25 / 24 |
| 46 | White | 5 | 0 | 19 / 31 |
| 47 | Black | 7 | 7 *(corner)* | 23 / 28 |
| 48 | White | 2 | 0 | 20 / 32 |
| 49 | Black | 2 | 1 | 27 / 26 |
| 50 | White | 1 | 0 | 22 / 32 |
| 51 | Black | 0 | 0 *(corner)* | 28 / 27 |
| 52 | White | 0 | 4 | 26 / 30 |
| 53 | Black | 2 | 2 | 32 / 25 |
| 54 | White | 0 | 1 | 28 / 30 |
| 55 | Black | 7 | 2 | 33 / 26 |
| 56 | White | 0 | 2 | 31 / 29 |
| 57 | Black | 0 | 3 | 39 / 22 |
| 58 | White | 1 | 1 | 36 / 26 |
| - | *Black passed* (no legal moves) | - | - | - |
| 59 | White | 7 | 0 | 34 / 29 |
| 60 | Black | 7 | 1 | **36 / 28** |

---

## Final Board

```
     0   1   2   3   4   5   6   7
  0  B   B   B   B   W   W   W   W
  1  B   W   W   W   W   W   W   W
  2  B   B   B   B   W   B   W   W
  3  B   B   B   B   W   W   B   W
  4  B   B   B   W   B   W   W   W
  5  B   B   W   W   W   B   W   W
  6  B   B   B   B   B   W   B   B
  7  W   B   B   B   B   B   B   B
```

---

## Key Contract Calls

```bash
# Submit a move
stellar contract invoke --id $CONTRACT_ID --network testnet --source <player> \
  -- submit_move \
  --player <player> \
  --row <row> --col <col>

# Read board state
stellar contract invoke --id $CONTRACT_ID --network testnet --source <player> \
  -- get_board

# Read score / winner
stellar contract invoke --id $CONTRACT_ID --network testnet --source <player> \
  -- get_score

# Read turn / game status
stellar contract invoke --id $CONTRACT_ID --network testnet --source <player> \
  -- get_state
```

---

## Notes

- Black won 3 corners (0,0), (7,7), and via adjacency; White captured (0,7).
- The decisive swing was Black's move 57 at (0,3) which flipped **5 pieces** in three directions simultaneously, extending Black's lead from 31–29 to 39–22.
- Black was passed once (after move 58) when no legal move existed; both sides exhausted all moves by move 60, triggering `pass_count=2` and `status=1`.
