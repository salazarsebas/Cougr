# Trading Card Game

A two-player on-chain trading card game built with the [Cougr](https://github.com/salazarsebas/Cougr) ECS framework on Stellar Soroban. This example demonstrates **atomic multi-action turns** via `BatchBuilder` and **match-scoped sessions** via `SessionBuilder`.

## Why Atomic Turns Matter

In a trading card game a single turn consists of multiple sequential actions: play a creature, cast a spell, declare attacks. If the player's actions were submitted as separate transactions:

- A partially-executed turn could leave the game in an **invalid intermediate state** (e.g., a card removed from hand but never placed on the field).
- The opponent could observe and exploit that intermediate state.
- A wallet approval prompt between every action would create terrible UX.

`BatchBuilder` solves all three problems. The entire turn - every `PlayCreature`, `CastSpell`, and `DeclareAttack` - is composed into a single batch. Because a Soroban contract invocation is atomic by design, **any panic inside `submit_turn` reverts every storage write made during that call**. Either the whole turn succeeds, or nothing changes.

### How it works

```
submit_turn(player, actions)
  │
  ├─ Guard checks (turn ownership, session validity)
  │
  ├─ DrawSystem   ← draw one card at turn start
  ├─ ManaSystem   ← increment max-mana, refill current mana
  │
  ├─ BatchBuilder ← assemble one GameAction per player action
  │                 (proves intent; acts as the turn manifest)
  │
  ├─ For each action in the batch:
  │     PlayCreature  → PlayCardSystem   (validate mana, move card to field)
  │     CastSpell     → CastSpellSystem  (validate mana, deal damage)
  │     DeclareAttack → CombatSystem     (resolve damage, remove dead creatures)
  │                     ↑ any PANIC here reverts the entire call
  │
  ├─ WinConditionSystem ← check health totals
  └─ Advance turn
```

If, say, the second action in the batch runs out of mana, the contract panics, and **all** state changes (including the first action that succeeded) are discarded - the player's hand, field, stats, and mana are exactly as they were before the call.

## Session Keys

Each player calls `start_session` at the beginning of the match. This builds a `SessionScope` (via `SessionBuilder`) that:

- Allows only the three game actions: `play`, `spell`, `attack`.
- Caps the session at 200 operations.
- Expires after a 2-hour TTL (7 200 ledger seconds).

`submit_turn` validates the session before executing any actions. If the session has expired, the entire call is rejected - no stale turns can sneak in after a match is effectively abandoned.

## Game Flow

```
1. MATCH SETUP
   new_match(player_a, player_b, deck_a, deck_b)
     → Both players draw STARTING_HAND_SIZE cards
     → stats: health=20, mana=1/1

   start_session(player_a)   → returns expiry timestamp
   start_session(player_b)   → returns expiry timestamp

2. EACH TURN  (atomic batch)
   submit_turn(active_player, [
     PlayCreature(card_id),
     CastSpell(card_id),
     DeclareAttack(attacker_idx, target_idx),
     ...
   ])
   → DrawSystem:   draw 1 card
   → ManaSystem:   max_mana++, mana refill
   → BatchBuilder: compose action manifest
   → Execute actions atomically
   → WinCondition: check health == 0
   → Advance turn

3. GAME END
   status == StatusAWins | StatusBWins | Conceded
```

## Card Library

| ID | Kind    | Cost | Power | Toughness | Notes                |
|----|---------|------|-------|-----------|----------------------|
| 1  | Creature| 1    | 1     | 2         | Cheap blocker        |
| 2  | Creature| 2    | 2     | 2         | Vanilla              |
| 3  | Creature| 2    | 1     | 3         | Sturdy blocker       |
| 4  | Creature| 3    | 3     | 2         | Aggressive           |
| 5  | Creature| 3    | 2     | 4         | Resilient            |
| 6  | Creature| 4    | 4     | 4         | Mid-range            |
| 7  | Creature| 5    | 5     | 5         | Threat               |
| 8  | Creature| 6    | 6     | 6         | Late-game bomb       |
| 9  | Spell   | 2    | 3     | - | 3 direct damage      |
| 10 | Spell   | 4    | 5     | - | 5 direct damage      |

## ECS Components

| Component     | Fields                                  | Purpose                        |
|---------------|-----------------------------------------|--------------------------------|
| `PlayerHand`  | `cards: Vec<CardId>`                    | Cards currently held           |
| `PlayerField` | `creatures: Vec<CreatureState>`         | Creatures on the battlefield   |
| `PlayerStats` | `health, mana, max_mana`               | Health and resource pool       |
| `MatchState`  | `turn, active_player, phase, status`   | Match phase tracking           |

## ECS Systems

| System                  | Trigger           | Responsibility                              |
|-------------------------|-------------------|---------------------------------------------|
| `DrawSystem`            | Turn start        | Draw one card from deck into hand           |
| `ManaSystem`            | Turn start        | Increment max mana (cap 10), refill mana    |
| `PlayCardSystem`        | `PlayCreature`    | Validate mana, move card to field           |
| `CastSpellSystem`       | `CastSpell`       | Validate mana, deal damage to opponent      |
| `CombatSystem`          | `DeclareAttack`   | Resolve creature-vs-creature or face damage |
| `WinConditionSystem`    | End of turn       | Detect health == 0, set match status        |

## Contract API

```rust
// Setup
fn new_match(env, player_a, player_b, deck_a, deck_b)
fn start_session(env, player) -> u64  // returns session expiry

// Gameplay
fn submit_turn(env, player, actions: Vec<Action>) -> TurnResult
fn concede(env, player)

// Queries
fn get_state(env) -> MatchState
fn get_hand(env, player) -> Vec<Card>
fn get_field(env) -> FieldState
fn get_stats(env, player) -> PlayerStats
```

## Building and Testing

```bash
# Build
cargo build

# Run tests
cargo test

# Build WASM contract
stellar contract build
```

## Dependencies

```toml
[dependencies]
soroban-sdk = "25.1.0"
cougr-core  = { branch = "main", git = "https://github.com/salazarsebas/Cougr.git" }
```

## Resources

- [BatchBuilder source](../../src/accounts/batch.rs)
- [SessionBuilder source](../../src/accounts/session_builder.rs)
- [Soroban Smart Contracts Documentation](https://developers.stellar.org/docs/build/smart-contracts)
