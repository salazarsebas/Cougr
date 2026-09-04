---
name: cougr-game-starter
description: Build new game projects from scratch with cougr-core. Use when Developer needs to scaffold a fresh Rust/Soroban game, add cougr-core as a dependency, design ECS components and systems, implement contract APIs, structure tests, or turn a game idea into a working prototype without relying on prior repository context.
---

# Cougr Game Starter

Build a new game as a small, shippable vertical slice. Treat `cougr-core` as the architectural foundation, not as decoration added after the game logic is already tangled.

## Workflow

1. Classify the game before writing code.
2. Scaffold a minimal project and add `cougr-core`.
3. Model the game state in ECS terms.
4. Implement contract APIs as thin entrypoints over game logic.
5. Add tests that prove the loop works.
6. Leave the project with documentation that another engineer can extend.

Read bundled references as needed:

- Read [references/bootstrap.md](references/bootstrap.md) before creating or restructuring a project.
- Read [references/patterns.md](references/patterns.md) when deciding how to model entities, components, systems, and contract APIs.
- Read [references/prototype-checklist.md](references/prototype-checklist.md) before finishing work or when deciding what belongs in a first prototype.

## Step 1: Classify The Game

Reduce the request to a concrete gameplay loop before creating files.

Capture:

- player count
- turn-based vs real-time tick updates
- hidden information vs fully public state
- win and loss conditions
- smallest playable loop
- one or two mechanics that make the prototype worth building

Prefer a narrow first milestone:

- `pong`, not full sports simulation
- `battleship` placement and attack loop, not matchmaking and rankings
- one arena combat loop, not inventory, guilds, and tournaments at once

If the user asks for a large game, implement the thinnest coherent slice first and state that scope in the project `README`.

## Step 2: Scaffold The Project

Scaffold with the `cougr` CLI. Do not hand-write the project layout, `Cargo.toml`, or a starter contract skeleton, `cougr new` already generates a canonical `lib.rs` / `components.rs` / `systems.rs` layout with a passing test suite, and hand-rolling one only reproduces what the CLI does correctly.

```bash
cargo install cougr-cli
cougr new <name> --template <template>
cd <name>
cargo test
```

Pick the template from the classification in Step 1:

| Game shape | Template |
|---|---|
| Simple loop, fully public state | `starter` |
| Turn-based, multiple rounds or phases | `turn-based` |
| Hidden information, commit-reveal | `hidden-info` |
| Session keys or account abstraction | `session-auth` |

If no template fits closely, scaffold with `starter` and restructure from there rather than building a project by hand.

Use [references/bootstrap.md](references/bootstrap.md) for guidance beyond what the CLI generates: build target flags, dependency strategy for local development against an unpublished `cougr-core`, and how to extend the generated layout.

## Step 3: Model The Game In ECS Terms

Translate the design into:

- entities: players, units, projectiles, cards, tiles, matches
- components: position, health, owner, cooldown, score, turn state, visibility state
- systems: movement, collision, combat resolution, turn advancement, reveal, scoring
- resources or singleton state: match configuration, round timer, board metadata, global phase

Keep contract methods small. Most methods should do one of these:

- initialize state
- submit one player action
- advance one tick or phase
- read state for clients

Do not build monolithic state structs unless the game is genuinely tiny. Even in simple prototypes, separate state by responsibility so later mechanics can be added without rewriting everything.

Use [references/patterns.md](references/patterns.md) for recommended modeling patterns and anti-patterns.

## Step 4: Implement The Contract Surface

Prefer a contract API that is easy to test and easy to evolve. A good first prototype usually has:

- one initializer
- one or a few action entrypoints
- one or two read entrypoints

Examples:

- `init_game`
- `submit_move`
- `advance_tick`
- `get_state`
- `get_board`

Keep the external API stable and move gameplay logic into helper functions or systems. The contract layer should validate inputs, load state, invoke logic, persist state, and return a clear result.

When hidden information or proofs are involved:

- keep commit and reveal phases explicit
- bind proofs or commitments to the right state transitions
- avoid mixing secret-state handling with unrelated gameplay updates

## Step 5: Build Tests Alongside The Loop

Write tests as gameplay proofs, not just line coverage.

Start with:

- initialization works
- one legal action updates state correctly
- one illegal action is rejected
- win, loss, or round transition works
- state cannot advance after the game ends, if that rule exists

Add focused edge-case tests only after the playable path exists.

Prefer tests that read like short match scripts. This keeps the prototype understandable and makes later refactors safer.

## Step 6: Finish With Maintainable Outputs

Before closing the task:

- ensure the `README` explains the loop, commands, and current scope
- ensure build commands use `wasm32v1-none`
- remove throwaway notes, generated reports, and planning debris
- keep the project tree understandable from the root

Use [references/prototype-checklist.md](references/prototype-checklist.md) as the final pass.

## Implementation Rules

- Prefer one clear gameplay loop over broad but shallow feature coverage.
- Keep public methods explicit; avoid magic phase transitions hidden inside unrelated calls.
- Name components by domain responsibility, not by storage details.
- Separate data updates from read formatting where possible.
- Keep comments sparse and useful.
- If a mechanic is complex, document the rule in the local project `README`.
- If the user asks for a prototype, optimize for playability and structure, not premature genericity.

## Cougr-Specific Guidance

- Use `cougr-core` to structure state and logic, not just as a dependency checkbox.
- Start with `SimpleWorld` or straightforward component-oriented organization unless scale clearly requires more.
- Use ECS decomposition even when persistence is serialized as a single contract state object.
- Add advanced capabilities such as ZK flows, session keys, recovery, or multi-device support only when the game design actually benefits from them.
- When those capabilities matter, isolate them into dedicated components and systems instead of folding them into unrelated game rules.

## Output Expectations

When using this skill, deliver:

- a runnable project structure
- core gameplay logic for the requested slice
- tests for the main loop
- a short, professional `README`
- commands aligned with Soroban and `wasm32v1-none`

