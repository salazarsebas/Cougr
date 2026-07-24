# UX Strategy: The Complete Developer Journey

*Part 9 of 17. Mapping "I have an idea" to "my game is in production with active players," with friction points named at each stage and audience-specific notes.*

## Stage 1: Discovery ("I have an idea, could Stellar do this")

**Current state.** A developer searching for on-chain game tooling on Stellar has almost nothing to find: no landing page, 8 GitHub stars, no showcase. Even a developer who finds the repository via crates.io or a Stellar ecosystem list has to read source and Markdown to understand what Cougr actually offers.

**Target state.** A one-page, fast-loading site (can be the docs site's homepage) that states the value proposition in one sentence backed by the actual differentiators (ECS plus privacy plus account abstraction, in one crate), shows three or four showcase screenshots, and has exactly one obvious next action: "Start a project" leading to the CLI install/quickstart. This is the single biggest lever on the entire journey, because every later stage is unreachable if this one fails.

## Stage 2: First five minutes ("let me try it")

**Current state.** No CLI. The fastest current path is cloning the whole repository and reading `examples/README.md` to find something to copy, a 15 to 30 minute process for someone who already knows Rust and Soroban, longer for anyone else.

**Target state.** `cargo install cougr-cli && cougr new my-game && cd my-game && cargo test` produces a passing test against a real, working game skeleton in under two minutes on a machine that already has Rust installed. This is the specific, measurable bar the CLI work in [06-product-strategy.md](./06-product-strategy.md) is designed to hit, and it should be tested with a stopwatch against a clean machine before being called done, not assumed.

## Stage 3: Learning the model ("how does this actually work")

**Current state.** Reference documentation is strong (ARCHITECTURE.md, ECS_CORE.md, PATTERNS.md) but there is no guided, sequential tutorial; a newcomer has to synthesize the mental model themselves from reference material written for people who already have it.

**Target state.** A single, canonical "Build Your First Game" tutorial (see [12-documentation-architecture.md](./12-documentation-architecture.md)) that builds one real example end to end, introducing components, systems, the scheduler, and a deploy step in a fixed order, with no branching, so it can be followed literally rather than adapted. This is what turns Cougr's existing reference-quality docs into onboarding-quality docs without rewriting the reference material itself.

## Stage 4: Building for real ("I want to add X to my game")

**Current state.** A developer building past the tutorial stage has to read example source directly to find patterns (how do I add hidden information, how do I gate an action behind an access-control role). This works for an experienced Rust developer and is a real wall for anyone else.

**Target state.** `cougr add <piece>` (see [06-product-strategy.md](./06-product-strategy.md)) plus a `PATTERNS.md` organized by problem ("I want fairness," "I want to gate an action," "I want off-chain-friendly real-time movement") rather than by module name, so the entry point is the developer's actual question, not Cougr's internal architecture.

## Stage 5: Testing and shipping ("does this actually work, and is it safe to deploy")

**Current state.** `GameHarness`/`Scenario`/snapshot testing is genuinely strong but undocumented as a standalone guide; resource-cost surprises at deploy time are a real, currently unaddressed risk (see [04-onchain-gaming-research.md](./04-onchain-gaming-research.md)).

**Target state.** `cougr check` plus a documented testing guide plus (medium-term) resource-cost reporting in the test harness, so a team knows before deploying whether their design fits Soroban's resource model, not after a failed or unexpectedly expensive mainnet transaction.

## Stage 6: Player-facing launch ("real players are now playing this")

**Current state.** Only one example (`murdoku`) demonstrates an actual client. Wallet/session UX is solved at the contract level but has no client-side counterpart most teams can adopt directly.

**Target state.** The client SDK described in [06-product-strategy.md](./06-product-strategy.md), paired with the passkey/session flow, so "add a sign-in button that doesn't require a seed phrase" is a documented recipe, not a from-scratch integration project for every team.

## Stage 7: Growth and retention ("people keep playing, and other developers notice")

**Current state.** No discoverability mechanism exists for a shipped game (see [04-onchain-gaming-research.md](./04-onchain-gaming-research.md), problem 8).

**Target state.** The showcase becomes a live directory a shipped game can be added to (with the "Cougr Verified" badge from [05-ecosystem-vision.md](./05-ecosystem-vision.md) as a trust signal), turning individual team success into ecosystem-visible proof, which feeds back into Stage 1 discovery for the next developer.

## Audience-specific notes

**Blockchain beginners with Rust experience.** The largest realistic near-term audience. The tutorial (Stage 3) should assume Rust competence but zero Soroban/Stellar knowledge, and should explicitly explain the concepts a Rust developer coming from traditional software has no prior exposure to: what a contract invocation costs, what resource limits mean, why storage is different from a normal database.

**Experienced Soroban developers.** The audience most likely to evaluate Cougr against "just writing raw `soroban-sdk`." For them, the value proposition has to be demonstrated, not asserted, the timed before/after comparison recommended in [03-competitive-analysis.md](./03-competitive-analysis.md) is aimed squarely at this audience.

**Game designers and technical artists without deep Rust experience.** Currently the least served audience, and honestly so: Cougr requires Rust today, and this research does not recommend building a no-code layer to serve this audience prematurely (see [05-ecosystem-vision.md](./05-ecosystem-vision.md) on the visual editor). The realistic near-term accommodation is documentation written so a designer can read the tutorial and understand the game-logic concepts even if they hand implementation to a Rust-capable teammate, not a promise that they can build alone yet.

**Studios.** Care most about production readiness, licensing clarity, and support channels, all addressed by the maturity model, MIT license, and (medium-term) the verification badge and hosted-service option in [07-business-strategy.md](./07-business-strategy.md), not by new engineering.

**Educators and hackathon participants.** Care most about time-to-first-result and a low-friction path for a cohort of students or hackers to start simultaneously, directly served by the CLI and tutorial track, and by the RFG board giving hackathon participants a menu of meaningful starting points instead of a blank page.
