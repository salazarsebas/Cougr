# Shadow Draft Card Game

A Soroban smart-contract example demonstrating:

- **Hidden-hand draft gameplay** - players commit to a card choice via a SHA-256 hash before either reveal is visible, preventing last-second strategy adaptation.
- **`stellar-zk` proof-backed card validation** - when a verification key is registered, each card play must include a Groth16 proof that the chosen card is a member of the active allowed set, validated on-chain without revealing the card ID until both players have committed.
- **DAO-governed format rules** - any player can propose banning or unbanning a card from the active season format; proposals execute immediately (modelling a passed governance vote).

## Architecture

### Components

| Component | Fields | Purpose |
|---|---|---|
| `DeckComponent` | `player`, `active_format` | Deck context and active season format |
| `HandCommitmentComponent` | `commitment`, `committed`, `revealed`, `revealed_card` | Hidden-hand state per player |
| `BoardStateComponent` | `round`, `score_one/two`, `last_played_*`, `round_state` | Visible board after each resolution |
| `FormatProposalComponent` | `proposal_id`, `ban_card`, `unban_card`, `proposer`, `vote_count`, `status` | DAO governance proposals |
| `GameStatusComponent` | `status`, `phase` | Top-level match lifecycle |

### Systems

| System | Trigger | Responsibility |
|---|---|---|
| `DraftSystem` | Both players committed | Advance `PHASE_DRAFT → PHASE_PLAY` |
| `ProofValidationSystem` | `play_card` | Verify SHA-256 commitment + optional Groth16 proof |
| `CardPlaySystem` | `play_card` | Enforce active format ban list |
| `RoundResolutionSystem` | Both players revealed | Compare card powers, update scores, reset or end match |
| `FormatGovernanceSystem` | `submit_format_proposal` | Apply accepted ban/unban to `banned_cards` |

## Game Flow

```
init_match(p1, p2)
│
└─► Round N
     ├─ PHASE_DRAFT
     │   ├─ p1: submit_choice({ commitment: sha256(card_id || nonce) })
     │   └─ p2: submit_choice({ commitment: sha256(card_id || nonce) })
     │         → DraftSystem advances to PHASE_PLAY
     │
     ├─ PHASE_PLAY
     │   ├─ p1: play_card({ card_id, nonce, proof, public_inputs })
     │   │       ProofValidationSystem: sha256 check + optional Groth16 verify
     │   │       CardPlaySystem: ban-list check
     │   └─ p2: play_card(...)
     │         → RoundResolutionSystem fires
     │
     └─ Resolution
          higher card_power(card_id) wins the round
          first to ROUNDS_TO_WIN (3) wins the match
```

## stellar-zk Integration

The contract uses `cougr-core::zk::groth16::verify_groth16` (compatible with
[stellar-zk](https://crates.io/crates/stellar-zk) /
[salazarsebas/stellar-zk](https://github.com/salazarsebas/stellar-zk)) for
on-chain proof verification.

When a verification key is registered via `set_vk`, each `play_card` call must
supply a valid Groth16 proof attesting that the revealed `card_id` belongs to
the active allowed set. This allows players to prove card legality in
zero-knowledge without leaking which card they chose before the opponent reveals.

```
Public inputs: [ hash_of_commitment, format_id ]
Private inputs: [ card_id, nonce ]
Circuit: sha256(card_id || nonce) == commitment AND card_id ∈ allowed_set[format_id]
```

## Format Governance

Governance references: [governance.script3.io](https://governance.script3.io)

A minimal DAO surface is provided via `submit_format_proposal`. In a production
deployment this would gate execution on an on-chain quorum vote. For the
reference implementation the proposal is auto-accepted upon submission to keep
the flow testable without a separate governance contract.

```rust
// Propose a seasonal card ban
contract.submit_format_proposal(
    proposer,
    ProposalInput { ban_card: 8, unban_card: 0 },
);

// Lift a prior ban
contract.submit_format_proposal(
    proposer,
    ProposalInput { ban_card: 0, unban_card: 8 },
);
```

## Contract API

```rust
fn init_match(env: Env, player_one: Address, player_two: Address)
fn submit_choice(env: Env, player: Address, choice: ChoiceInput)
fn play_card(env: Env, player: Address, play: PlayInput)
fn submit_format_proposal(env: Env, proposer: Address, proposal: ProposalInput)
fn get_state(env: Env) -> GameState
fn set_vk(env: Env, vk: VerificationKey)
```

## Validation Commands

```bash
cd examples/shadow_draft_card_game
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
stellar contract build
```
