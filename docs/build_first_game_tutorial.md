# Build Your First On-Chain Game with Cougr

Step-by-step end-to-end tutorial for building, testing, and deploying a simple turn-based strategy game using `cougr-core` and Soroban smart contracts.

---

## Prerequisites

- **Rust:** `1.80.0` or higher with `wasm32-unknown-unknown` target.
- **Soroban CLI:** `stellar-cli` v22.0.0+
- **Cougr CLI:** `cargo install --path crates/cougr-cli`

---

## Step 1: Initialize Project

```bash
cougr new my-first-game --template turn-based
cd my-first-game
```

---

## Step 2: Define Game State & Invariants

```rust
use cougr_core::{GameHarness, Scenario, WorldFixture};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

#[contract]
pub struct GameContract;

#[contractimpl]
impl GameContract {
    pub fn init_match(env: Env, player_one: Address, player_two: Address) -> Symbol {
        let match_id = symbol_short!("MATCH1");
        // State initialization logic
        match_id
    }
}
```

---

## Step 3: Run Determinism Tests

```bash
cargo test --test determinism_suite
```

---

## Step 4: Deploy to Testnet

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/my_first_game.wasm \
  --source admin \
  --network testnet
```

---

## References

- Game Harness docs: [`docs/testing_guide.md`](testing_guide.md)
- Boundary guide: [`docs/boundary_guide.md`](boundary_guide.md)
