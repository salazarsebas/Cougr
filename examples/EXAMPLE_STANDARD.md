# Example Quality Standard

This document defines the minimum quality bar that every project under `examples/` must satisfy before being considered a professional reference. Future cleanup issues should use the [checklist at the end](#quality-checklist) to track progress example by example.

## Purpose

Examples are the primary onboarding surface for external developers building games with `cougr-core` on Stellar/Soroban. They must be self-contained, buildable without access to the internal monorepo, and explicit about which Cougr pattern they demonstrate. A reader should be able to clone the repo, `cd` into any example, follow its README, and have a working Soroban contract in minutes.

---

## 1. Dependency Requirements

Every example must declare `cougr-core` using the published crate on crates.io, **not** a local path dependency.

```toml
# Correct — external release
[dependencies]
cougr-core = "1.1"
soroban-sdk = "25.1.0"

# Wrong for published examples — breaks for external users
[dependencies]
cougr-core = { path = "../../" }

```

Path dependencies silently work inside the monorepo but fail for any developer who installs the example independently. The published crate is the contract.

### 1.1 Path Dependencies in Monorepo Development

Some canonical examples depend on APIs that have not yet been published to crates.io (e.g. `circuits`, `session`, `SorobanGame`). These examples **must** use path dependencies during monorepo development but **must** switch to published versions before any release tag is cut.

**When path dependencies are permitted:**

* The example exercises an API that exists only in the workspace copy of `cougr-core` (e.g. `cougr_core::circuits::*`, `cougr_core::session::*`, `cougr_core::test::*`).
* The example is under active development inside the monorepo and no crate version on crates.io yet includes the required API.

**When path dependencies must be replaced:**

* Before cutting a release tag (including release candidates), every example's `Cargo.toml` must be updated to reference the published crate version.
* If the required API is available in a published version, the path dependency must be replaced immediately, even outside a release cycle.

**How to mark a path-dependency example:**

Add a comment above the dependency line explaining the exception:

```toml
# path dep — pending cougr-core 1.x publication of circuits::*
cougr-core = { path = "../../" }

```

CI must fail if any example still carries a path dependency at release-tag time.

**Current examples using path dependencies (as of 1.1.0):**

| Example | Reason for path dependency |
| --- | --- |
| `spawn_and_move` | Exercises `SorobanGame` / `impl_soroban_game!` |
| `tic_tac_toe` | Exercises `impl_soroban_game!` and `impl_rich_component!` |
| `session_arena` | Exercises `session::SessionManager` (Beta) |
| `hidden_hand` | Exercises `circuits::hidden_cards` (Experimental) |
| `fog_explorer` | Exercises `circuits::fog_of_war` (Experimental) |
| `dice_duel` | Exercises `circuits::fair_dice` (Experimental) |
| `blind_auction` | Exercises `circuits::sealed_bid` (Experimental) |

---

## 2. Required Validation Commands

Every example must pass both of the following commands without errors or warnings:

```bash
cargo test
stellar contract build

```

`cargo test` validates the game logic in the Soroban test environment. `stellar contract build` validates that the contract compiles to a valid WASM artifact using the Stellar toolchain. A Rust crate that compiles with `cargo build` but fails `stellar contract build` is not a valid Soroban contract.

CI workflows for each example should run both commands on every push to `main` and `develop`.

---

## 3. Module Structure

The source layout of every example should reflect the separation of concerns that Cougr encourages. Monolithic `lib.rs` files are acceptable only for the simplest two- or three-system examples; anything more complex must be split.

### Minimum layout

```
examples/<name>/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs          # Contract entrypoints, #[contract] impl, GameApp wiring
    ├── components.rs   # Cougr components (impl_component! types)
    └── systems.rs      # Game systems registered with GameApp

```

### Extended layout (use when applicable)

| File | When to add |
| --- | --- |
| `types.rs` | Domain enums or structs shared across modules but not Cougr components |
| `auth.rs` | Session key setup, `CougrAccount` wiring, multi-device logic |
| `privacy.rs` | Commit-reveal flows, Merkle proof helpers, hidden-state management |
| `zk.rs` | Groth16 / BLS proof submission and verification wrappers |

Do not add files that are not used. Do not collapse `components.rs` and `systems.rs` back into `lib.rs` once they exist.

---

## 4. README Requirements

Every example must have a `README.md` that covers all of the following sections. Keep each section concise; depth belongs in inline code comments, not in the README.

| Section | Required content |
| --- | --- |
| **Purpose and pattern** | One paragraph: what game mechanic does this demonstrate and which Cougr pattern does it showcase? |
| **Public contract API** | A table or list of every `#[contractimpl]` function: name, parameters, return type, one-line description |
| **Architecture overview** | How systems, components, and the `GameApp` tick interact. A short prose description or ASCII diagram |
| **Storage model** | What is stored in instance storage vs persistent storage vs temporary storage, and why |
| **Main gameplay flow** | Step-by-step description of one complete game round, from initialization to terminal state |
| **Cougr APIs used** | Which modules from `cougr-core` are imported and why (ECS, scheduler, ZK, auth, standards) |
| **Build and test commands** | The exact commands: `cargo test` and `stellar contract build` |
| **Known limitations** | Anything intentionally simplified or out of scope for this example |

Avoid embedding hardcoded contract IDs, testnet addresses, or deployment results in the README. Those belong in a local `NOTES.md` or a developer's own notes.

---

## 5. Testing Requirements

Each example must have a test module (typically `src/lib.rs` inline tests or `src/test.rs`) that covers the following categories:

| Category | What to test |
| --- | --- |
| **Initialization** | Calling the init function produces valid starting state |
| **Happy-path gameplay** | One full game round proceeds without errors |
| **Invalid actions** | Illegal moves, out-of-turn actions, or bad input return the expected error |
| **Rule and invariant tests** | Game-specific invariants hold after any sequence of valid moves |
| **Cougr integration** | If the example uses `GameApp`, `SimpleQueryBuilder`, auth, or ZK APIs, at least one test exercises that integration path |

Tests must use the `soroban-sdk` `testutils` feature and `Env::default()`. Do not mock the Soroban environment at the Rust level; use the SDK test harness.

---

## 6. Repository Hygiene

* Do not commit `target/` directories. Each example's `.gitignore` (or the root `.gitignore`) must exclude them.
* Do not commit `.wasm` artifacts, `*.wasm`, or build output.
* `Cargo.lock` should be committed for examples (they are end-user applications, not libraries).
* Keep `Cargo.toml` minimal: only direct dependencies, no unused features, no wildcard version specifiers.

---

## 7. Canonical vs Transitional Examples

### Canonical examples

A canonical example is a maintained reference architecture. It is held to the full standard in this document and is expected to stay current as `cougr-core` evolves. Canonical examples are the ones new contributors should read first.

Current canonical examples (aligned with the 1.1.0 release):

| Example | Category | Pattern demonstrated |
| --- | --- | --- |
| `spawn_and_move` | Starter | Complete idiomatic Cougr pattern: `SorobanGame` + `impl_component_observed!` + typed ECS |
| `tic_tac_toe` | Rich components | Turn-based game with `impl_rich_component!` for `Address` and `Vec` fields |
| `session_arena` | Session UX | `session::SessionManager`, session lifecycle, and multi-round state (Beta) |
| `hidden_hand` | ZK circuits | `circuits::hidden_cards` — hidden-card ZK proof flow (Experimental) |
| `fog_explorer` | ZK circuits | `circuits::fog_of_war` — fog-of-war exploration with Merkle proofs (Experimental) |
| `dice_duel` | ZK circuits | `circuits::fair_dice` — fair dice roll with on-chain verification (Experimental) |
| `blind_auction` | ZK circuits | `circuits::sealed_bid` — sealed-bid auction with commit-reveal ZK (Experimental) |
| `snake` | Arcade (GameApp) | Arcade loop, `GameApp` tick model, basic ECS |
| `battleship` | Hidden information | Commit-reveal and selective state disclosure using `privacy::stable` Merkle primitives |
| `guild_arena` | Authentication & recovery | Account abstraction, social recovery, multi-device authorization |

### Transitional examples

A transitional example is one that was written before the current standard or that intentionally preserves an older pattern for compatibility reference. It must be clearly marked as such in its own README:

```markdown
> **Transitional example**: This example uses an older Cougr pattern and is preserved
> for compatibility reference. For the current recommended approach, see `snake`.

```

Transitional examples are still expected to pass `cargo test` and `stellar contract build`. They are not required to match the module structure or README depth of canonical examples, but they must not mislead readers into thinking the older pattern is preferred.

---

## 8. Cougr API Usage Guidance

Use the following table to decide which Cougr APIs an example should use and document.

| API | Use when | Stability |
| --- | --- | --- |
| `SorobanGame` trait | Any example that loads/saves `SimpleWorld` to Soroban storage; implement once with `impl_soroban_game!` | Stable |
| `impl_soroban_game!` macro | Wiring `SorobanGame` to a `#[contract]` struct; replaces manual `load_world` / `save_world` boilerplate | Stable |
| `GameApp` | Any example with more than one system or stage | Stable |
| `ScheduleStage` | Systems must run in a defined order within a tick | Stable |
| `SimpleWorld` | The example stores and queries entities with multiple components | Stable |
| `SimpleQueryBuilder` | The example scans entities by component type (more than one entity type) | Stable |
| `impl_rich_component!` | A component contains `Address` or `Vec` fields that need Soroban-native storage | Stable |
| `session::SessionManager` | The example manages multi-round player sessions, timeouts, or session-key lifecycle | Beta |
| `circuits::hidden_cards` | The example demonstrates hidden-card ZK proof flows | Experimental |
| `circuits::fog_of_war` | The example demonstrates fog-of-war exploration with Merkle proof verification | Experimental |
| `circuits::fair_dice` | The example demonstrates fair dice roll with on-chain Groth16 verification | Experimental |
| `circuits::sealed_bid` | The example demonstrates sealed-bid auctions with commit-reveal ZK proofs | Experimental |
| `test::GameHarness` | Writing integration tests that exercise full game rounds in a sandboxed Soroban environment; pair with `Scenario` and `ReplayLog` | Experimental |
| `auth` (Beta) | The example demonstrates session keys, multi-device flows, or account recovery | Beta |
| `privacy::stable` | The example demonstrates commit-reveal, Merkle proofs, or selective disclosure | Stable |
| `privacy::experimental` | The example demonstrates Groth16 proof submission or BN254/BLS12-381 operations | Experimental |
| `ops` standards | The example needs pausability, access control, or ownership transfer | Stable |

When an API is used, the README must explain **why** that API was chosen, not just that it was used.

### API–Example Cross Reference

The following table maps each canonical example to its primary Cougr APIs:

| Example | Primary APIs |
| --- | --- |
| `spawn_and_move` | `SorobanGame`, `impl_soroban_game!`, `impl_component_observed!`, `SimpleWorld`, `SimpleQueryBuilder` |
| `tic_tac_toe` | `SorobanGame`, `impl_soroban_game!`, `impl_rich_component!`, `GameApp` |
| `session_arena` | `SorobanGame`, `session::SessionManager`, `GameApp` |
| `hidden_hand` | `SorobanGame`, `circuits::hidden_cards`, `privacy::experimental` |
| `fog_explorer` | `SorobanGame`, `circuits::fog_of_war`, `privacy::stable` |
| `dice_duel` | `SorobanGame`, `circuits::fair_dice`, `privacy::experimental` |
| `blind_auction` | `SorobanGame`, `circuits::sealed_bid`, `privacy::experimental` |
| `snake` | `GameApp`, `ScheduleStage`, `SimpleWorld`, `SimpleQueryBuilder` |
| `battleship` | `privacy::stable`, `GameApp`, `SimpleWorld` |
| `guild_arena` | `auth`, `ops::Ownable`, `ops::RecoveryGuard`, `GameApp` |

> **Note:** All `circuits::*` functions return a `GameCircuitSpec` and are currently considered **Experimental** (matching the `CHANGELOG.md`).

---

---

## 9. Preview Image

Every canonical example must have a `preview.svg` checked into its directory. This image is displayed in the examples gallery and must represent the actual game state, not a placeholder or hand-drawn mockup.

### Board / grid games

Generate `preview.svg` using the tooling in `tools/preview-gen/`:

```bash
cd tools/preview-gen
node generate.js <example-name>
```

The generator reads a state JSON from `tools/preview-gen/states/<example-name>.json`. That state must be derived from a specific, named test assertion in the example's `src/test.rs` — not invented. See [tools/preview-gen/README.md](../tools/preview-gen/README.md) for the full contributor workflow.

### Real-time / physics arcade games

Games where no single frame conveys how the game works (Snake, Asteroids, Pong, Flappy Bird, Geometry Dash, etc.) may use the category fallback renderer:

```bash
node generate.js snake   # snake is registered in FALLBACK_GAMES
```

This produces a branded category-icon card rather than a fake gameplay screenshot. Using the fallback is acceptable and honest; faking a state screenshot for a real-time game is not.

### Checklist item

See the [Quality Checklist](#quality-checklist) below for the corresponding checklist entry.

---

## Quality Checklist

Copy this checklist into a follow-up cleanup issue for each example:

```markdown
## Example quality checklist — `<example-name>`

### Dependencies
- [ ] Uses published `cougr-core` version, not a path dependency — **or** carries an annotated path dependency exception per §1.1

### Build validation
- [ ] `cargo test` passes
- [ ] `stellar contract build` passes

### Module structure
- [ ] `lib.rs` contains only contract entrypoints and GameApp wiring
- [ ] `components.rs` exists and contains all `impl_component!` types
- [ ] `systems.rs` exists and contains game logic systems
- [ ] Additional modules (`auth.rs`, `privacy.rs`, `zk.rs`, `types.rs`) added only if used

### README
- [ ] Purpose and pattern section present
- [ ] Public contract API documented
- [ ] Architecture overview present
- [ ] Storage model described
- [ ] Main gameplay flow documented
- [ ] Cougr APIs used section present with rationale
- [ ] Build and test commands shown
- [ ] Known limitations noted
- [ ] No hardcoded testnet contract IDs or deployment results

### Tests
- [ ] Initialization test present
- [ ] Happy-path gameplay test present
- [ ] Invalid action test present
- [ ] Rule/invariant test present
- [ ] Cougr integration test present (if applicable)
- [ ] Tests use `soroban-sdk` testutils and `Env::default()`

### Repository hygiene
- [ ] `target/` excluded from version control
- [ ] No committed `.wasm` artifacts
- [ ] `Cargo.lock` committed
- [ ] `Cargo.toml` has no unused dependencies or wildcard versions

### Preview image
- [ ] `preview.svg` present in the example directory
- [ ] State data is traceable to a named test assertion in `src/test.rs` (see `_source` field in the corresponding `states/*.json`)
- [ ] Generated via `tools/preview-gen/` — not hand-drawn or manually edited
- [ ] For real-time/arcade games: category fallback used rather than a faked screenshot

### Classification
- [ ] Marked as canonical or transitional in the README
- [ ] If canonical: category matches one listed in §7 (Starter, Rich components, Session UX, ZK circuits, Arcade, Hidden information, Authentication & recovery)
- [ ] If transitional: banner note present pointing to preferred alternative

