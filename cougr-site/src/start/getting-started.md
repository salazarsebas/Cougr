# Getting Started

> **This page is synced from the main Cougr repository.** To edit it, open a PR against [`salazarsebas/Cougr`](https://github.com/salazarsebas/Cougr).

---

## Prerequisites

Before you can build a Cougr game, you need the following tools installed:

| Tool | Install command |
|---|---|
| **Rust** | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` |
| **Stellar CLI** | `cargo install stellar-cli --features opt` |
| **wasm32 target** | `rustup target add wasm32-unknown-unknown` |

## Clone and run the starter example

```bash
git clone https://github.com/salazarsebas/Cougr
cd Cougr/examples/spawn_and_move
cargo test
```

All tests should pass within two minutes on a machine that has Rust installed.

## Next step

Once the tests pass, head over to [Build Your First Game](build-your-first-game.md) for the full sequential tutorial.
