# Smart Accounts & Wallet Integration Research

> Deep research on natively implementing Smart Accounts for easy developer integration, while maintaining compatibility with standard wallets.

---

## Table of Contents

1. [Account Abstraction Fundamentals](#1-account-abstraction-fundamentals)
2. [OpenZeppelin: Implementation Reference](#2-openzeppelin-implementation-reference)
3. [Stellar/Soroban: Native Account Abstraction](#3-stellarsoroban-native-account-abstraction)
4. [Gaming-Specific Smart Account Patterns](#4-gaming-specific-smart-account-patterns)
5. [Dual Wallet Support Architecture](#5-dual-wallet-support-architecture)
6. [Integration Proposal for Cougr](#6-integration-proposal-for-cougr)
7. [References](#7-references)

---

## 1. Account Abstraction Fundamentals

### Traditional Accounts (EOAs) vs Smart Accounts

Traditional blockchain accounts, known as Externally Owned Accounts (EOAs), are controlled exclusively by a single private key. Every transaction requires the holder to sign it directly, creating a rigid and inflexible user experience. Smart Accounts fundamentally change this model by replacing the private key as the sole authority with programmable smart contract logic.

| Feature | EOA | Smart Account |
|---------|-----|---------------|
| **Control** | Single private key | Programmable logic |
| **Validation** | ECDSA signature only | Custom validation rules |
| **Batch Operations** | One tx at a time | Multiple ops per tx |
| **Gas Payment** | Must hold native token | Can be sponsored |
| **Recovery** | Seed phrase only | Social recovery, guardians |
| **Multi-device** | Share private key | Register multiple keys |
| **Upgradeability** | None | Modular, upgradeable |

### ERC-4337: The Standard

ERC-4337 is the dominant standard for Account Abstraction on Ethereum, achieving this without modifying consensus rules. It introduces a parallel infrastructure:

- **UserOperation**: A pseudo-transaction object containing sender, nonce, callData, gas parameters, and signatures. Instead of going to the standard mempool, UserOperations flow through an alternative mempool.
- **EntryPoint Contract**: A singleton contract deployed across networks that receives batches of UserOperations, validates them, and executes them. It serves as the single point of trust.
- **Bundlers**: Off-chain infrastructure operators that collect UserOperations from the alt-mempool, bundle them into standard transactions, and submit them to the EntryPoint.
- **Paymasters**: Optional contracts that can sponsor gas fees for users, enabling "gasless" experiences. A game server, for instance, can deploy a Paymaster that covers gas for gameplay transactions.
- **Factory Contracts**: Create new smart account instances deterministically, allowing counterfactual deployment (address known before deployment).

### EIP-7702: Bridging EOAs and Smart Accounts

Introduced with Ethereum's Pectra upgrade (May 7, 2025), EIP-7702 allows existing EOAs to temporarily execute smart contract code. This bridges the gap between traditional and smart accounts:

- EOAs can gain batch transaction capabilities
- Gas sponsorship becomes available for existing wallets
- No migration required from EOA to smart account
- Industry projections anticipated over 200 million smart accounts by late 2025

### Cross-Chain State of Account Abstraction

- **Ethereum**: Full ERC-4337 with production bundlers (Stackup, Gelato, Pimlico)
- **Polygon, Arbitrum, Optimism**: Full ERC-4337 support
- **zkSync**: Native account abstraction built into the protocol
- **Starknet**: Native AA via Cairo smart contracts (all accounts are smart accounts)
- **Stellar/Soroban**: Native custom accounts via `__check_auth` (different paradigm, detailed below)

---

## 2. OpenZeppelin: Implementation Reference

### OpenZeppelin Contracts v5.2: Account Abstraction Framework

OpenZeppelin released Contracts v5.2 with a comprehensive Account Abstraction framework. This release provides production-ready ERC-4337 primitives and a modular architecture for building smart accounts.

#### Core Account Architecture

The base `Account.sol` contract implements the `IAccount` interface and handles EntryPoint plumbing. Smart accounts built with OpenZeppelin inherit from:

- **`Account.sol`**: Core ERC-4337 functionality (`validateUserOp`)
- **`EIP712.sol`**: Typed data signatures for structured signing
- **`ERC7739.sol`**: Replay-attack-resistant typed signatures
- **`ERC7821.sol`**: Minimal batch executor interface
- **`ERC721Holder.sol` / `ERC1155Holder.sol`**: NFT receiving capabilities

The `PackedUserOperation` structure contains: sender, nonce, initCode, callData, accountGasLimits, preVerificationGas, gasFees, paymasterAndData, and signature fields.

#### ERC-7579: Modular Smart Accounts

ERC-7579 defines a standardized interface for modular smart accounts, enabling accounts to install, uninstall, and interact with modules that extend their capabilities in a composable manner. OpenZeppelin provides full support via two main contracts:

- **`AccountERC7579`**: Extension of Account that implements support for executor, validator, and fallback handler modules
- **`AccountERC7579Hooked`**: Extension that additionally supports hook modules

##### Module Type 1: Validators

Validators handle signature verification and UserOperation validation. They determine if a transaction should proceed based on the module's configured rules.

```
// Validators define the "who can authorize" logic
// Examples: ECDSA signer, multi-sig, WebAuthn, session keys
```

##### Module Type 2: Executors

Executors can execute transactions on behalf of smart accounts via callbacks. They extend account functionality by allowing external contracts to trigger authorized actions.

```
// Executors define the "what can be done" logic
// Examples: Social recovery, automated actions, scheduled transfers
```

##### Module Type 3: Fallback Handlers

Fallback handlers extend the fallback functionality, enabling accounts to support additional interfaces without modifying core account logic.

##### Module Type 4: Hooks

Hooks execute custom logic before and after transactions. They provide enforcement mechanisms and access control.

```
// Hooks define the "under what conditions" logic
// Examples: Spending limits, time restrictions, whitelist enforcement
```

> **Important**: Modules can implement multiple types simultaneously. For example, a module can combine executor functionality with hooks to enforce spending limits on recovery operations.

#### Execution Modes

The framework supports multiple execution patterns encoded as `bytes32` values:

| Call Type | ID | Description |
|-----------|-----|-------------|
| `CALLTYPE_SINGLE` | 0x00 | Single contract call |
| `CALLTYPE_BATCH` | 0x01 | Batch of multiple calls |
| `CALLTYPE_DELEGATECALL` | 0xFF | Delegate call execution |

| Execution Type | ID | Behavior |
|----------------|-----|----------|
| `EXECTYPE_DEFAULT` | 0x00 | Reverts on failure |
| `EXECTYPE_TRY` | 0x01 | Emits event on failure instead of reverting |

#### Ready-to-Use Composable Modules

OpenZeppelin Community Contracts provides pre-built modules:

- **`ERC7579Executor`**: Foundation for building executor modules
- **`ERC7579Validator`**: Base layer for validator implementations
- **`ERC7579Multisig`**: Multi-signature validation with configurable thresholds
- **`ERC7579MultisigWeighted`**: Assigns different weights to signers
- **`ERC7579MultisigConfirmation`**: Verification system for adding new signers
- **`ERC7579MultisigStorage`**: Enables presigned recovery operations
- **`ERC7579DelayedExecutor`**: Time-delayed execution with cancellation windows

#### Social Recovery Pattern

The documentation provides practical examples combining executor and multisig modules for guardian-based recovery:

1. **Basic Recovery**: `ERC7579Executor` + `ERC7579Multisig` for immediate guardian recovery
2. **Delayed Recovery**: `ERC7579DelayedExecutor` adds a cancellation window (e.g., 48h) before recovery completes, preventing hostile guardian attacks

#### ERC4337Utils

The `ERC4337Utils` library provides helper functions for manipulating UserOperation structs and ERC-4337 related values, simplifying the development of custom account logic.

---

## 3. Stellar/Soroban: Native Account Abstraction

### Soroban's Built-In Account Abstraction

Unlike Ethereum's retrofit approach (ERC-4337), Stellar/Soroban has account abstraction built directly into the protocol. This is achieved through two account types and a native authorization framework.

### Two Account Types

| Type | Address Format | Description |
|------|---------------|-------------|
| **Classic Stellar Account** | G address | Protocol-native, sequence-based, multi-sig built-in |
| **Contract Account** | C address | Turing-complete, custom logic, `__check_auth` |

### The `CustomAccountInterface`

Any contract that implements `CustomAccountInterface` and the `__check_auth` function becomes a **contract account**. When any other contract calls `require_auth` for the Address of this contract, the Soroban host automatically calls `__check_auth`.

```rust
impl CustomAccountInterface for AccountContract {
    type Signature = Vec<AccSignature>;
    type Error = AccError;

    fn __check_auth(
        env: Env,
        signature_payload: Hash<32>,  // Data that required signing
        signatures: Vec<AccSignature>, // Custom signature format
        auth_context: Vec<Context>,    // All invocations being authorized
    ) -> Result<(), AccError> {
        // Custom authentication and authorization logic
    }
}
```

**Key properties of `__check_auth`:**
- Called automatically by the Soroban host during `require_auth` verification
- Can mutate its own state without additional authorization (host guarantees it's only called during auth verification)
- Receives the full authorization context, allowing per-invocation policy decisions
- Must return `()` to approve or error/panic to deny

### Multisig with Tiered Policies

The official Stellar example demonstrates a two-tier authorization policy:

**Tier 1 - Strict Operations** (all signers required):
- Modifying the contract itself
- Contract creation operations
- Any operation outside defined spend limits

**Tier 2 - Spend-Limited Operations** (subset of signers):
- Token transfers within per-token spending caps
- When not all signers authorize, the contract tracks remaining spend allowances

```rust
// Per-signer and per-token spend tracking
enum DataKey {
    SignerCnt,
    Signer(BytesN<32>),
    SpendLimit(Address),  // Per-token spending caps
}
```

### WebAuthn/Passkeys Support

Since Protocol 21, Stellar supports the **secp256r1** signature scheme, enabling WebAuthn/Passkey integration:

- Users authenticate with fingerprint or Face ID instead of seed phrases
- Device's passkey public key registered on-chain in the contract account
- `__check_auth` verifies WebAuthn signatures against stored public keys
- **Meridian Pay** wallet: Production example with 1,000+ users using passkey authentication
- Client data JSON and challenge verification built into the auth flow

### Native Fee Sponsorship

Stellar provides native fee sponsorship at the protocol level - one account can pay for another's operations without any smart contract infrastructure. This is simpler than Ethereum's Paymaster pattern.

### Authorization Framework Advantages

Soroban's authorization framework provides built-in implementations for:
- Signature verification
- Replay attack prevention
- Auth tree traversal (nested contract calls)
- Automatic context propagation

The framework is built into the core protocol, meaning custom authentication can be implemented without protocol-level changes.

---

## 4. Gaming-Specific Smart Account Patterns

### Session Keys

Session keys are temporary signing credentials scoped to specific contracts, functions, gas limits, and time windows. They enable seamless gameplay without constant wallet pop-ups.

**Architecture:**
```
Player connects wallet (one-time)
    ↓
Game creates session key (scoped: contract X, functions [move, attack], 1 hour, max 100 ops)
    ↓
Game server uses session key for gameplay transactions
    ↓
Session expires → player must re-authorize
```

**Scoping parameters:**
- **Contract address**: Only interact with the game contract
- **Function whitelist**: Only call gameplay functions (not withdraw/transfer)
- **Time window**: Valid for the game session duration
- **Gas limit**: Cap total gas expenditure
- **Operation count**: Maximum number of transactions

**Stellar Implementation**: A custom account contract can implement session keys by:
1. Storing temporary public keys with scoping metadata
2. In `__check_auth`, checking if the signer is a session key
3. Validating the invocation falls within the session key's scope
4. Tracking usage counts and expiration

### Batched Transactions

Multiple game actions executed atomically in a single transaction:

```
Turn = [
    move_entity(player, position),
    attack(player, target),
    collect_loot(player, item_id),
    update_score(player, +100),
]
// All succeed or all fail
```

**Benefits for gaming:**
- Single verification overhead for multiple operations
- Atomic state transitions (no partial game states)
- Reduced gas costs (amortized over batch)
- Lower latency (one round-trip instead of N)

**Stellar Implementation**: Soroban naturally supports this - a single contract invocation can perform multiple state changes atomically.

### Social Recovery

Guardian-based account recovery for gaming accounts:

- **Guardians**: Trusted friends, family, or services hold recovery keys
- **M-of-N threshold**: e.g., 3 of 5 guardians required to approve recovery
- **Time-lock delay**: 3-5 day delay before recovery completes (prevents instant theft)
- **Cancellation window**: Original owner can cancel fraudulent recovery attempts

**Stellar Implementation**: A custom account contract stores guardian addresses and implements recovery logic in `__check_auth` with time-locked state transitions.

### Multi-Device Support

Players need seamless gaming across devices:

- **Device registration**: Each device gets an authorized key pair
- **Per-device policies**: Mobile might have lower spend limits than desktop
- **Revocation**: Remove lost device keys without affecting other devices
- **Sync**: Game state consistent across devices (same on-chain account)

**Stellar Implementation**: Store multiple authorized public keys in the contract account, each with device-specific metadata and policies.

### Gasless Gameplay

The game subsidizes transaction fees for core gameplay:

- **Core gameplay**: Game server pays fees (move, attack, collect)
- **Economic actions**: Player pays fees (trade, transfer, marketplace)
- **Onboarding**: First session free to reduce friction
- **Progressive fees**: Free tier → subsidized → full cost as player advances

**Stellar Implementation**: Native fee sponsorship means no Paymaster contract needed. The game server simply sponsors transactions for authorized players.

---

## 5. Dual Wallet Support Architecture

### Unified Interface Design

The core challenge is supporting both Classic Stellar accounts (G addresses) and Custom Contract accounts (C addresses) with a single developer API.

```
┌──────────────────────────────┐
│       Game Logic Layer       │
│   (ECS Systems & Components) │
└──────────────┬───────────────┘
               │
┌──────────────▼───────────────┐
│   Account Abstraction Layer   │
│   (Unified CougrAccount API)  │
└──────┬───────────────┬───────┘
       │               │
┌──────▼──────┐ ┌──────▼──────┐
│   Classic    │ │  Contract   │
│   Stellar    │ │  Account    │
│   Account    │ │  (C addr)   │
│   (G addr)   │ │             │
│              │ │ - Session   │
│ - Standard   │ │   keys      │
│   signing    │ │ - Batching  │
│ - Native     │ │ - Custom    │
│   multi-sig  │ │   auth      │
│ - Fee        │ │ - WebAuthn  │
│   sponsor    │ │ - Recovery  │
└─────────────┘ └─────────────┘
```

### Capability Detection

At connection time, detect what the connected account supports:

```rust
pub struct AccountCapabilities {
    pub can_batch: bool,           // Contract accounts: yes, Classic: limited
    pub can_sponsor_gas: bool,     // Both: yes (Stellar native)
    pub has_session_keys: bool,    // Contract accounts only
    pub has_social_recovery: bool, // Contract accounts only
    pub has_multi_device: bool,    // Contract accounts: yes, Classic: via multi-sig
    pub auth_methods: Vec<AuthMethod>, // ed25519, secp256r1, WebAuthn, etc.
}

pub enum AuthMethod {
    Ed25519,       // Classic Stellar default
    Secp256r1,     // WebAuthn/Passkeys
    Custom(String), // Contract-specific
}
```

### Graceful Degradation

When a feature isn't available for the connected account type, degrade gracefully:

| Feature | Contract Account | Classic Account | Fallback |
|---------|-----------------|-----------------|----------|
| Session keys | Native | Not available | Per-action signing |
| Batching | Full support | Single tx | Sequential calls |
| Gas sponsorship | Via contract | Native | Player pays |
| Social recovery | Custom guardians | Not available | Seed phrase |
| Multi-device | Custom keys | Multi-sig signers | Share key |
| WebAuthn | Full support | Since Protocol 21 | Standard signing |

### Best Practices

1. **Use abstract interfaces**: Game code never interacts with account type directly
2. **Feature flags**: Runtime detection of account capabilities
3. **Consistent UX**: Same game experience regardless of account type
4. **Progressive enhancement**: Smart features available when account supports them
5. **Test both paths**: Every flow tested with both account types

---

## 6. Integration Proposal for Cougr

All account abstraction functionality lives as an internal module (`src/accounts/`) within the single `cougr-core` crate. This is a **major expansion** of the crate - adding a full account layer with session management, authorization policies, capability detection, social recovery, and multi-device support. The developer experience stays simple: one dependency (`cougr-core`), one import path (`cougr_core::accounts::*`), no workspace or multi-crate complexity.

### Module Architecture: `src/accounts/`

```
src/accounts/
├── mod.rs                  // Module entry point, re-exports all public API
├── traits.rs               // CougrAccount, SessionKeyProvider, RecoveryProvider traits
├── types.rs                // AccountCapabilities, AuthMethod, GameAction, PlayerId
├── error.rs                // AccountError variants (Unauthorized, SessionExpired, etc.)
├── classic_account.rs      // ClassicStellarAccount - G address implementation
├── contract_account.rs     // ContractStellarAccount - C address implementation
├── session/
│   ├── mod.rs              // Session key orchestration
│   ├── key.rs              // SessionKey type with scoping metadata
│   ├── scope.rs            // SessionScope (allowed_systems, time, ops, gas)
│   ├── storage.rs          // On-chain session key storage and lookup
│   └── validation.rs       // Session key validation against auth_context
├── batch/
│   ├── mod.rs              // Batch transaction orchestration
│   ├── builder.rs          // BatchBuilder for composing multiple game actions
│   └── executor.rs         // Atomic batch execution with rollback
├── recovery/
│   ├── mod.rs              // Social recovery orchestration
│   ├── guardian.rs          // Guardian registration and management
│   ├── timelock.rs         // Time-locked recovery with cancellation windows
│   └── policy.rs           // M-of-N threshold policies
├── multi_device/
│   ├── mod.rs              // Multi-device key management
│   ├── device_key.rs       // Per-device key registration and policies
│   └── revocation.rs       // Device key revocation without affecting others
├── capability.rs           // Runtime capability detection for connected accounts
├── degradation.rs          // Graceful degradation logic (contract → classic fallbacks)
└── testing.rs              // MockAccount, MockSession for unit testing without blockchain
```

This adds **~20 new source files** to `cougr-core`, representing a complete account abstraction layer purpose-built for on-chain gaming.

### Core Traits

```rust
/// Core account trait that abstracts over Classic and Contract accounts.
/// This is the primary interface that game code interacts with.
pub trait CougrAccount {
    /// Get the account address
    fn address(&self) -> Address;

    /// Check account capabilities at runtime
    fn capabilities(&self) -> AccountCapabilities;

    /// Authorize a single game action
    fn authorize(&self, env: &Env, action: &GameAction) -> Result<(), AccountError>;

    /// Batch multiple game actions into a single atomic authorization
    fn batch_authorize(&self, env: &Env, actions: &[GameAction]) -> Result<(), AccountError>;
}

/// Session key management for contract accounts.
/// Classic accounts gracefully degrade to per-action signing.
pub trait SessionKeyProvider: CougrAccount {
    /// Create a new session key scoped to specific systems, time, and gas
    fn create_session(&self, env: &Env, scope: SessionScope) -> Result<SessionKey, AccountError>;

    /// Validate a session key is still active and within scope
    fn validate_session(&self, env: &Env, key: &SessionKey) -> Result<bool, AccountError>;

    /// Revoke a session key immediately
    fn revoke_session(&self, env: &Env, key: &SessionKey) -> Result<(), AccountError>;

    /// List all active session keys for this account
    fn active_sessions(&self, env: &Env) -> Result<Vec<SessionKey>, AccountError>;
}

/// Social recovery for contract accounts.
pub trait RecoveryProvider: CougrAccount {
    /// Register a guardian for account recovery
    fn add_guardian(&self, env: &Env, guardian: Address) -> Result<(), AccountError>;

    /// Initiate recovery (starts time-lock period)
    fn initiate_recovery(
        &self, env: &Env, new_owner: Address, guardian_signatures: Vec<Signature>,
    ) -> Result<RecoveryRequest, AccountError>;

    /// Cancel a pending recovery (by current owner within cancellation window)
    fn cancel_recovery(&self, env: &Env, request_id: u64) -> Result<(), AccountError>;

    /// Finalize recovery after time-lock expires
    fn finalize_recovery(&self, env: &Env, request_id: u64) -> Result<(), AccountError>;
}
```

### Session Key Implementation

```rust
/// Session scope defines exactly what a session key can do
pub struct SessionScope {
    pub allowed_systems: Vec<Symbol>,  // Which game systems can be called
    pub allowed_actions: Vec<Symbol>,  // Which specific actions within those systems
    pub max_operations: u32,           // Maximum total operations
    pub expires_at: u64,               // Timestamp expiration
    pub max_gas: u64,                  // Gas spending cap for the session
}

/// A session key with full tracking metadata
pub struct SessionKey {
    pub public_key: BytesN<32>,        // The temporary signing key
    pub scope: SessionScope,           // What this key can do
    pub created_at: u64,               // When the session started
    pub operations_used: u32,          // How many operations consumed
    pub gas_used: u64,                 // How much gas consumed
}

/// Builder pattern for session creation
pub struct SessionBuilder {
    // ...
}

impl SessionBuilder {
    pub fn new() -> Self;
    pub fn allow_system(mut self, system: Symbol) -> Self;
    pub fn allow_action(mut self, action: Symbol) -> Self;
    pub fn max_ops(mut self, count: u32) -> Self;
    pub fn expires_in(mut self, seconds: u64) -> Self;
    pub fn max_gas(mut self, gas: u64) -> Self;
    pub fn build(self, env: &Env, account: &impl CougrAccount) -> Result<SessionKey, AccountError>;
}
```

### Batch Transaction Builder

```rust
/// Compose multiple game actions into one atomic transaction
pub struct BatchBuilder {
    actions: Vec<GameAction>,
}

impl BatchBuilder {
    pub fn new() -> Self;

    /// Add a system call to the batch
    pub fn add<S: System>(mut self, system: Symbol, input: S::In) -> Self;

    /// Execute all actions atomically - all succeed or all revert
    pub fn execute(
        self, world: &mut World, account: &impl CougrAccount,
    ) -> Result<Vec<SystemResult>, AccountError>;
}

// Usage in a game:
BatchBuilder::new()
    .add::<MoveSystem>(symbol_short!("move"), MoveInput { x: 5, y: 3 })
    .add::<AttackSystem>(symbol_short!("attack"), AttackInput { target: enemy_id })
    .add::<CollectSystem>(symbol_short!("collect"), CollectInput { item: loot_id })
    .execute(&mut world, &player_account)?;
```

### Integration with ECS World

```rust
/// Account-aware World that tracks player accounts and enforces authorization.
/// These methods extend the existing World - no separate type needed.
impl World {
    /// Register a player account and create their player entity
    pub fn register_player(&mut self, account: impl CougrAccount) -> PlayerId;

    /// Get a registered player's account capabilities
    pub fn player_capabilities(&self, player: PlayerId) -> AccountCapabilities;

    /// Execute a system call with account authorization
    pub fn execute_authorized<S: System>(
        &mut self,
        system: &mut S,
        account: &impl CougrAccount,
        input: S::In,
    ) -> Result<S::Out, AccountError>;

    /// Batch execute multiple system calls atomically
    pub fn batch_execute(
        &mut self,
        account: &impl CougrAccount,
        actions: Vec<SystemCall>,
    ) -> Result<Vec<SystemResult>, AccountError>;

    /// Start a game session with a session key
    pub fn start_session(
        &mut self,
        account: &impl SessionKeyProvider,
        scope: SessionScope,
    ) -> Result<SessionKey, AccountError>;

    /// Execute using an active session key (no wallet popup)
    pub fn execute_with_session<S: System>(
        &mut self,
        system: &mut S,
        session: &SessionKey,
        input: S::In,
    ) -> Result<S::Out, AccountError>;
}
```

### Account Implementations

```rust
/// Classic Stellar account (G address) - works out of the box
pub struct ClassicStellarAccount {
    address: Address,
}

impl CougrAccount for ClassicStellarAccount {
    fn capabilities(&self) -> AccountCapabilities {
        AccountCapabilities {
            can_batch: false,           // Limited to single tx
            can_sponsor_gas: true,      // Native Stellar feature
            has_session_keys: false,    // Not supported
            has_social_recovery: false, // Not supported
            has_multi_device: true,     // Via native multi-sig
            auth_methods: vec![AuthMethod::Ed25519],
        }
    }
    // ...
}

/// Contract account (C address) - full smart account features
pub struct ContractStellarAccount {
    address: Address,
    // Internal state for session keys, guardians, device keys
}

impl CougrAccount for ContractStellarAccount { /* full capabilities */ }
impl SessionKeyProvider for ContractStellarAccount { /* session management */ }
impl RecoveryProvider for ContractStellarAccount { /* social recovery */ }
```

### Testing Utilities

```rust
/// Mock account for unit testing without Stellar/Soroban
pub struct MockAccount {
    pub address: Address,
    pub capabilities: AccountCapabilities,
    pub should_authorize: bool,  // Control auth behavior in tests
}

/// Mock session for testing session key flows
pub struct MockSession {
    pub key: SessionKey,
    pub is_valid: bool,
}

// Usage in tests:
#[test]
fn test_game_with_session() {
    let account = MockAccount::new_contract_account();
    let session = account.create_mock_session(SessionScope { ... });
    world.execute_with_session(&mut move_system, &session, input).unwrap();
}
```

### Developer Experience Goals

1. **Single crate, single dependency**: `cougr-core` includes everything - no extra crates to add
2. **Zero-config for basic usage**: Classic accounts work out of the box with zero account code
3. **Opt-in smart features**: Session keys, batching, recovery available when using contract accounts
4. **Single API**: Developers write game logic once, works with all account types
5. **Graceful degradation**: Smart features silently fall back to simpler alternatives for classic accounts
6. **Type-safe**: Compile-time errors for invalid configurations
7. **Testable**: `MockAccount` and `MockSession` for unit testing without blockchain
8. **Builder patterns**: `SessionBuilder` and `BatchBuilder` for ergonomic, readable code

---

## 7. References

- [OpenZeppelin Account Abstraction Documentation](https://docs.openzeppelin.com/contracts/5.x/account-abstraction)
- [OpenZeppelin ERC-7579 Account Modules](https://docs.openzeppelin.com/community-contracts/account-modules)
- [OpenZeppelin Contracts v5.2 Release](https://www.openzeppelin.com/news/introducing-openzeppelin-contracts-5.2-and-openzeppelin-community-contracts)
- [OpenZeppelin WebAuthn Smart Accounts Guide](https://docs.openzeppelin.com/contracts/5.x/learn/webauthn-smart-accounts)
- [ERC-4337 Documentation](https://docs.erc4337.io/index.html)
- [ERC-7579: Minimal Modular Smart Accounts](https://eips.ethereum.org/EIPS/eip-7579)
- [Stellar Smart Contract Authorization](https://developers.stellar.org/docs/learn/encyclopedia/security/authorization)
- [Stellar Custom Account Example](https://developers.stellar.org/docs/build/smart-contracts/example-contracts/custom-account)
- [Stellar Auth Blog Post](https://stellar.org/blog/developers/auth-entication-and-orization-in-blockchain)
- [WebAuthn Smart Wallet Discussion (Stellar)](https://github.com/orgs/stellar/discussions/1499)
- [Meridian Pay Passkey Wallet](https://blockchain.news/news/stellar-meridian-pay-smart-wallet-passkey-authentication)
- [Using __check_auth in Interesting Ways](https://developers.stellar.org/docs/build/guides/conventions/check-auth-tutorials)
- [Passkeys on Stellar (Leigh McCulloch)](https://leighmcculloch.com/talks/experimenting-with-webauthn-passkeys-on-stellar/)
- [CAP-0046-11: Smart Contract Authorization](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0046-11.md)
