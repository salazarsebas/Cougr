# Turn-Based RPG Arena

Cougr smart contract example implementing a compact, deterministic turn-based combat loop with status effects and cooldowns.
Built to demonstrate a deterministic two-combatant battle using clean Cougr-style structure.

## Validation Commands

To build and run tests natively or against standard architectures, be aware that Soroban projects target `wasm32v1-none`. The following commands utilize `wasm32v1-none`.

```bash
cd examples/rpg_arena
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
stellar contract build
```
