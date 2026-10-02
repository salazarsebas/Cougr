# Season Ladder

## Purpose and pattern

`season_ladder` runs a competitive season of head-to-head results: players
register themselves, the season admin records each fixture, and an integer
rating is updated after every result. It demonstrates the **standards layer
of `cougr-core`** — `Ownable` for admin authority and `Pausable` for an
emergency stop — instead of a private admin flag, and it shows integer-only
rating math that stays deterministic inside a Soroban contract.

## Public contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `admin: Address` | `Address` | One-time setup; binds the season admin through `Ownable`. |
| `register_player` | `player: Address` | `PlayerRecord` | Self-registration at the starting rating. Frozen while paused. |
| `report_result` | `reporter, home, away: Address`, `match_no: u32`, `home_goals, away_goals: u32` | `MatchResult` | Owner-only; records fixture `match_no` for the ordered pair and updates both ratings. Frozen while paused. |
| `pause` | `admin: Address` | – | Owner-only emergency stop. |
| `unpause` | `admin: Address` | – | Owner-only resume. |
| `is_paused` | – | `bool` | Current pause state. |
| `owner` | – | `Option<Address>` | Current season admin. |
| `player` | `id: Address` | `PlayerRecord` | Full record (rating, matches, W/D/L). |
| `rating` | `id: Address` | `i64` | Convenience view of a player's rating. |
| `matches_played` | `home, away: Address` | `u32` | Fixtures already recorded for the ordered pair. |
| `fixture` | `home, away: Address`, `match_no: u32` | `MatchResult` | A stored result. |
| `standings` | – | `Vec<(Address, i64)>` | Ladder ordered by rating, highest first. |

## Architecture overview

The contract is intentionally monolithic (it has three small systems, well
under the threshold where the standard asks for a split):

- **Standards layer** — `Ownable` and `Pausable` from `cougr_core::standards`
  own the authorization of `report_result` / `pause` / `unpause` and the
  freeze of `register_player` / `report_result`.
- **Ratings** — pure integer functions (`expected_score_ppm`,
  `rating_delta`, `score_ppm`) convert a result into a mirrored rating swing.
- **Storage** — player records, per-pair fixture counters, stored fixtures,
  and the roster all live in instance storage.

There is no tick loop: every entrypoint is a direct call driven by players
(self-registration) and the admin (results, pause).

## Storage model

| Key | Storage | Why |
|---|---|---|
| `Player(Address)` | instance | Small example; ratings and W/D/L are read on every result. |
| `PairMatches(Address, Address)` | instance | Next fixture number per ordered pair; enforces the duplicate guard. |
| `Fixture(Address, Address, u32)` | instance | Each reported result, round-trippable through the `fixture` view. |
| `Roster` | instance | Registration order, feeds `standings`. |
| `Ownable` / `Pausable` state | persistent (standards layer) | Written by `cougr_core::standards` under its own namespaced keys. |

## Rating rule (integer-only)

- Start rating: `1000`. K-factor: `32`.
- Expected score for a player against an opponent, in parts-per-million:
  `expected_ppm = 1_000_000 * rating / (rating + opponent)` (integer
  division, truncated).
- Result score: win `1_000_000`, draw `500_000`, loss `0`.
- Swing: `delta = (32 * (score_ppm - expected_ppm)) / 1_000_000`,
  truncated toward zero. The winner's swing is applied to the home side and
  mirrored (`-delta`) to the away side, so the total rating is conserved
  unless a floor clamp triggers.
- Ratings are clamped to a floor of `100` so expected-score math always has
  a positive denominator.

Consequences that the tests pin down: an even first match pays `±16`; a
favourite's win shrinks as the gap grows (15, 14, …); an upset win by the
lower-rated side pays **more** than the even-rating 16; draws between even
players change nothing.

## What pause freezes

While paused (`Pausable`):

- `register_player` is rejected,
- `report_result` is rejected,
- views (`player`, `standings`, `fixture`, …) keep working read-only,
- only the `Ownable` owner can `pause`/`unpause`.

After `unpause`, reporting continues with the existing fixture counters —
the pause never renumbers or discards fixtures.

## Main gameplay flow

1. Deploy, then `initialize(admin)` — binds the owner.
2. Players call `register_player` themselves (starting rating 1000).
3. The admin records fixtures in order: `report_result(admin, home, away, 1, 2, 1)`
   — ratings update immediately.
4. Repeat with `match_no` 2, 3, … for the same pair; duplicates or skipped
   numbers are rejected.
5. If something is wrong, the admin calls `pause`; the season freezes.
   `unpause` resumes exactly where it left off.
6. `standings` shows the ladder at any time.

## Cougr APIs used

- `cougr_core::standards::Ownable` — `initialize`, `require_owner`, `owner`.
- `cougr_core::standards::Pausable` — `require_not_paused`, `pause`,
  `unpause`, `is_paused`.
- `cougr_core::standards::StandardsError` — surfaced through the standards
  layer's `Result`s.

No ECS, `GameApp`, ZK, or session APIs are used: the issue this example
answers scopes it to results, ratings, and the standards layer.

## Build and test commands

```bash
cd examples/season_ladder
cargo test
stellar contract build
```

## Known limitations

- Fixtures are keyed by the **ordered** pair `(home, away)`; a rematch with
  the sides swapped starts a new fixture counter.
- `match_no` must be reported in order (1, 2, 3, …) per pair.
- Everything lives in instance storage; a real season would move player
  records to persistent storage with TTLs.
- The rating floor (`100`) can shrink a mirrored delta, so total-rating
  conservation holds only when no clamp triggers.
- No tokens, wagers, or matchmaking: ratings are the only stake.
