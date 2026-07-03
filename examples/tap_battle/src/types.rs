//! Data types for Tap Battle
//!
//! This module defines all ECS components and data structures needed for
//! the Tap Battle game, including passkey identity, tap tracking, power-ups,
//! round management, and player profiles.
//!
//! **Cougr-Core Integration**: Components follow the ECS pattern from
//! cougr-core, with passkey authentication via `secp256r1_auth` and
//! session management via `SessionBuilder`.

use soroban_sdk::{contracttype, Address, BytesN};

// ============================================================================
// ECS Components
// ============================================================================

/// Passkey identity component — stores a registered secp256r1 public key.
/// This is the on-chain representation of a player's WebAuthn/Passkey credential.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PasskeyIdentity {
    /// SEC-1 uncompressed public key (65 bytes: 0x04 || x || y)
    pub pubkey: BytesN<65>,
    /// Ledger sequence when the passkey was registered
    pub registered_at: u64,
}

/// Tap counter component — tracks a player's tapping activity within a round.
#[contracttype]
#[derive(Clone, Debug)]
pub struct TapCounter {
    /// Total tap count in the current round
    pub count: u32,
    /// Current combo streak length
    pub combo: u32,
    /// Current score multiplier (derived from combo)
    pub multiplier: u32,
    /// Ledger sequence of the last tap (for combo timing)
    pub last_tap_ledger: u64,
}

impl TapCounter {
    /// Create a new zeroed tap counter.
    pub fn new() -> Self {
        Self {
            count: 0,
            combo: 0,
            multiplier: 1,
            last_tap_ledger: 0,
        }
    }
}

impl Default for TapCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of power-ups available to players.
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum PowerUpKind {
    /// Double the value of each tap for the next N taps
    DoubleTap = 0,
    /// Shield from opponent power-up effects
    Shield = 1,
    /// Burst of bonus points
    Burst = 2,
}

/// Power-up component — represents a charged ability.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PowerUp {
    /// Number of available charges
    pub charges: u32,
    /// Type of power-up
    pub kind: u32,
}

impl PowerUp {
    /// Create a new power-up with zero charges.
    pub fn new(kind: u32) -> Self {
        Self { charges: 0, kind }
    }
}

/// Round state component — tracks a competitive match between two players.
#[contracttype]
#[derive(Clone, Debug)]
pub struct RoundState {
    /// Ledger sequence when the round started
    pub started_at: u64,
    /// Duration of the round in ledger sequences
    pub duration: u64,
    /// Player A's address
    pub player_a: Address,
    /// Player B's address
    pub player_b: Address,
    /// Player A's accumulated score
    pub player_a_score: u32,
    /// Player B's accumulated score
    pub player_b_score: u32,
    /// Whether the round has been finalized
    pub finished: bool,
}

/// Player profile component — persistent stats across rounds.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerProfile {
    /// Total number of rounds won
    pub total_wins: u32,
    /// Total number of taps across all rounds
    pub total_taps: u32,
    /// Best combo streak ever achieved
    pub best_combo: u32,
}

impl PlayerProfile {
    /// Create a new zeroed player profile.
    pub fn new() -> Self {
        Self {
            total_wins: 0,
            total_taps: 0,
            best_combo: 0,
        }
    }
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self::new()
    }
}

/// Result returned after each tap action.
#[contracttype]
#[derive(Clone, Debug)]
pub struct TapResult {
    /// Total tap count after this tap
    pub count: u32,
    /// Current combo streak
    pub combo: u32,
    /// Current multiplier
    pub multiplier: u32,
    /// Score earned from this tap
    pub score: u32,
}

// ============================================================================
// Storage Keys
// ============================================================================

/// Storage keys for Soroban persistent storage.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// PasskeyIdentity for a player (Address)
    Passkey(Address),
    /// TapCounter for a player (Address)
    TapState(Address),
    /// PowerUp for a player (Address)
    PowerUpState(Address),
    /// Current round state
    Round,
    /// PlayerProfile for a player (Address)
    Profile(Address),
    /// Flag indicating if a round is active
    RoundActive,
    /// Count of cougr-core entities (ECS integration)
    EntityCount,
}

// ============================================================================
// Game Constants
// ============================================================================

/// Number of ledger sequences within which taps maintain a combo streak.
pub const COMBO_WINDOW: u64 = 5;

/// Maximum multiplier achievable through combo streaks.
pub const MAX_MULTIPLIER: u32 = 10;

/// Number of combo taps required to earn one power-up charge.
pub const COMBO_CHARGE_THRESHOLD: u32 = 5;

/// Bonus points awarded by the DoubleTap power-up.
pub const DOUBLE_TAP_BONUS: u32 = 10;

/// Bonus points awarded by the Burst power-up.
pub const BURST_BONUS: u32 = 25;

/// Default session duration in ledger sequences.
pub const DEFAULT_SESSION_DURATION: u64 = 1000;

/// Default maximum operations per session.
pub const DEFAULT_MAX_OPS: u32 = 500;
