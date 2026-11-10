# Cougr Patterns

Use this reference when deciding how to map a game into `cougr-core` concepts.

## Modeling Heuristics

Ask four questions:

1. What are the active things in the game?
2. What data belongs to each thing?
3. What transitions happen repeatedly?
4. What rules are global rather than entity-specific?

Map the answers like this:

| Design concept | Usually becomes |
|---|---|
| Player, unit, projectile, tile, card | Entity |
| Health, position, owner, cooldown, revealed state | Component |
| Tick update, attack resolution, scoring, turn advance | System |
| Match config, phase, board size, ruleset | Resource or singleton state |

## Pattern: Thin Contract, Thick Logic

Prefer:

- contract entrypoint loads state
- helper or system validates action
- helper or system mutates state
- contract persists state
- contract returns result

Avoid burying all rule logic inline inside contract methods. That makes later changes expensive and tests harder to reason about.

## Pattern: Public API Around Player Intent

Design methods around what players do, not around low-level storage operations.

Prefer:

- `submit_move`
- `attack`
- `play_card`
- `end_turn`
- `reveal_commitment`

Avoid:

- `set_piece_position`
- `update_board_cell`
- `write_match_state`

The first group preserves invariants. The second group leaks internals and encourages brittle code.

## Pattern: Phase-Driven Games

For turn-based or staged games, model the phase explicitly.

Examples:

- `Setup`
- `Commit`
- `Reveal`
- `MainTurn`
- `Resolution`
- `Finished`

This makes illegal transitions easy to reject and keeps tests readable.

## Pattern: One Source Of Truth

Do not maintain duplicated versions of the same game fact unless there is a strong reason.

Bad:

- board cells say a unit exists
- unit list separately says it exists
- score history also tries to infer the same state

Better:

- one canonical state
- derived views computed when needed

## Pattern: Minimal Vertical Slice

For a new prototype, implement only the smallest loop that proves the idea:

| Game type | Good first slice |
|---|---|
| Arcade | Spawn, move, collide, lose |
| Board game | Initialize board, submit legal move, detect winner |
| Card game | Draw, play one card type, resolve one effect |
| Hidden information game | Commit, reveal, resolve outcome |

Add progression, polish, and secondary systems after the loop is stable.

## Pattern: Test As Match Scripts

Write tests as short game stories:

- initialize the game
- perform one or more legal actions
- assert the resulting state
- assert illegal actions fail where relevant

This style catches behavior regressions and doubles as executable documentation.

## When To Use Advanced Cougr Features

Use them because the design needs them, not because they exist.

| Feature | Good use case |
|---|---|
| ZK tooling | hidden legality checks, private state proofs, selective verification |
| Commit-reveal | hidden choices, fog-of-war, simultaneous decisions |
| Session keys | repeated player actions with delegated authorization |
| Recovery and multi-device | account continuity as part of the product experience |

If the game does not need these, leave them out of the first prototype.

## Anti-Patterns

- building a framework before building the game
- exposing low-level setters as contract API
- mixing setup, gameplay, and admin logic into one large function
- implementing three genres of mechanics in the first pass
- over-documenting speculative features that are not yet implemented

