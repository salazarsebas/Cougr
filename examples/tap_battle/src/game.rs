//! Game logic systems for Tap Battle.
//!
//! Implements the core gameplay systems following cougr-core's ECS pattern:
//! - **TapSystem**: Increments tap counter, calculates combos
//! - **ComboSystem**: Applies multiplier based on combo streak
//! - **PowerUpSystem**: Charges and activates power-ups
//! - **RoundSystem**: Manages round timing and winner declaration

use soroban_sdk::{Address, Env};

use crate::types::*;

// ============================================================================
// TapSystem - Core tapping mechanic
// ============================================================================

/// Process a tap action for a player.
///
/// Increments the tap counter, calculates combo streak (taps within
/// `COMBO_WINDOW` ledgers maintain the streak), applies multiplier,
/// and updates the player's score in the active round.
///
/// This function is optimized to be as lightweight as possible for
/// mobile-first gameplay (minimal on-chain state reads per tap).
pub fn process_tap(env: &Env, player: &Address) -> TapResult {
    // Load tap state
    let mut tap_state: TapCounter = env
        .storage()
        .persistent()
        .get(&DataKey::TapState(player.clone()))
        .unwrap_or_default();

    let current_ledger = env.ledger().sequence() as u64;

    // === COMBO SYSTEM ===
    // Check if this tap is within the combo window
    // Use count > 0 to detect non-first taps (ledger sequence can be 0)
    if tap_state.count > 0 && current_ledger.wrapping_sub(tap_state.last_tap_ledger) <= COMBO_WINDOW
    {
        // Combo continues
        tap_state.combo += 1;
    } else {
        // Combo breaks - reset
        tap_state.combo = 1;
    }

    // Update multiplier (capped at MAX_MULTIPLIER)
    tap_state.multiplier = if tap_state.combo > MAX_MULTIPLIER {
        MAX_MULTIPLIER
    } else {
        tap_state.combo
    };

    // Increment tap count
    tap_state.count += 1;
    tap_state.last_tap_ledger = current_ledger;

    // Calculate score for this tap
    let tap_score = tap_state.multiplier;

    // === POWER-UP CHARGING ===
    // Every COMBO_CHARGE_THRESHOLD combo taps, earn a power-up charge
    if tap_state.combo > 0 && tap_state.combo.is_multiple_of(COMBO_CHARGE_THRESHOLD) {
        let mut power_up: PowerUp = env
            .storage()
            .persistent()
            .get(&DataKey::PowerUpState(player.clone()))
            .unwrap_or_else(|| PowerUp::new(PowerUpKind::DoubleTap as u32));
        power_up.charges += 1;
        env.storage()
            .persistent()
            .set(&DataKey::PowerUpState(player.clone()), &power_up);
    }

    // Save tap state
    env.storage()
        .persistent()
        .set(&DataKey::TapState(player.clone()), &tap_state);

    // Update round score if a round is active
    update_round_score(env, player, tap_score);

    // Update persistent profile stats
    update_profile_taps(env, player, tap_state.combo);

    TapResult {
        count: tap_state.count,
        combo: tap_state.combo,
        multiplier: tap_state.multiplier,
        score: tap_score,
    }
}

// ============================================================================
// PowerUpSystem - Power-up activation
// ============================================================================

/// Activate a power-up for a player.
///
/// Consumes one charge and applies the effect:
/// - `DoubleTap (0)`: Adds DOUBLE_TAP_BONUS points
/// - `Shield (1)`: Defensive effect (currently no-op)
/// - `Burst (2)`: Adds BURST_BONUS points
///
/// # Panics
/// Panics if no charges are available.
pub fn activate_power_up(env: &Env, player: &Address, power_up_kind: u32) {
    let mut power_up: PowerUp = env
        .storage()
        .persistent()
        .get(&DataKey::PowerUpState(player.clone()))
        .expect("no power-up state");

    assert!(power_up.charges > 0, "no power-up charges available");

    // Consume a charge
    power_up.charges -= 1;

    // Apply power-up effect
    let bonus = match power_up_kind {
        0 => DOUBLE_TAP_BONUS, // DoubleTap
        1 => 0,                // Shield (defensive, no score bonus)
        2 => BURST_BONUS,      // Burst
        _ => panic!("unknown power-up kind"),
    };

    // Update power-up kind to requested type
    power_up.kind = power_up_kind;
    env.storage()
        .persistent()
        .set(&DataKey::PowerUpState(player.clone()), &power_up);

    // Apply bonus to round score
    if bonus > 0 {
        update_round_score(env, player, bonus);
    }
}

