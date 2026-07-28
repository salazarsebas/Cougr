# cougr-core-circuits (internal)

Off-chain Circom toolchain for `cougr_core::circuits`. Not published to crates.io.

## Package manager

**[Bun](https://bun.sh)** installs and runs the JS toolchain (`snarkjs`, `circomlibjs`).
**Node.js** is still required for Circom's `generate_witness.js` (native `circom_runtime`).

## Prerequisites

- [Bun](https://bun.sh) 1.3+
- [Node.js](https://nodejs.org) 18+ (witness generation only)
- [Circom 2.1+](https://github.com/iden3/circom/releases)

## Quick start

```bash
cd internal/cougr-core-circuits
bun install
export CIRCOM=/path/to/circom   # if not on PATH
bun run pipeline
```

## Scripts

| Script | Purpose |
|--------|---------|
| `bun run fixtures` | Regenerate `fixtures/*.input.json` |
| `bun run pipeline` | Full compile → setup → prove → verify |
| `scripts/compile.sh` | Circom → `artifacts/` |
| `scripts/download-ptau.sh` | Fetch or generate pot14 |
| `scripts/setup.sh` | Groth16 trusted setup per circuit |
| `scripts/prove.sh [name\|all]` | Witness, prove, verify |
| `scripts/export-vk.sh` | Copy VK JSON to `exported/` |
| `scripts/validate-layouts.sh` | Match Circom ↔ Rust layouts |

## Cryptography (per circuit)

| Circuit | Constraints | Logic |
|---------|-------------|--------|
| `hidden_cards` | ~13.6k | Poseidon hand commitment, salted deck chain, membership + uniqueness |
| `fog_of_war` | ~460 | Euclidean visibility + Poseidon explored-root transition |
| `fair_dice` | ~353 | Poseidon seed commit, `roll = (seed mod sides) + 1` |
| `sealed_bid` | ~325 | Poseidon bid opening `H(bid, salt, auction_id)` |

Trusted setup uses **pot14** (`PTAU_POWER=14`) to cover `hidden_cards`.

See [AUDIT.md](./AUDIT.md) for trusted-setup policy.

## Deploying to mainnet

Everything above is the **test** pipeline — fine for CI, examples, and integration tests, not for mainnet. If
you're preparing a real deployment:

1. [CEREMONY.md](./CEREMONY.md) — how to run a real, multi-party phase-2 ceremony instead of `setup.sh`'s
   single-contributor test key.
2. [PRODUCTION_KEYS.md](./PRODUCTION_KEYS.md) — how to get the ceremony's output into
   `GameCircuitSpec::with_verification_key` correctly.
