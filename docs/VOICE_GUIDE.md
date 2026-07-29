# Cougr Voice & Terminology Guide

## Purpose

This guide captures the small set of enforced terminology and voice rules that Cougr documentation follows. New documentation, examples, marketing copy, and showcase text must conform to these rules.

These rules are derived from actual usage in Cougr's strongest existing documentation — they codify what the project already does implicitly, not invented conventions.

---

## Enforced Terminology

| Always say | Never say | Why |
|---|---|---|
| **contract** (after first use) | "smart contract" (after first reference) | "smart contract" is fine on first mention for discoverability; all subsequent references use "contract" alone. The project's strongest docs already do this. |
| **component** | "entity data", "property", "field" | Per ECS convention. Components are the typed data attached to entities. |
| **system** | "script", "handler", "processor" | Per ECS convention. Systems are the logic operating on entities via their components. |
| **entity** | "object", "game object", "thing" | Per ECS convention. Entities are opaque runtime identities. |
| **world** | "store", "database" | World is the ECS storage backend, not a generic persistence term. |
| **Stable** | "production-ready", "locked", "finished" | Stable is a specific maturity tier (SemVer-protected, documented, tested). Use the maturity-model term exactly. |
| **Beta** | "unstable", "in development", "almost ready" | Beta is a specific maturity tier (usable, supported, evolving). Use the maturity-model term exactly. |
| **Experimental** | "alpha", "unstable", "proof of concept" | Experimental is a specific maturity tier (exploratory, no compatibility promise). Use the maturity-model term exactly. |
| **privacy::stable** | "zk::stable" (in new docs) | Both namespaces exist, but `privacy::stable` is the preferred domain-facing path for new documentation. |
| **auth** | "accounts" (in application-facing docs) | Both namespaces exist, but `auth` is the clearer Beta-facing domain alias. Use `auth` in onboarding and reference docs. |
| **ops** | "standards" (in application-facing docs) | Both namespaces exist, but `ops` is the clearer stable domain alias. Use `ops` in onboarding and reference docs. |
| **GameApp** (code font) | "game app", "Game App" | The type name `GameApp` is always in code font or backticks. |
| **plugin** | "addon", "module", "extension" | Plugins are the composable bundle mechanism in `GameApp`. |
| **canonical example** | "main example", "primary example", "core example" | Examples are classified as canonical (maintained reference) or transitional (older patterns). "Canonical" is the precise term. |
| **transitional example** | "legacy example", "old example" | "Transitional" precisely indicates the example preserves an older pattern for compatibility, not that it is abandoned. |
| **curated surface** | "stable API", "public API" (when referring to the 1.0 contract) | The curated surface is the *defended* subset of public APIs — not every public symbol is curated. Prefer "curated surface" to avoid overpromising. |

---

## Voice Principles

### 1. Honest-by-default

State gaps plainly rather than marketing around them. If a feature is Beta, say "Beta" — not "stable with some evolution expected." If a verification path is Experimental, say "Experimental" — not "advanced but reliable." This is Cougr's strongest differentiator as a documentation culture.

### 2. Precise over playful

Cougr is infrastructure (an on-chain game engine), not a game itself. Documentation should read closer to Linear or Vercel's developer docs than to a flashy Web3 project's landing page. Avoid:
- Game-themed metaphors for technical concepts ("power up your contract!")
- Exclamation points in technical explanations
- Unnecessary superlatives

### 3. Consistent maturity labels

Whenever a feature, namespace, or API is mentioned, its maturity tier (Stable / Beta / Experimental) must be named or clearly scoped by context. Never present a Beta surface without its maturity label in the same paragraph or table.

### 4. Namespace aliases are explained once

When a new namespace alias exists (`ops` for `standards`, `auth` for `accounts`, `privacy::stable` for `zk::stable`), explain the relationship once in the relevant document and then consistently use the preferred alias. Do not flip back and forth within the same page.

---

## Formatting Conventions

| Element | Formatting |
|---|---|
| Rust types, macros, traits, function names | Code font / backticks |
| Namespace paths (`cougr_core::app`) | Code font / backticks |
| Maturity tiers (Stable, Beta, Experimental) | Title case, no backticks |
| The project name "Cougr" | Title case, no backticks |
| ECS terms (component, system, entity, world) | Lowercase when used conceptually, code font when referring to the Rust type |
| Example names (`spawn_and_move`) | Code font / backticks |

---

## Cross-Reference Check

This voice guide was cross-checked against the source-of-truth documents before publication. The following minor inconsistencies were identified and flagged for cleanup:

| Document | Issue | Fix |
|---|---|---|
| `ARCHITECTURE.md:87` | "smart contract wallet" uses full phrase after first reference | Should be "contract wallet" for consistency (future PR) |
| `README.md:19` | "contract standards" is correct | No fix needed |
| `ECS_CORE.md:14` | Lists `SimpleQuery` in recommended path, while code examples use `SimpleQueryBuilder` | Both are valid, but the list should use `SimpleQueryBuilder` to match usage (future PR) |

---

## Required Reference

All documentation contributors must read:
- [MATURITY_MODEL.md](MATURITY_MODEL.md) — the three maturity tier definitions (Stable, Beta, Experimental)
- [GLOSSARY.md](GLOSSARY.md) — full term definitions
- This voice guide — terminology and voice rules above