// ============================================================================
// RoundSystem - Match management
// ============================================================================

/// Start a new competitive round between two players.
///
/// Both players must have registered passkeys. Round duration is measured
/// in ledger sequences (not wall-clock time).
///
/// # Panics
/// Panics if either player has no registered passkey or if a round is active.
pub fn start_round(env: &Env, player_a: &Address, player_b: &Address, duration: u64) {
    // Verify both players have registered passkeys
    assert!(
        env.storage()
            .persistent()
            .has(&DataKey::Passkey(player_a.clone())),
        "player A has no registered passkey"
    );
    assert!(
        env.storage()
            .persistent()
            .has(&DataKey::Passkey(player_b.clone())),
        "player B has no registered passkey"
    );

    // Check no round is currently active
    let round_active: bool = env
        .storage()
        .instance()
        .get(&DataKey::RoundActive)
        .unwrap_or(false);
    assert!(!round_active, "a round is already active");

    // Initialize round state
    let round = RoundState {
        started_at: env.ledger().sequence() as u64,
        duration,
        player_a: player_a.clone(),
        player_b: player_b.clone(),
        player_a_score: 0,
        player_b_score: 0,
        finished: false,
    };

    env.storage().instance().set(&DataKey::Round, &round);
    env.storage().instance().set(&DataKey::RoundActive, &true);

    // Reset tap counters for both players
    env.storage()
        .persistent()
        .set(&DataKey::TapState(player_a.clone()), &TapCounter::new());
    env.storage()
        .persistent()
        .set(&DataKey::TapState(player_b.clone()), &TapCounter::new());

    // Reset power-ups for both players
    env.storage().persistent().set(
        &DataKey::PowerUpState(player_a.clone()),
        &PowerUp::new(PowerUpKind::DoubleTap as u32),
    );
    env.storage().persistent().set(
        &DataKey::PowerUpState(player_b.clone()),
        &PowerUp::new(PowerUpKind::DoubleTap as u32),
    );
}

/// Get the current round state, auto-finalizing if duration has elapsed.
///
/// If the round duration has expired, this function declares the winner,
/// updates persistent profiles, and marks the round as finished.
pub fn get_round(env: &Env) -> RoundState {
    let mut round: RoundState = env
        .storage()
        .instance()
        .get(&DataKey::Round)
        .expect("no active round");

    // Check if round should be finalized
    if !round.finished {
        let current_ledger = env.ledger().sequence() as u64;
        if current_ledger >= round.started_at + round.duration {
            // Round is over - finalize
            round.finished = true;
            env.storage().instance().set(&DataKey::Round, &round);
            env.storage().instance().set(&DataKey::RoundActive, &false);

            // Determine winner and update profiles
            if round.player_a_score > round.player_b_score {
                increment_wins(env, &round.player_a);
            } else if round.player_b_score > round.player_a_score {
                increment_wins(env, &round.player_b);
            }
            // Tie: no winner recorded
        }
    }

    round
}

/// Get a player's profile, creating a default one if it doesn't exist.
pub fn get_profile(env: &Env, player: &Address) -> PlayerProfile {
    env.storage()
        .persistent()
        .get(&DataKey::Profile(player.clone()))
        .unwrap_or_default()
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Update the active round's score for a player.
fn update_round_score(env: &Env, player: &Address, points: u32) {
    let round_active: bool = env
        .storage()
        .instance()
        .get(&DataKey::RoundActive)
        .unwrap_or(false);

    if !round_active {
        return;
    }

    let mut round: RoundState = env
        .storage()
        .instance()
        .get(&DataKey::Round)
        .expect("round state missing");

    if round.finished {
        return;
    }

    if player == &round.player_a {
        round.player_a_score += points;
    } else if player == &round.player_b {
        round.player_b_score += points;
    }

    env.storage().instance().set(&DataKey::Round, &round);
}

/// Update persistent profile with tap stats.
fn update_profile_taps(env: &Env, player: &Address, current_combo: u32) {
    let mut profile = get_profile(env, player);
    profile.total_taps += 1;
    if current_combo > profile.best_combo {
        profile.best_combo = current_combo;
    }
    env.storage()
        .persistent()
        .set(&DataKey::Profile(player.clone()), &profile);
}

/// Increment win count for a player.
fn increment_wins(env: &Env, player: &Address) {
    let mut profile = get_profile(env, player);
    profile.total_wins += 1;
    env.storage()
        .persistent()
        .set(&DataKey::Profile(player.clone()), &profile);
}
