# CLAUDE.md

Instructions for AI assistants (Claude Code and others) working in this repository. The goal is that a contribution written or assisted by an AI is indistinguishable in quality from one written by a maintainer who knows this codebase well.

## Before writing code

Read the doc that governs the area you're touching, don't guess at Cougr's conventions from general Rust or general Soroban knowledge:

| Touching... | Read first |
|---|---|
| Any architectural choice (world backend, storage placement, entrypoint shape) | [docs/PATTERNS.md](docs/PATTERNS.md) |
| Storage or query performance | [docs/PERFORMANCE.md](docs/PERFORMANCE.md) |
| `unsafe`, or anything that feels like it might need it | [docs/UNSAFE_INVARIANTS.md](docs/UNSAFE_INVARIANTS.md), assume the answer is no |
| A new or modified `examples/*` project | [examples/EXAMPLE_STANDARD.md](examples/EXAMPLE_STANDARD.md) |
| A public API addition or change | the Public API Checklist in [CONTRIBUTING.md](CONTRIBUTING.md) |
| Accounts, session keys, or passkeys | [docs/ACCOUNT_KERNEL.md](docs/ACCOUNT_KERNEL.md) |
| Hidden state, commit-reveal, or ZK circuits | [docs/PRIVACY_MODEL.md](docs/PRIVACY_MODEL.md) |
| What belongs on-chain at all | [docs/ONCHAIN_OFFCHAIN_BOUNDARY.md](docs/ONCHAIN_OFFCHAIN_BOUNDARY.md) |
| Docs or example prose | [docs/VOICE_GUIDE.md](docs/VOICE_GUIDE.md) |

## Soroban and contract code

- `#![no_std]` is load-bearing, not a formality. Don't introduce a dependency or pattern that requires `std`.
- Contract entrypoints stay thin: validate input, load state, call into a system or helper, persist state, return. Push actual gameplay logic into `systems.rs` / plugin systems, not into `#[contractimpl]` methods. See [docs/PATTERNS.md § Default Entry Point](docs/PATTERNS.md#default-entry-point).
- Don't panic on reachable, caller-triggerable conditions. Validate and return a clear error instead; reserve panics for genuine invariant violations.
- Every state-changing entrypoint that should be access-controlled must call `require_auth` (or the relevant `AccessControl`/`Ownable` check) before mutating state, not after.
- Prefer the standards layer (`Ownable`, `AccessControl`, `Pausable`, `ExecutionGuard`, `BatchExecutor`, ...) over hand-rolling the same guard again. Check [docs/STANDARDS_LAYER.md](docs/STANDARDS_LAYER.md) before writing a new one.
- Choose `SimpleWorld` vs `ArchetypeWorld`, and table vs sparse storage, based on the actual query/mutation shape, not by default. See [docs/PERFORMANCE.md](docs/PERFORMANCE.md).
- Use `impl_component_observed!` for state a client needs to react to without polling; plain `impl_component!` otherwise. Don't emit events nobody consumes.
- New examples depend on the published `cougr-core` from crates.io, not a path dependency, unless they exercise an API that genuinely isn't published yet, in which case follow the exception format in [examples/EXAMPLE_STANDARD.md § 1.1](examples/EXAMPLE_STANDARD.md).

## Clean code

- Keep changes focused; don't mix an unrelated refactor into a feature or fix.
- Name things by domain responsibility (`turn_state`, `reveal_deadline`), not by storage mechanics (`data_1`, `flag_a`).
- No dead code, no commented-out blocks, no speculative abstraction for a second use case that doesn't exist yet.
- Match the surrounding module's existing patterns before introducing a new one; a second, competing way to do the same thing is a cost even when the new way is individually reasonable.
- Comments explain a non-obvious *why* (a constraint, an invariant, a Soroban-specific gotcha), never restate what the code already says.

## Before calling something done

Run, and fix everything these report:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

If the change touches a Soroban contract (`src/`, `examples/*`, `cli/templates/*`), also run `stellar contract build` for every affected crate. A crate that passes `cargo build` but fails `stellar contract build` is not a valid contract, and CI will reject it.

## Writing style

- Never use em dashes (`—`) in code, comments, commit messages, documentation, or any other generated content. Use a comma, period, colon, or a regular hyphen (`-`) instead.

## Other conventions

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full development standards, local validation commands, and pull request expectations.
