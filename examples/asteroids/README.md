# Asteroids (Soroban Example)

This folder hosts the on-chain Asteroids example contract.

## Project Structure

This example uses the single-contract Soroban layout:

```text
.
├── src
│   ├── lib.rs
│   ├── test.rs
│   └── Makefile
├── Cargo.toml
└── README.md
```

## Setup (from the Soroban "Hello World" guide)

These steps mirror the official Soroban getting-started flow, but scoped to this example.

1) Install Rust + Cargo (via rustup):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2) Ensure your toolchain is up to date and add the WASM target:

```bash
rustup update
rustup target add wasm32-unknown-unknown
```

3) Install the Stellar CLI (Soroban tooling):

```bash
cargo install stellar-cli --locked
```

4) Initialize a Soroban project (already done here):

```bash
mkdir -p examples/asteroids
cd examples/asteroids
stellar contract init .
```

## Common Troubleshooting

- Rust version errors: run `rustup update` and retry.
- Missing WASM target: re-run `rustup target add wasm32-unknown-unknown`.
- `stellar` not found: ensure `~/.cargo/bin` is on your `PATH`, or re-open your shell after installing the CLI.

## Notes

- This example is a single Soroban contract crate.

## Verification (Jan 28, 2026)

Build:

```bash
cd examples/asteroids
cargo build
```

Soroban WASM build:

```bash
stellar contract build
```

Resulting WASM:

```
examples/asteroids/target/wasm32v1-none/release/asteroids.wasm
```

Tests:

```bash
cargo test
```

All tests pass:

```
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
```
