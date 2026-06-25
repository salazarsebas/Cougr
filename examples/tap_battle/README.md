# 🎮 Tap Battle — Mobile-First Competitive Tapping Game

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/salazarsebas/Cougr)
[![Tests](https://img.shields.io/badge/tests-19%20passing-brightgreen)](https://github.com/salazarsebas/Cougr)
[![Stellar](https://img.shields.io/badge/Stellar-Testnet-blue)](https://stellar.org)

A competitive tapping game implemented as a **Soroban smart contract** using `cougr-core`'s ECS framework with **passkey authentication** (secp256r1/WebAuthn) on the Stellar blockchain.

## 🔐 Why Passkeys?

Traditional blockchain games require seed phrases and per-transaction wallet prompts. **Tap Battle eliminates both**:

| Traditional | With Passkeys |
|---|---|
| Write down 24-word seed phrase | Tap Face ID / Touch ID |
| Approve every transaction in wallet | Session key handles gameplay |
| Complex key management | Browser/OS manages credentials |
| Crypto-native only | Anyone can play |

Passkeys use the **secp256r1** curve (the same as WebAuthn/FIDO2), enabling biometric authentication natively on mobile devices without any crypto knowledge.

---

## 🚀 Mobile-First Authentication Flow & SessionManager Lifecycle

Tap Battle uses `cougr-core` 1.1.0's `session::SessionManager` as the beta-standard API for session approval, execution, renewal, and status queries. For a minimal, canonical demonstration of the session lifecycle without passkeys, see the [session_arena](../session_arena/README.md) example.

```
┌─────────────────────────────────────────────────────────┐
│ 1. REGISTRATION                                         │
│    Player registers passkey (secp256r1 public key)       │
│    → Secp256r1Storage persists the key on-chain          │
│    → No seed phrases, no mnemonics                       │
├─────────────────────────────────────────────────────────┤
│ 2. AUTHENTICATION + SESSION APPROVAL                     │
│    Player authenticates via passkey (Face ID / Touch ID)  │
│    → verify_secp256r1() validates the signature           │
│    → SessionManager::approve creates gameplay session     │
│    → Session scoped to: tap, use_power_up                 │
├─────────────────────────────────────────────────────────┤
│ 3. GAMEPLAY (gasless via SessionManager)                  │
│    Rapid tapping → tap(session_key) executes action       │
│    SessionManager::execute_action enforces scope/budget   │
│    Combos: consecutive taps within time window = multi    │
│    Power-ups: spend combo charges for burst effects       │
├─────────────────────────────────────────────────────────┤
│ 4. RENEWAL & FALLBACK                                   │
│    SessionManager::status prompts renewal before expiry   │
│    SessionManager::renew extends session window           │
│    SessionManager::fallback_execute falls back to direct  │
├─────────────────────────────────────────────────────────┤
│ 5. RESULT                                                │
│    Compare scores → winner determined                    │
│    Stats recorded on-chain in PlayerProfile               │
└─────────────────────────────────────────────────────────┘
```

---

## 🔧 Cougr-Core Integration

This example showcases two key cougr-core features:

### Passkey Authentication (`secp256r1_auth`)

```rust
use cougr_core::auth::{Secp256r1Key, Secp256r1Storage, verify_secp256r1};

// Register: store public key on-chain
Secp256r1Storage::store(&env, &player, &key);

// Authenticate: verify biometric signature
verify_secp256r1(&env, &pubkey, &message, &signature)?;
```

### Session Manager (`SessionManager`)

```rust
use cougr_core::accounts::SessionBuilder;
use cougr_core::session::SessionManager;

// Create scoped session scope
let scope = SessionBuilder::new(&env)
    .allow_action(symbol_short!("tap"))
    .allow_action(Symbol::new(&env, "use_power_up"))
    .max_operations(500)
    .expires_in(duration)
    .build_scope();

// Approve session via SessionManager
SessionManager::approve(&env, &player, scope)?;
```

### ECS Components

| Component | Fields | Purpose |
|---|---|---|
| `PasskeyIdentity` | `pubkey`, `registered_at` | Player's passkey credential |
| `TapCounter` | `count`, `combo`, `multiplier`, `last_tap_ledger` | Tap tracking per round |
| `PowerUp` | `charges`, `kind` | Charged abilities from combos |
| `RoundState` | `started_at`, `duration`, `scores`, `finished` | Match state |
| `PlayerProfile` | `total_wins`, `total_taps`, `best_combo` | Persistent stats |
| `SessionState` | `player`, `expires_at`, `ops_remaining` | Legacy session definition |

---

## 📖 Contract API

### Authentication

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `register_passkey` | `player`, `pubkey: BytesN<65>` | - | Register a secp256r1 passkey |
| `authenticate_and_start_session` | `player`, `signature`, `challenge`, `duration` | `Address` | Auth + create session |

### Gameplay & Session Lifecycle

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `tap` | `session_key` | `TapResult` | Submit a tap (gasless) |
| `use_power_up` | `session_key`, `power_up: u32` | - | Activate a power-up |
| `renew_session` | `owner`, `key_id`, `expires_in` | `ActiveSession` | Extend session lifetime |
| `session_status` | `owner`, `key_id` | `SessionStatus` | Check session health |
| `fallback_tap` | `owner`, `key_id` | `TapResult` | Tap with session/direct fallback |
| `get_latest_session_key`| `player` | `BytesN<32>` | Get latest session key ID |
| `start_round` | `player_a`, `player_b`, `duration` | - | Start a match |
| `get_round` | - | `RoundState` | Get round state (auto-finalizes) |
| `get_profile` | `player` | `PlayerProfile` | Get persistent stats |

### Power-Up Types

| ID | Name | Effect |
|---|---|---|
| 0 | DoubleTap | +10 bonus points |
| 1 | Shield | Defensive (blocks opponent effects) |
| 2 | Burst | +25 bonus points |

---

## 🎮 Game Mechanics

### Combo System

Taps within **5 ledger sequences** of each other maintain a combo streak:

| Combo | Multiplier | Effect |
|---|---|---|
| 1 | 1x | Normal tap |
| 2-4 | 2-4x | Building combo |
| 5 | 5x | Earns a power-up charge |
| 10+ | 10x (max) | Maximum multiplier |

### Game Constants

| Constant | Value | Description |
|---|---|---|
| `COMBO_WINDOW` | 5 ledgers | Time window for combo |
| `MAX_MULTIPLIER` | 10 | Maximum score multiplier |
| `COMBO_CHARGE_THRESHOLD` | 5 | Combos to earn a power-up charge |
| `DOUBLE_TAP_BONUS` | 10 | DoubleTap power-up bonus |
| `BURST_BONUS` | 25 | Burst power-up bonus |
| `DEFAULT_MAX_OPS` | 500 | Default session operations |

---

## 🏗️ Quick Start

### Prerequisites

| Tool | Version | Installation |
|---|---|---|
| Rust | 1.70.0+ | [rustup.rs](https://rustup.rs) |
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

**Test Results**: 19 tests passing ✅

| Test | Description |
|---|---|
| `test_register_passkey` | Passkey registration stores key on-chain |
| `test_register_multiple_players` | Multiple players register independently |
| `test_session_creation` | Session created and validated |
| `test_session_ops_decrement` | Operations decremented per action |
| `test_session_ops_exhausted` | Session stops after ops limit |
| `test_tap_increments` | Tap counter increments correctly |
| `test_combo_streak` | Combo streak with multiplier |
| `test_multiplier_cap` | Multiplier capped at MAX_MULTIPLIER |
| `test_power_up_charge_and_use` | Combo charges power-ups |
| `test_power_up_no_charges` | No activation without charges |
| `test_start_round` | Round initializes correctly |
| `test_start_round_no_passkey` | Round requires registered passkeys |
| `test_round_scoring_and_winner` | Scores tracked per player |
| `test_profile_stats` | Profile tracks taps and combos |
| `test_default_profile` | Default profile for new players |
| `test_session_expiry_during_round` | Session expires correctly |
| `test_combo_breaks_outside_window` | Combo breaks outside window |
| `test_renew_session_extends_play_window` | Session renewal extends window |
| `test_fallback_tap_uses_direct_auth_after_session_expires` | Fallback tap works |

---

## 📁 Project Structure

```
examples/tap_battle/
├── Cargo.toml          # Dependencies: cougr-core + soroban-sdk
├── README.md           # This documentation
└── src/
    ├── lib.rs          # Contract entry points
    ├── types.rs        # ECS Components (PasskeyIdentity, TapCounter, etc.)
    ├── auth.rs         # AuthSystem + SessionSystem (secp256r1 + SessionManager)
    ├── game.rs         # TapSystem, ComboSystem, PowerUpSystem, RoundSystem
    └── test.rs         # Unit tests (19 tests)
```

---

## 📄 License

MIT OR Apache-2.0
