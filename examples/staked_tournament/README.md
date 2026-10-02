# Staked Tournament

## Purpose and pattern

`staked_tournament` demonstrates per-match token escrow for a four- or
eight-player, single-elimination bracket. Each match stores its own stake
flags and settlement state, and a settled winner is assigned to the next
round automatically. This is a pattern for multi-match staked play, not a
production tournament product.

## Public contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `initialize` | `creator: Address`, `entrants: Vec<Address>`, `token_address: Address`, `stake: i128`, `timeout_seconds: u64` | `()` | Creator-authorized setup for four or eight unique entrants. |
| `stake_match` | `player: Address`, `match_id: u32` | `()` | Transfers the player's stake into the contract before the match deadline. |
| `resolve_match` | `winner: Address`, `match_id: u32` | `()` | Winner-authorized settlement after both players stake; pays the pot and advances the winner. |
| `forfeit_no_show` | `match_id: u32` | `()` | After the deadline, refunds and advances the sole player who staked. |
| `get_match` | `match_id: u32` | `MatchRecord` | Reads the players, stake flags, deadline, and settlement result for one match. |
| `champion` | - | `Option<Address>` | Reads the final winner once the bracket is complete. |

## Architecture overview

The contract stores the bracket as a flat list of match records. Every match
has its own two stake flags, deadline, winner, and precomputed next-round
destination. Normal settlement pays that match's two-stake pot; no-show
settlement returns the present player's own stake. Both paths use the same
winner-advancement function, which fills the appropriate parent match slot
and starts its deadline once both players are assigned.

There is no `GameApp` tick or ECS integration. `resolve_match` trusts an
authenticated winner to report the result, so a real game should call an
escrow settlement path only after verifying its own match outcome.

## Storage model

Tournament configuration, the fixed bracket slots, and champion are stored
together in instance storage. The example keeps all bracket data in one
instance value for clarity; a longer-running tournament would need an
explicit storage and TTL strategy.

## Main gameplay flow

1. The creator initializes the contract with four or eight unique players, a
   Soroban token address, a positive stake, and a timeout.
2. Each first-round player calls `stake_match`; their stake moves to the
   contract and is tracked only in that match's record.
3. After both stakes arrive, the authenticated winner calls `resolve_match`.
   The winner receives both stakes and advances automatically.
4. If exactly one player stakes before the deadline, anyone can call
   `forfeit_no_show` after it expires. The present player's stake is returned
   and they advance, so the next round can proceed.
5. The two advancing players stake in their new match. The final settlement
   records the champion.

## Cougr APIs used

None. This example isolates the Soroban token interface and bracket escrow
pattern; it does not use Cougr ECS, scheduler, auth, or standards APIs.

## Build and test commands

```bash
cd examples/staked_tournament
cargo test
stellar contract build
```

## Known limitations

- Only four- and eight-player single-elimination brackets are supported.
- No seeding, ranking, matchmaking, house fee, or dispute resolution is
  included.
- The winner self-reports normal match results; the contract does not verify
  gameplay outcomes.
- A no-show refund requires exactly one player to have staked. A match where
  neither player stakes, or where both stake and then disappear, needs an
  external decision process and cannot advance automatically.
- This example is not a production tournament product and makes no claim
  about live assets or mainnet deployment.