# zk\_dice\_session

**A composition example: `fair_dice` ZK verification authorized through a session key.**

This example wires two existing Cougr subsystems together for the first time.
It does not introduce a new circuit or a new account model – only the integration
between them.

| Subsystem | Source example this builds on |
|---|---|
| ZK proof verification (`fair_dice` circuit) | [`examples/dice_duel`](../dice_duel) |
| Session key lifecycle (`SessionBuilder` / `SessionManager`) | [`examples/session_arena`](../session_arena) |

---

## Why this matters

Every ZK-circuit example in the repo re-signs with the owner key on every proof
submission. A player who wants ten rapid dice rolls pays ten wallet prompts.

`session_arena` (and `tap_battle`) show how to approve a scoped key once and
act many times without re-signing, but neither example touches a ZK proof.

`zk_dice_session` closes that gap: a player approves once, then rolls as many
times as the session budget allows – each roll verified by an on-chain Groth16
proof, each authorized by the session key, zero additional wallet prompts.

---

## Contract API

### `init_game(sides, seed_commitment) → GameConfig`

Initialises the game with a die face count (≥ 2) and a Pedersen seed
commitment. Internally calls `fair_dice()` to validate the parameters; the
circuit spec is reconstructed from storage on every `roll` call.

### `open_session(owner, max_rolls, expires_in) → ActiveSession`

Creates a session scoped to the `"roll"` action. Requires one owner wallet
signature. Returns an `ActiveSession` containing the `key_id` needed for
subsequent `roll` calls.

### `roll(owner, key_id, roll_result, nonce, proof) → bool`

Submits a Groth16 proof of a fair dice roll. Authorization happens in two gates:

1. **Session gate** – `SessionManager::execute_action` checks expiry, budget,
   and action scope. No wallet prompt.
2. **ZK gate** – `spec.verify_dice_roll` is called identically to the
   corresponding call in `dice_duel`. The proof format is unchanged.

Returns `true` if both gates pass, `false` if the proof is invalid.
Panics (rolls back the transaction) if the session is missing, expired, or
budget-exhausted.

### `last_roll(player) → RollRecord`

Returns the last accepted roll for `player`.

---

## Quick start

```bash
# From the repo root
cd examples/zk_dice_session
cargo test
stellar contract build
```

---

## Test coverage

| Test | What it proves |
|---|---|
| `roll_succeeds_inside_active_session` | Roll accepted when session is active (DoD) |
| `roll_fails_with_no_session` | Roll panics when no session key exists (DoD) |
| `roll_fails_after_session_expiry` | Roll panics once the session window closes (DoD) |
| `roll_returns_false_for_invalid_proof` | Out-of-range roll_result → `false`, proof gate intact |
| `roll_fails_when_session_budget_exhausted` | Budget exhaustion panics before proof verification |
| `independent_sessions_for_multiple_players` | Each player's session is isolated |
| `init_game_rejects_single_sided_die` | Circuit parameter contract enforced |
| `circuit_spec_matches_fair_dice` | Circuit ID matches `FairDice` (same path as dice_duel) |

---

## Cougr features demonstrated

- `zk-circuits` – `fair_dice` verifier, `Groth16Proof`, `CircuitId::FairDice`
- `session-auth` – `SessionBuilder`, `SessionManager`, `SessionStorage`, `ActiveSession`
- `soroban-game` – `#[contract]` / `#[contractimpl]` / `#[contracttype]`
