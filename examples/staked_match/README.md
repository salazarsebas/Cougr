# Staked Match

A two-player match escrow. Both players lock an equal stake in a Soroban token,
an agreed result pays the winner the whole pot, an agreed draw returns each
stake, and a match nobody joins is recoverable by cancel or by timeout.

This is a pattern, not a production betting product. It has no house fee, no
custodial treasury, no dispute process, and no oracle. A real product needs a
result source that does not depend on both losers cooperating, and a legal
review this example does not attempt.

## State machine

```
                  create_match
                       |
                       v
                    [Open] --------- cancel --------> [Refunded] -> withdraw (host)
                       |                                   ^
                       | join                              |
                       v                    claim_timeout  |
                  [Active] ---------------------------------
                    /    \
       agree_result/      \agree_draw
                  v        v
           [Settled]    [Drawn]
                |            |
        withdraw (winner)   withdraw (each player)
```

`Open` holds one stake, `Active` holds two. `Settled`, `Drawn` and `Refunded`
are terminal: the pot only leaves through `withdraw`, which pays each address at
most once.

## Auth model

- `create_match`, `join` and `cancel` are authorized by the single player they
  affect.
- `agree_result` and `agree_draw` require **both** players. Neither side can
  declare an outcome the other has not signed, which is what removes the need
  for an admin.
- `withdraw` is authorized by the caller and pays only what that caller is owed.
- `claim_timeout` is deliberately permissionless. It moves the escrow to
  `Refunded` and the funds can only ever reach the host, so a third party
  unsticking an abandoned match takes nothing.

## What each test defends

| Test | Threat |
| --- | --- |
| `winner_withdraws_the_whole_pot` | Winner paid less or more than both stakes |
| `draw_returns_each_stake_exactly_once` | A draw pays one player twice or strands the pot |
| `cancel_refunds_host_and_is_rejected_once_joined` | Host cancels after an opponent has committed funds |
| `timeout_refunds_only_after_the_deadline` | Stake locked forever, or timeout claimed early to escape a match |
| `double_settle_and_double_withdraw_are_rejected` | Agreed result overwritten, or the pot withdrawn twice |
| `stranger_and_loser_cannot_withdraw` | A non-participant drains the pot, or the loser withdraws anyway |
| `host_cannot_join_their_own_match` | Host fakes an opponent to stake against themselves |
| `join_is_rejected_after_the_deadline` | Opponent joins once the host is already owed a refund |
| `invalid_stake_and_deadline_are_rejected` | Match opened in a state that cannot settle |
| `winner_must_be_a_participant` | Payout directed to an address that never staked |
| `escrow_state_tracks_the_pot` | Reported state disagrees with the escrowed balance |

## Running

```bash
cargo test
stellar contract build
```

Tests use the token contract `soroban-sdk` provides in `testutils`. Nothing here
deploys, and no asset or secret is committed.
