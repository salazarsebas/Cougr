# Security Policy

## Status

Cougr now defines a `1.0.0` stable contract for a scoped subset of the crate. Not every public subsystem is part of that stable guarantee.

Security-sensitive areas include:

- account authorization
- session lifecycle and replay protection
- persistent storage integrity
- proof verification and privacy primitives
- ECS mutation ordering where authorization depends on state transitions

## Maturity and Guarantees

Current guidance:

| Area | Status | Guidance |
|---|---|---|
| ECS runtime and storage | Stable | Part of the `1.0` contract when used through the documented onboarding and runtime surfaces |
| Accounts and smart-account flows | Beta | Do not assume full production guarantees without project-specific review |
| Standards layer (`standards`) | Stable | Reusable contract primitives are part of the `1.0` stable contract |
| Privacy primitives (`zk::stable`) | Stable | Commit-reveal, hidden-state codecs, and Merkle utilities are the stable privacy contract |
| Advanced ZK verification | Experimental | Treat as non-stable until verification contracts and assumptions are fully hardened |

The latest maturity definitions live in [docs/MATURITY_MODEL.md](docs/MATURITY_MODEL.md).
The current threat-model baseline lives in [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md).
The explicit compatibility story lives in [docs/COMPATIBILITY_PROMISES.md](docs/COMPATIBILITY_PROMISES.md).

## Threat Model Expectations

Cougr does not currently claim:

- external audit coverage
- formal verification
- full production guarantees across all auth and privacy paths
- stable compatibility guarantees for experimental modules

Before adopting Cougr in security-critical deployments, review at minimum:

- auth and signer flows
- replay handling
- session scope and revocation rules
- storage schema assumptions
- proof verification assumptions

## Reporting a Vulnerability

If you find a security issue:

1. Do not open a public issue with exploit details.
2. Report the issue privately to the project maintainers.
3. Include:
   - affected module
   - reproduction steps
   - impact assessment
   - version or commit information
   - suggested mitigation if available

The monitored disclosure channel is GitHub Private Vulnerability Reporting:

https://github.com/salazarsebas/Cougr/security/advisories/new

`security@cougr.dev` is reserved as a future dedicated inbox and is not yet independently staffed. Do not rely on it for acknowledgment SLAs.

## Supported Versions

The latest stable release line and current mainline development state should be assumed relevant for fixes unless a maintenance policy says otherwise.

## Secure Contribution Expectations

Changes affecting auth, privacy, storage, or unsafe internals should include:

- updated invariants or trust assumptions
- negative-path tests
- compatibility notes when public behavior changes
- documentation changes when guarantees or maturity shift
