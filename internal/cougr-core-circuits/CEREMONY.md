# Production Trusted Setup Ceremony Runbook

## Who this is for

Anyone preparing to deploy a Cougr ZK game circuit (`hidden_cards`, `fog_of_war`, `fair_dice`, `sealed_bid`, or a
custom circuit built the same way) to **mainnet**. If you are only running examples, tests, or a testnet demo, you
do not need this document - `scripts/setup.sh` already gives you a working test key.

## Why this exists

`scripts/setup.sh` in this package runs a Groth16 phase-2 setup against **pot14**
(`powersOfTau28_hez_final_14.ptau`) with a single, locally-generated contribution. That is fine for CI, examples,
and integration tests. It is **not** fine for mainnet:

- the toxic waste from that single contribution exists, briefly, on whatever machine ran `setup.sh` (yours, or a CI
  runner's)
- nothing about that contribution is independently verifiable by a third party
- if that toxic waste is ever reconstructed, an attacker can forge proofs that pass on-chain verification - for a
  game circuit, that means forged dice rolls, forged card deals, forged auction reveals, etc.

A production ceremony's entire job is to make sure no single party - including you - ever knows the toxic waste
for the final key. As long as **at least one** honest participant discards their contribution's randomness, the
resulting key is safe, even if every other participant is malicious or compromised.

## Before you start

Confirm all of the following. Do not start a ceremony until they're true:

- [ ] Your circuit is compiled and stable - the `.circom` source is final, `circom` version is pinned, and
      `scripts/validate-layouts.sh` passes (Circom public signals match the Rust `PublicInputLayout` in
      `src/circuits/spec.rs`). A ceremony run against a circuit you later change is wasted; the key is tied to that
      exact R1CS.
- [ ] You know your circuit's constraint count (see the table in [README.md](./README.md)) and therefore your
      required `ptau` power. `hidden_cards` needs pot14; the others fit in smaller powers, but there is no harm in
      reusing a pot14-or-larger phase-1 transcript for all circuits deployed together.
- [ ] You have identified this circuit's `CircuitId` (`src/circuits/spec.rs`) and are not going to reuse a
      production key across circuit variants (different deck size, different map bounds, etc. are different
      circuits - see the "Public-input freeze" note in [AUDIT.md](./AUDIT.md)).
- [ ] You've decided who your ceremony participants are (see "Choosing participants" below).

## Phase 1: Powers of Tau

You almost never need to run your own phase-1 ceremony. Phase 1 (the "powers of tau") is circuit-agnostic and
several large, well-audited public ceremonies already exist with hundreds to thousands of participants - using one
of these is strictly safer than running your own with fewer participants.

Use the same Hermez/zkEVM `pot14` transcript that `scripts/download-ptau.sh` already fetches
(`powersOfTau28_hez_final_14.ptau`), **or** a phase-1 transcript from another audited ceremony of at least the same
power, such as the [Perpetual Powers of Tau](https://github.com/privacy-scaling-explorations/perpetualpowersoftau)
project. Whichever you choose:

1. Download it independently of this repo's script (don't trust a single mirror - fetch from at least two sources
   and diff the files).
2. Record the exact URL(s), the file's SHA-256, and the ceremony's published transcript/attestation reference in
   your release notes.
3. Do **not** use `scripts/download-ptau.sh`'s local-generation fallback (`snarkjs powersoftau new` +
   single-contribution `prepare phase2`) for production. That fallback exists purely so CI never blocks on a
   flaky download; it produces a single-contributor phase-1 transcript with no external verifiability.

## Phase 2: Circuit-specific ceremony

This is the phase that's specific to each Cougr circuit and is what `scripts/setup.sh` currently does with one
contributor. For production, replace that single step with a real multi-party ceremony:

### 1. Initialize

```bash
snarkjs groth16 setup <circuit>.r1cs pot14_final.ptau <circuit>_0000.zkey
```

Use the phase-1 `.ptau` file from the previous step (not the one `download-ptau.sh` generates locally), and the
`.r1cs` produced by `scripts/compile.sh` for your final, reviewed circuit.

### 2. Sequential contributions

Each participant contributes in turn, each one building on the previous participant's output:

```bash
snarkjs zkey contribute <circuit>_NNNN.zkey <circuit>_NNNN+1.zkey \
  --name="<participant name or handle>" -v
```

