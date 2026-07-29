# Cougr Privacy Model

## Purpose

This document defines Cougr's privacy and proof-verification contract after the `1.0` release gate.

Its job is to separate the stable privacy subset from experimental proof systems so
that the repository can make a smaller, stronger claim about what is safe to depend
on in the stable contract.

## Stable Privacy Surface

The stable privacy subset in Cougr is:

- commitments
- commit-reveal flows
- hidden-state encoding interfaces
- Merkle inclusion verification
- sparse Merkle utilities
- privacy interfaces:
  - `CommitmentScheme`
  - `MerkleProofVerifier`
  - `HiddenStateCodec`
  - `ProofVerifier` as an interface contract only

These are exposed through:

- `cougr_core::privacy::stable`
- `cougr_core::zk::stable` as the compatibility alias

## Experimental Privacy Surface

The following remain Experimental:

- Groth16 proof verification flows
- proof-submission execution helpers
- prebuilt verification circuits
- fog-of-war Merkle exploration orchestration
- multiplayer ZK state-channel transition contracts
- recursive proof-composition descriptors
- advanced hidden-state automation
- hazmat Poseidon-based privacy helpers
- broader confidential-state abstractions

These are exposed through:

- `cougr_core::privacy::experimental`
- `cougr_core::zk::experimental` as the compatibility alias

Compatibility note:

Experimental modules may still be re-exported from `cougr_core::zk` for transition
convenience, but they are not part of Cougr's stable privacy promise. New application
code should prefer `cougr_core::privacy::experimental` so the product-level intent is
obvious at the import site.

## `1.0` Privacy Freeze

The frozen `1.0` privacy contract is exactly:

- commitments
- commit-reveal flows
- hidden-state codec interfaces
- Merkle inclusion verification
- sparse Merkle utilities
- the interface contracts re-exported from `cougr_core::privacy::stable`

The following are explicitly excluded from the `1.0` stable privacy contract:

- `zk::experimental`
- proof-submission orchestration that depends on experimental verification
- Groth16 verifier implementations
- prebuilt advanced circuit helpers
- state-channel, recursive, and fog-of-war orchestration helpers

## Privacy Maturity Table

| Surface | Status | Notes |
|---|---|---|
| Commitments | Stable | Explicit interface and verification contract |
| Commit-reveal | Stable | Explicit component semantics and deadline behavior |
| Hidden-state encoding | Stable | Stable codec interface; fixed-width codecs can be defended |
| Merkle inclusion and sparse Merkle utilities | Stable | Malformed proof behavior and inclusion semantics are explicit |
| Proof submission systems | Beta | Useful orchestration, but still coupled to experimental verification flows |
| Groth16 verification and prebuilt circuits | Experimental | Assumptions are explicit, but not yet strong enough for a stable promise |

## Proof Verification Contract

Cougr's experimental Groth16 verifier makes these explicit guarantees:

- verification keys must satisfy `vk.ic.len() == public_inputs.len() + 1`
- malformed verification-key shape returns `ZKError::InvalidVerificationKey`
- malformed pairing inputs return `ZKError::InvalidInput`
- a well-formed but invalid proof returns `Ok(false)` only when the pairing check fails

Cougr does not currently claim stronger guarantees for Groth16 around:

- subgroup validation beyond Soroban host-type decoding
- normalization guarantees beyond fixed-width typed wrappers
- broader proof-system maturity for production confidentiality claims

That is why the implementation remains Experimental even though the verifier
interface is explicit.

## Merkle Verification Contract

Cougr's stable Merkle verification guarantees:

- malformed proofs with `siblings.len() != depth` return `ZKError::InvalidProofLength`
- well-formed but non-matching proofs return `Ok(false)`
- sparse Merkle utilities produce the same on-chain proof representation used by
  the stable SHA256 verifier

## Hidden-State Encoding Contract

Stable hidden-state codecs must:

- define an exact byte-level representation
- reject malformed encoded state with `ZKError::InvalidInput`
- avoid silent truncation or padding

The built-in `Bytes32HiddenStateCodec` satisfies this by requiring an exact
32-byte payload in both directions.

## Relationship to Public Surface

This model works with:

- [docs/MATURITY_MODEL.md](docs/MATURITY_MODEL.md)
- [docs/API_CONTRACT.md](docs/API_CONTRACT.md)
- [docs/PUBLIC_GAPS.md](docs/PUBLIC_GAPS.md)
- [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md)

Any future claim that advanced proof verification is Stable should add stronger
input-validation guarantees, clearer host-assumption boundaries, and tighter
negative-path coverage than exists today.
