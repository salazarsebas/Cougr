# Cougr Threat Model

## Purpose

This document defines the current threat model baseline for Cougr at `1.0`.

It is not a claim of formal verification or exhaustive review. It exists so the repository is explicit about what must be defended, where trust assumptions live, and which subsystems need extra scrutiny before production use.

## Security Objectives

Cougr aims to preserve:

- authorization correctness for gameplay and account actions
- replay resistance for session-like or delegated actions
- storage integrity for ECS state and account-linked records
- proof-verification correctness for accepted privacy and ZK flows
- predictable state transitions when authorization depends on mutation ordering

## Sensitive Subsystems

The most sensitive subsystems in the current repo are:

- `accounts`
  - authorization decisions
  - session creation and revocation
  - recovery, device, and fallback flows
- `world`, `simple_world`, `archetype_world`, `commands`, `scheduler`
  - mutation ordering and delayed execution
  - correctness of state reads used by auth or proof logic
- `incremental` and persistent storage helpers
  - durability and consistency of serialized state
- `zk`
  - proof verification assumptions
  - commitment and Merkle verification
  - malformed-input handling

## Threat Actors

Assume the following threat classes:

- untrusted users submitting arbitrary contract inputs
- authorized users attempting to exceed granted session scope
- integrators treating beta or experimental modules as stronger guarantees than documented
- adversaries replaying previously valid auth or proof material
- malformed or adversarial proof inputs targeting verification edges

## Out-of-Scope Guarantees

Cougr does not currently guarantee:

- audit-backed production readiness across all public modules
- complete replay resistance across every future integration pattern
- hardened contracts for every advanced ZK abstraction
- compatibility guarantees for non-stable support surfaces

## Primary Threat Areas

### Authorization and Session Flows

Primary risks:

- scope bypass
- expired or exhausted session reuse
- ambiguous fallback behavior
- weak or colliding session identifiers

Current posture:

- accounts remain Beta
- session semantics, replay domains, and typed intents are implemented, but the broader auth contract remains intentionally unfrozen inside the Beta `accounts` namespace

### Replay and Intent Reuse

Primary risks:

- repeating previously valid actions
- reusing delegated auth artifacts outside intended scope
- lack of globally coherent nonce or intent semantics

Current posture:

- replay-sensitive behavior is explicit in the kernel and storage model
- integrations must still review how their own action semantics map onto Cougr's replay domains

### ECS State Integrity

Primary risks:

- authorization depending on stale state
- unsafe assumptions around deferred commands
- observer or scheduler ordering changing effective behavior

Current posture:

- ECS primitives are broadly usable
- stable invariants are frozen for the documented `1.0` contract, while non-contract support surfaces remain out of scope

### Proof Verification and Privacy

Primary risks:

- accepting malformed proofs or malformed public inputs
- overstating the maturity of advanced verification flows
- confusing stable commitments and Merkle utilities with broader confidential abstractions

Current posture:

- commitments, commit-reveal, hidden-state codecs, and Merkle utilities are the stable privacy subset
- advanced ZK verification remains Experimental

## Required Review Areas Before Production Use

Review at minimum:

- auth entrypoints and fallback logic
- session scope, expiration, and revocation behavior
- replay assumptions in the integrating application
- persistent storage schema assumptions
- proof-verification failure modes
- state-transition ordering where auth depends on world mutations

## Relationship to Maturity

This threat model works with:

- [docs/MATURITY_MODEL.md](docs/MATURITY_MODEL.md)
- [docs/API_CONTRACT.md](docs/API_CONTRACT.md)
- [docs/UNSAFE_INVARIANTS.md](docs/UNSAFE_INVARIANTS.md)
- [SECURITY.md](../SECURITY.md)

Any future claim that a subsystem is Stable should be accompanied by tighter invariants, explicit failure modes, and threat-model updates where applicable.
