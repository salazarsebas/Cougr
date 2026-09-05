# ️ Guild Treasury Wars - DAO-Governed Factions with stellar-zk Commitments

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/salazarsebas/Cougr)
[![Tests](https://img.shields.io/badge/tests-14%20passing-brightgreen)](https://github.com/salazarsebas/Cougr)
[![Stellar](https://img.shields.io/badge/Stellar-Testnet-blue)](https://stellar.org)

A guild-based strategy game implemented as a **Soroban smart contract** using `cougr-core`'s ECS framework. Guilds manage shared treasuries, vote on strategic actions through DAO mechanics, and compete through hidden strategic commitments powered by **stellar-zk**.

## Why stellar-zk?

In strategy games, revealing your plans to the enemy is a losing move. **stellar-zk** enables **sealed war plans** - players commit to strategies without revealing them, then prove their commitments on-chain when it's time to resolve battles.

| Without stellar-zk | With stellar-zk |
|---|---|
| All actions are public on-chain | Strategies are hidden until reveal |
| Opponents can counter every move | Sealed commitments prevent scouting |
| No strategic depth | Real hidden-information gameplay |
| Generic DAO demo | Stellar-native governance game |

---

## Gameplay Flow

```
┌─────────────────────────────────────────────────────────┐
│ 1. GUILD CREATION                                        │
│    Admin creates a guild with INITIAL_TREASURY (1000)     │
│    → Members join the guild (up to 10)                    │
│    → cougr-core ECS World tracks entities                 │
├─────────────────────────────────────────────────────────┤
│ 2. GOVERNANCE (DAO-style proposals)                       │
│    Members submit proposals: Defend, Attack, Upgrade,     │
│    or Allocate resources from the shared treasury          │
│    → 51% vote threshold for approval/rejection             │
│    → Inspired by Stellar governance (governance.script3)   │
├─────────────────────────────────────────────────────────┤
│ 3. TREASURY EXECUTION                                     │
│    Approved proposals deduct treasury and apply effects:   │
│    → Defend (100): +10 defense strength                   │
│    → Attack (200): Launch campaign against enemy guild     │
│    → Upgrade (150): +10 attack strength                   │
│    → Allocate: Custom resource allocation                  │
├─────────────────────────────────────────────────────────┤
│ 4. SEALED WAR PLANS (stellar-zk commit-reveal)            │
│    Members commit: SHA256(action || target || amount ||    │
│    salt) → hash stored on-chain, strategy hidden           │
│    Reveal: provide preimage, contract verifies hash        │
│    → Nullifier consumed (anti-replay protection)           │
├─────────────────────────────────────────────────────────┤
│ 5. BATTLE RESOLUTION                                      │
│    Attack strength vs defense strength                     │
│    → Winner plunders defender's treasury                   │
│    → Game round incremented                                │
└─────────────────────────────────────────────────────────┘
```

---

## stellar-zk Integration

This example uses stellar-zk for **real hidden-strategy gameplay**, not just narrative flavor. The integration follows stellar-zk's on-chain verification patterns:

### Where stellar-zk Fits

| Phase | stellar-zk Role | Contract Function |
|---|---|---|
| **Commit** | SHA256 hash seals the war plan | `submit_strategy_commitment()` |
| **Reveal** | Preimage verification proves the plan | `reveal_strategy()` |
| **Nullifier** | Anti-replay prevents double-reveal | `DataKey::Nullifier(hash)` |
| **Resolve** | Deterministic outcome after verification | `resolve_battle()` |

### Commitment Hash

```
commitment = SHA256(action_type ∥ target_guild_id ∥ resource_amount ∥ salt)
```

The `salt` (32-byte random value) ensures that identical strategies produce different hashes, preventing pattern analysis by opponents.

### Nullifier Pattern

Each commitment hash acts as a **nullifier** (following stellar-zk's verifier contract design). Once revealed:
1. The commitment is marked as `revealed = true`
2. The hash is stored in `DataKey::Nullifier(hash) → true`
3. Any attempt to reuse the same commitment is rejected

### References

- [stellar-zk](https://github.com/salazarsebas/stellar-zk) - ZK DevKit for Stellar/Soroban
- [stellar-zk on crates.io](https://crates.io/crates/stellar-zk) - Crate reference
- [Stellar Governance](https://governance.script3.io) - DAO voting patterns inspiration

---

## Cougr-Core Integration

### ECS Components

| Component | Fields | Purpose |
|---|---|---|
| `Guild` | `admin`, `name`, `treasury`, `member_count`, `defense/attack_strength` | On-chain faction |
| `Proposal` | `proposer`, `action`, `votes_for/against`, `status` | DAO governance |
| `StrategyCommitment` | `commitment_hash`, `committer`, `revealed` | Sealed war plan |
| `StrategyReveal` | `action_type`, `target_guild_id`, `resource_amount`, `salt` | Revealed preimage |
| `GameState` | `total_guilds`, `total_proposals`, `active_campaigns`, `round` | Global state |

### ECS World

```rust
// cougr-core ECS entity management
let mut world = cougr_core::SimpleWorld::new(&env);
let _guild_entity = world.spawn_entity();
```

---

## Contract API

### Guild Management

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `init_guild` | `guild_admin` | `u32` | Create guild with treasury |
| `join_guild` | `guild_id`, `member` | - | Join an existing guild |

### Governance

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `submit_proposal` | `proposer`, `proposal: ProposalInput` | `u32` | Create a DAO proposal |
| `vote` | `voter`, `proposal_id`, `support: bool` | - | Cast a vote |
| `execute_proposal` | `proposal_id` | - | Execute approved proposal |

### Strategy (stellar-zk)

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `submit_strategy_commitment` | `guild_member`, `proof_input: ProofInput` | - | Seal a war plan |
| `reveal_strategy` | `guild_member`, `guild_id`, `reveal: StrategyReveal` | - | Reveal and verify |
| `resolve_battle` | `attacker_member`, `defender_member`, `attacker_guild_id`, `defender_guild_id` | - | Resolve outcome |

### Queries

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `get_state` | - | `GameState` | Global game state |
| `get_guild` | `guild_id` | `Guild` | Guild info |
| `get_proposal` | `proposal_id` | `Proposal` | Proposal info |

---

## ️ Quick Start

### Prerequisites

| Tool | Version | Installation |
|---|---|---|
| Rust | 1.88.0+ | [rustup.rs](https://rustup.rs) |
| Stellar CLI | Latest | [Stellar Docs](https://developers.stellar.org/docs/tools/cli) |
| WASM Target | - | `rustup target add wasm32v1-none` |

### Build

```bash
# Standard Rust build
cargo build

# Build WASM for Soroban deployment
stellar contract build
```

### Test

```bash
cargo test
```

### Validation

```bash
# Full validation suite
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
stellar contract build
```

**Test Results**: 14 tests passing ✅

| Test | Description |
|---|---|
| `test_init_guild` | Guild creates with correct treasury and state |
| `test_join_guild` | Member joins and count increments |
| `test_submit_proposal` | Proposal created with correct fields |
| `test_unauthorized_proposal` | Non-member rejected |
| `test_vote_counting_and_threshold` | 51% threshold approves |
| `test_vote_rejection` | Majority-against rejects |
| `test_unauthorized_vote` | Non-member vote rejected |
| `test_double_vote` | Double-voting prevented |
| `test_treasury_execution_approved` | Treasury deducted, effects applied |
| `test_execute_unapproved` | Non-approved execution rejected |
| `test_strategy_commitment_and_reveal` | Full commit-reveal flow |
| `test_invalid_reveal` | Wrong preimage rejected |
| `test_double_reveal_prevention` | Nullifier blocks re-reveal |
| `test_battle_resolution` | Attack resolves, resources transfer |

---

## Project Structure

```
examples/guild_treasury_wars/
├── Cargo.toml          # Dependencies: cougr-core + soroban-sdk
├── README.md           # This documentation
└── src/
    ├── lib.rs          # Contract entry points
    ├── types.rs        # ECS Components (Guild, Proposal, Strategy, etc.)
    ├── governance.rs   # ProposalSystem, VotingSystem, TreasuryExecutionSystem
    ├── strategy.rs     # StrategyProofSystem, ResolutionSystem (stellar-zk)
    └── test.rs         # Unit tests (14 tests)
```

---

## License

MIT OR Apache-2.0
