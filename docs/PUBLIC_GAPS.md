# Public Surface Gaps

## Purpose

This document lists public-facing behaviors and constraints that remain intentionally outside Cougr's stable promise after the `1.0` release gate.

The goal is to keep the stable story honest. If a behavior is still evolving, security-sensitive, or not yet defensible as a long-term contract, it belongs here instead of being implied as stable.

## Current Gaps

### Accounts and Smart-Account Flows

Status: Beta

Remaining gaps:

- the account kernel, signed intents, and replay domains are implemented, but the public auth contract is still Beta and not yet SemVer-frozen
- replay protection is explicit, but not yet presented as a stable cross-crate compatibility promise
- fallback authorization behavior is documented, but it should still be reviewed carefully per integration

### Privacy and ZK

Status: Stable contract plus Experimental extensions

Remaining gaps:

- stable privacy primitives are intentionally narrow and do not imply stable advanced proof verification
- advanced proof-verification and confidential abstractions are explicitly Experimental
- proof-submission orchestration remains Beta where it depends on experimental verification flows

### Broad Top-Level Crate Surface

Status: Stable contract plus Beta extensions

Remaining gaps:

- the crate still exports more modules than the frozen stable contract actually guarantees
- internals-heavy modules remain public in places where durable invariants are not yet fully documented
- migration still depends on user choice; the crate does not automatically force older imports onto the curated path

### Scheduling and Query Ergonomics

Status: Stable default path plus some evolving edges

Remaining gaps:

- `GameApp`, declarative runtime system registration, and `SimpleQuery` are now the intended default path
- dependency validation is stage-local by design; cross-stage ordering should remain a phase concern, not an arbitrary graph promise

### Standards Layer

Status: Stable

Remaining gaps:

- integrating contracts are still responsible for composing caller authentication around generic state-machine helpers such as `Pausable` and `RecoveryGuard`

## Removed or Downgraded During Phase 0

- deprecated placeholder helpers were removed from `src/lib.rs` in favor of the curated root API
- accounts and advanced privacy features are described as Beta or Experimental rather than stable production guarantees

## How To Use This List

Treat this document as the current boundary between:

- what Cougr can present as intentionally supported today
- what is still usable but evolving
- what must not be interpreted as part of a stable production contract

For the practical migration path toward the curated surface, see [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md).
