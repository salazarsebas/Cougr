# 🛡️ Session Arena — Canonical Session UX Example

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/salazarsebas/Cougr)
[![Tests](https://img.shields.io/badge/tests-4%20passing-brightgreen)](https://github.com/salazarsebas/Cougr)
[![Stellar](https://img.shields.io/badge/Stellar-Testnet-blue)](https://stellar.org)

`session_arena` demonstrates the canonical `cougr-core` 1.1.0 session lifecycle using `session::SessionManager`. It serves as the minimal reference example for developers integrating gasless, scoped gameplay sessions on Stellar.

For an advanced integration combining `SessionManager` with **secp256r1 passkey authentication** (WebAuthn/Biometrics), see the [tap_battle](../tap_battle/README.md) example.

---

## 🔄 Canonical Session Lifecycle

The `SessionManager` API establishes a beta-standard pattern for seamless on-chain gaming:

```
┌─────────────────────────────────────────────────────────┐
│ 1. APPROVAL (approve_session)                           │
│    Player signs once via wallet to authorize scope       │
│    → SessionManager::approve creates session key         │
│    → ActiveSession component returned for UI tracking   │
├─────────────────────────────────────────────────────────┤
│ 2. EXECUTION (tap)                                      │
│    Frictionless gameplay without per-tx wallet prompts    │
│    → SessionManager::execute_action validates intent      │
│    → Enforces operation budget, expiry, and nonce        │
├─────────────────────────────────────────────────────────┤
│ 3. RENEWAL (renew_session)                              │
│    UI prompts player to renew before session expires     │
│    → SessionManager::status reports health/renewal needs │
│    → SessionManager::renew extends expiry window         │
├─────────────────────────────────────────────────────────┤
│ 4. FALLBACK (fallback_tap)                              │
│    Robust transaction execution during expiry transition  │
│    → SessionManager::fallback_execute tries session first│
│    → Automatically falls back to direct auth if expired  │
└─────────────────────────────────────────────────────────┘
```

---

## 📖 Contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `approve_session` | `owner`, `max_taps`, `expires_in` | `ActiveSession` | One-time owner approval |
| `tap` | `owner`, `key_id` | `u32` | Gameplay action via active session |
| `renew_session` | `owner`, `key_id`, `expires_in` | `ActiveSession` | Extend session lifetime |
| `fallback_tap` | `owner`, `key_id` | `u32` | Session tap with direct auth fallback |
| `score` | `owner` | `u32` | Get current score |

---

## 🏗️ Quick Start

### Build & Test

```bash
# Build WASM
cargo build --target wasm32v1-none --release

# Run Tests
cargo test
```

**Test Results**: 4 tests passing ✅

| Test | Description |
|---|---|
| `approve_and_tap_without_reauth` | Verifies session approval and gasless execution |
| `renew_session_extends_play_window` | Verifies session renewal extends expiry |
| `fallback_tap_uses_direct_auth_after_session_expires` | Verifies fallback to direct auth on expiry |
| `mock_session_helper_matches_manager_flow` | Verifies test harness compatibility |

---

## 📄 License

MIT
