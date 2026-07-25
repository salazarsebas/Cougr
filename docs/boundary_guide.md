# On-Chain vs. Off-Chain Boundary Guide in Cougr

This document specifies architectural principles for partitioning game logic between on-chain Soroban smart contracts and off-chain game clients/servers in Cougr applications.

---

## Architectural Responsibility Matrix

| Component | Responsibility | Environment | Latency Requirement |
| :--- | :--- | :--- | :--- |
| **On-Chain Contracts** | Final state verification, asset custody, reward settlement, invariant validation | Soroban WASM Runtime | Sub-second (~5s settlement) |
| **Off-Chain Engine** | Real-time game loops, physics simulation, user input handling, rendering | Browser / WebAssembly / Native Game Server | High-frequency (60-120 FPS, <16ms) |
| **Relay / Indexer** | Event indexing, state proofs, transaction batch submission | Node.js / Rust Relay | Asynchronous (<1s) |

---

## State Transition Verification

```
[ Client Input ] ────> [ Off-Chain Simulation ] ────> [ Generate State Proof / Action Hash ]
                                                                   │
                                                                   ▼
[ Investor / Player ] <─── [ On-Chain State Settled ] <─── [ Submit Action to Contract ]
```

1. High-frequency actions (movement, rotation, collisions) are calculated off-chain.
2. Checkpoint state hashes and critical match outcomes (winning score, item acquisitions) are submitted on-chain for verification and payout.

---

## Invariant Safety Rules

- **Rule 1:** Never store per-frame tick data in contract persistent storage.
- **Rule 2:** Contracts MUST independently re-validate state hashes against game rules before transferring funds or minting items.
- **Rule 3:** Off-chain clients MUST degrade gracefully if chain network delays occur.
