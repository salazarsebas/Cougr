# Session-Scoped ZK Dice Example

This example composes two existing subsystems:
1. The `fair_dice` circuit and verification path (`internal/cougr-core-circuits` / `examples/dice_duel`).
2. The `SessionBuilder` and `SessionManager` account session authorization lifecycle (`examples/session_arena`).

Instead of requiring a fresh owner signature for every single dice roll, players can authorize a limited session key to submit verified rolls until session expiry.

## Running Tests

```bash
cargo test -p session_dice
```