For each contribution:

- The participant supplies their own entropy - ideally from a source outside the ceremony coordinator's control
  (their own hardware RNG, dice rolls, mouse jitter, whatever they trust). Do **not** reuse the
  `head -c 32 /dev/urandom` pattern from `scripts/setup.sh` verbatim across participants sharing a machine; each
  participant's randomness must be independently sourced.
- The participant destroys their local entropy and intermediate `.zkey` immediately after contributing - recommend
  a portable/live OS or a machine wiped afterward for anyone who wants strong deletion guarantees.
- `snarkjs` prints a contribution hash after each step. Publish it immediately (a public repo, a pinned message in
  a shared channel, a tweet) **before** moving to the next participant, so no one can claim a different
  contribution after the fact.

### 3. Choosing participants

- **Minimum for a real production ceremony: multiple independent participants**, ideally including at least one
  who is not part of your core team (an auditor, another Stellar/Soroban team, a community member). The security
  property only requires **one honest participant who deletes their randomness**, but you can't verify honesty - more independent participants means fewer entities that would all have to be simultaneously compromised or
  colluding.
- Prefer participants using different machines, different operating systems, and not on the same network segment.
- Document each participant's identity (or verifiable pseudonym) and the order they contributed in.

### 4. Random beacon (optional but recommended)

After the last participant, apply a public randomness beacon as a final contribution nobody could have predicted
in advance - e.g. a future Bitcoin/Ethereum block hash at a pre-announced height, or drand output:

```bash
snarkjs zkey beacon <circuit>_final_contrib.zkey <circuit>_final.zkey \
  <32-byte-beacon-hash-hex> 10 -n="Final Beacon"
```

This closes the ceremony deterministically - anyone can recompute the beacon step and confirm you didn't add a
hidden extra contribution afterward.

### 5. Export and verify

```bash
snarkjs zkey export verificationkey <circuit>_final.zkey <circuit>_vk.json
snarkjs zkey verify <circuit>.r1cs pot14_final.ptau <circuit>_final.zkey
```

`zkey verify` replays every recorded contribution against the transcript and confirms the final key is a valid
composition of all of them - run this yourself, and ask at least one participant to run it independently too.

## Publishing the ceremony record

Before the key goes anywhere near a mainnet contract, publish (in your release notes, a `CEREMONY_RECORD.md` in
your own deploy repo, or similar - this repo's `.gitignore` excludes `keys/` and `exported/`, so records belong in
your release artifacts, not committed here):

- phase-1 transcript source, URL(s), and SHA-256
- phase-2 participant list and contribution order
- each contribution's hash, as published in real time during the ceremony
- the beacon value and block height/source, if used
- the final `<circuit>_vk.json` and its SHA-256
- the exact `circom` compiler version and `.circom` source commit hash the R1CS was built from

Anyone should be able to take this record and `snarkjs zkey verify` their way to the same verification key you
shipped.

## After the ceremony

- Store the final `.zkey` (proving key) off-repo, encrypted, with access limited to whoever needs to generate
  proofs in production (often nobody - many games only need the verification key on-chain and generate proofs
  client-side or don't need the proving key long-term at all).
- Embed only the **verification key** in your contract. See
  [PRODUCTION_KEYS.md](./PRODUCTION_KEYS.md) for how to go from `<circuit>_vk.json` to
  `GameCircuitSpec::with_verification_key`.
- Re-run your integration tests against the new VK before deploying (`GameCircuitSpec::with_verification_key` is
  exactly the swap point - see [PRODUCTION_KEYS.md](./PRODUCTION_KEYS.md)).
- Never let a test key (`scripts/setup.sh` output, or anything with a single, unpublished contribution) reach a
  mainnet contract. If you're unsure which key a deployed contract is using, that's a sign to check before
  shipping, not after.

## Re-running a ceremony

Any change to the circuit's constraints or public-input layout invalidates the ceremony - the new R1CS needs a new
phase-2 setup from scratch. This includes changes that look cosmetic in Circom but change the constraint system
(reordering signals, changing a fixed bound like deck size or map size, etc.). If your public-input layout changes,
you also need a new `CircuitId` per the freeze policy in [AUDIT.md](./AUDIT.md).
