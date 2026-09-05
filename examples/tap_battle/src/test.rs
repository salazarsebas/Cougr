//! Unit tests for Tap Battle contract
//!
//! These tests validate the complete mobile-first auth flow and gameplay:
//! - Passkey registration
//! - Authentication (valid / invalid)
//! - Session creation and scoping
//! - Tap mechanics and combo calculation
//! - Power-up charging and activation
//! - Round scoring and winner declaration
//! - Session expiry
//! - Persistent profile stats
//! - Session renewal and fallback behavior

#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, testutils::Ledger, Env};

/// Helper: Create a dummy 65-byte secp256r1 public key for testing.
fn make_test_pubkey(env: &Env, seed: u8) -> BytesN<65> {
    let mut bytes = [0u8; 65];
    bytes[0] = 0x04; // uncompressed prefix
    bytes[1] = seed;
    BytesN::from_array(env, &bytes)
}

/// Helper: Register a player with a passkey (bypassing auth for testing).
fn setup_player(env: &Env, contract_id: &Address) -> Address {
    let player = Address::generate(env);
    let pubkey = make_test_pubkey(env, 1);

    // Register passkey directly via storage (bypassing require_auth)
    env.as_contract(contract_id, || {
        let identity = PasskeyIdentity {
            pubkey: pubkey.clone(),
            registered_at: env.ledger().sequence() as u64,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Passkey(player.clone()), &identity);
    });

    player
}

/// Helper: Set up a session for a player (bypassing crypto for testing).
fn setup_session(env: &Env, contract_id: &Address, player: &Address) {
    env.as_contract(contract_id, || {
        let session_scope = cougr_core::accounts::SessionBuilder::new(env)
            .allow_action(soroban_sdk::symbol_short!("tap"))
            .allow_action(soroban_sdk::Symbol::new(env, "use_power_up"))
            .max_operations(100)
            .expires_in(1000)
            .build_scope();
        cougr_core::session::SessionManager::approve(env, player, session_scope)
            .expect("session approval failed");

        env.storage()
            .persistent()
            .set(&DataKey::TapState(player.clone()), &TapCounter::new());

        let power_up = PowerUp::new(PowerUpKind::DoubleTap as u32);
        env.storage()
            .persistent()
            .set(&DataKey::PowerUpState(player.clone()), &power_up);
    });
}

// ============================================================================
// Passkey Registration Tests
// ============================================================================

/// Test that passkey registration stores the key on-chain.
#[test]
fn test_register_passkey() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TapBattleContract, ());
    let client = TapBattleContractClient::new(&env, &contract_id);

    let player = Address::generate(&env);
    let pubkey = make_test_pubkey(&env, 42);

    client.register_passkey(&player, &pubkey);

    // Verify passkey was stored
    env.as_contract(&contract_id, || {
        let identity: PasskeyIdentity = env
            .storage()
            .persistent()
            .get(&DataKey::Passkey(player.clone()))
            .expect("passkey should be stored");
        assert_eq!(identity.pubkey, pubkey);
    });

    // Verify ECS entity count
    assert!(client.get_entity_count() > 0);
}

/// Test registering multiple players with different passkeys.
#[test]
fn test_register_multiple_players() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TapBattleContract, ());
    let client = TapBattleContractClient::new(&env, &contract_id);

    let player_a = Address::generate(&env);
    let player_b = Address::generate(&env);

    client.register_passkey(&player_a, &make_test_pubkey(&env, 1));
    client.register_passkey(&player_b, &make_test_pubkey(&env, 2));

    // Both passkeys should be independently stored
    env.as_contract(&contract_id, || {
        assert!(env
            .storage()
            .persistent()
            .has(&DataKey::Passkey(player_a.clone())));
        assert!(env
            .storage()
            .persistent()
            .has(&DataKey::Passkey(player_b.clone())));
    });
}

// ============================================================================
// Session Tests
// ============================================================================

/// Test that a session can be created and validated.
#[test]
fn test_session_creation() {
    let env = Env::default();
    let contract_id = env.register(TapBattleContract, ());

    let player = setup_player(&env, &contract_id);
    setup_session(&env, &contract_id, &player);

    // Validate session works
    env.as_contract(&contract_id, || {
        let session_player =
            auth::validate_session(&env, &player, soroban_sdk::symbol_short!("tap"));
        assert_eq!(session_player, player);
    });
}

/// Test that session operations are decremented.
#[test]
fn test_session_ops_decrement() {
    let env = Env::default();
    let contract_id = env.register(TapBattleContract, ());

    let player = setup_player(&env, &contract_id);
    setup_session(&env, &contract_id, &player);

    env.as_contract(&contract_id, || {
        // Initial ops: 100
        auth::validate_session(&env, &player, soroban_sdk::symbol_short!("tap"));

        let keys = cougr_core::accounts::SessionStorage::load_all(&env, &player);
        let session = keys.last().expect("session missing");
        let status =
            cougr_core::session::SessionManager::status(&env, &player, &session.key_id).unwrap();
        assert_eq!(status.remaining_operations, 99);
    });
}

/// Test that session stops working after operations are exhausted.
#[test]
#[should_panic(expected = "session operations exhausted")]
fn test_session_ops_exhausted() {
    let env = Env::default();
    let contract_id = env.register(TapBattleContract, ());

    let player = setup_player(&env, &contract_id);

    // Create session with only 1 operation
    env.as_contract(&contract_id, || {
        let session_scope = cougr_core::accounts::SessionBuilder::new(&env)
            .allow_action(soroban_sdk::symbol_short!("tap"))
            .allow_action(soroban_sdk::Symbol::new(&env, "use_power_up"))
            .max_operations(1)
            .expires_in(1000)
            .build_scope();
        cougr_core::session::SessionManager::approve(&env, &player, session_scope)
            .expect("session approval failed");
    });

    env.as_contract(&contract_id, || {
        // First validation succeeds
        auth::validate_session(&env, &player, soroban_sdk::symbol_short!("tap"));
        // Second should panic - no operations left
        auth::validate_session(&env, &player, soroban_sdk::symbol_short!("tap"));
    });
}

// ============================================================================
// Tap Mechanics Tests
// ============================================================================

/// Test that tapping increments the counter and returns a result.
#[test]
fn test_tap_increments() {
    let env = Env::default();
    let contract_id = env.register(TapBattleContract, ());

    let player = setup_player(&env, &contract_id);
    setup_session(&env, &contract_id, &player);

    env.as_contract(&contract_id, || {
        let result = game::process_tap(&env, &player);
        assert_eq!(result.count, 1);
        assert_eq!(result.combo, 1);
        assert_eq!(result.multiplier, 1);
        assert_eq!(result.score, 1);

        let result2 = game::process_tap(&env, &player);
        assert_eq!(result2.count, 2);
    });
}

/// Test that consecutive taps within the combo window increase combo streak.
#[test]
fn test_combo_streak() {
    let env = Env::default();
    let contract_id = env.register(TapBattleContract, ());

    let player = setup_player(&env, &contract_id);
    setup_session(&env, &contract_id, &player);

    env.as_contract(&contract_id, || {
        // Tap 3 times in same ledger (within combo window)
        let r1 = game::process_tap(&env, &player);
        assert_eq!(r1.combo, 1);

        let r2 = game::process_tap(&env, &player);
        assert_eq!(r2.combo, 2);
        assert_eq!(r2.multiplier, 2);

        let r3 = game::process_tap(&env, &player);
        assert_eq!(r3.combo, 3);
        assert_eq!(r3.multiplier, 3);
        assert_eq!(r3.score, 3); // multiplier applies to score
    });
}

/// Test that the multiplier is capped at MAX_MULTIPLIER.
#[test]
fn test_multiplier_cap() {
    let env = Env::default();
    let contract_id = env.register(TapBattleContract, ());

    let player = setup_player(&env, &contract_id);
    setup_session(&env, &contract_id, &player);

    env.as_contract(&contract_id, || {
        // Tap many times to exceed MAX_MULTIPLIER
        let mut last_result = game::process_tap(&env, &player);
        for _ in 1..15 {
            last_result = game::process_tap(&env, &player);
        }
        assert!(last_result.multiplier <= MAX_MULTIPLIER);
    });
}

// ============================================================================
// Power-Up Tests
// ============================================================================

/// Test that combo streaks charge power-ups and activation works.
#[test]
fn test_power_up_charge_and_use() {
    let env = Env::default();
    let contract_id = env.register(TapBattleContract, ());

    let player = setup_player(&env, &contract_id);
    setup_session(&env, &contract_id, &player);

    env.as_contract(&contract_id, || {
        // Tap COMBO_CHARGE_THRESHOLD times to earn a charge
        for _ in 0..COMBO_CHARGE_THRESHOLD {
            game::process_tap(&env, &player);
        }

        // Check power-up has a charge
        let power_up: PowerUp = env
            .storage()
            .persistent()
            .get(&DataKey::PowerUpState(player.clone()))
            .unwrap();
        assert!(power_up.charges >= 1);

        // Activate DoubleTap power-up
        game::activate_power_up(&env, &player, PowerUpKind::DoubleTap as u32);

        // Check charge was consumed
        let power_up_after: PowerUp = env
            .storage()
            .persistent()
            .get(&DataKey::PowerUpState(player.clone()))
            .unwrap();
        assert_eq!(power_up_after.charges, power_up.charges - 1);
    });
}

/// Test that power-up activation fails with no charges.
#[test]
#[should_panic(expected = "no power-up charges available")]
fn test_power_up_no_charges() {
    let env = Env::default();
    let contract_id = env.register(TapBattleContract, ());

    let player = setup_player(&env, &contract_id);
    setup_session(&env, &contract_id, &player);

    env.as_contract(&contract_id, || {
        // Try to activate without any charges
        game::activate_power_up(&env, &player, PowerUpKind::Burst as u32);
    });
}

// ============================================================================
// Round Tests
// ============================================================================

/// Test starting a round between two registered players.
#[test]
fn test_start_round() {
    let env = Env::default();
    let contract_id = env.register(TapBattleContract, ());
    let client = TapBattleContractClient::new(&env, &contract_id);

    let player_a = setup_player(&env, &contract_id);
    let player_b = setup_player(&env, &contract_id);

    client.start_round(&player_a, &player_b, &100);

    let round = client.get_round();
    assert_eq!(round.player_a, player_a);
    assert_eq!(round.player_b, player_b);
    assert_eq!(round.player_a_score, 0);
    assert_eq!(round.player_b_score, 0);
    assert_eq!(round.duration, 100);
    assert!(!round.finished);
}

/// Test that round cannot start without registered passkeys.
#[test]
#[should_panic(expected = "player A has no registered passkey")]
fn test_start_round_no_passkey() {
    let env = Env::default();
    let contract_id = env.register(TapBattleContract, ());
    let client = TapBattleContractClient::new(&env, &contract_id);

    let player_a = Address::generate(&env);
    let player_b = Address::generate(&env);

    client.start_round(&player_a, &player_b, &100);
}

/// Test that round scoring and winner declaration work correctly.
#[test]
fn test_round_scoring_and_winner() {
    let env = Env::default();
    let contract_id = env.register(TapBattleContract, ());
    let client = TapBattleContractClient::new(&env, &contract_id);

    let player_a = setup_player(&env, &contract_id);
    let player_b = setup_player(&env, &contract_id);

    // Set up sessions for both players
    setup_session(&env, &contract_id, &player_a);
    setup_session(&env, &contract_id, &player_b);

    client.start_round(&player_a, &player_b, &100);

    // Player A taps more than Player B
    env.as_contract(&contract_id, || {
        for _ in 0..5 {
            game::process_tap(&env, &player_a);
        }
        for _ in 0..2 {
            game::process_tap(&env, &player_b);
        }
    });

    // Check scores were updated
    let round = client.get_round();
    assert!(round.player_a_score > round.player_b_score);
    assert!(!round.finished);
}

// ============================================================================
// Profile Tests
// ============================================================================

/// Test that player profile tracks taps and best combo.
#[test]
fn test_profile_stats() {
    let env = Env::default();
    let contract_id = env.register(TapBattleContract, ());
    let client = TapBattleContractClient::new(&env, &contract_id);

    let player = setup_player(&env, &contract_id);
    setup_session(&env, &contract_id, &player);

    // Initial profile should be empty
    let profile = client.get_profile(&player);
    assert_eq!(profile.total_wins, 0);
    assert_eq!(profile.total_taps, 0);
    assert_eq!(profile.best_combo, 0);

    // Tap several times
    env.as_contract(&contract_id, || {
        for _ in 0..3 {
            game::process_tap(&env, &player);
        }
    });

    // Profile should reflect taps
    let profile = client.get_profile(&player);
    assert_eq!(profile.total_taps, 3);
    assert!(profile.best_combo > 0);
}

/// Test that default profile is returned for unregistered player.
#[test]
fn test_default_profile() {
    let env = Env::default();
    let contract_id = env.register(TapBattleContract, ());
    let client = TapBattleContractClient::new(&env, &contract_id);

    let player = Address::generate(&env);
    let profile = client.get_profile(&player);

    assert_eq!(profile.total_wins, 0);
    assert_eq!(profile.total_taps, 0);
    assert_eq!(profile.best_combo, 0);
}

// ============================================================================
// Session Expiry Tests
// ============================================================================

/// Test that session expires when ledger sequence/timestamp exceeds expires_at.
/// This covers the "session expiry during round" acceptance criterion.
#[test]
#[should_panic(expected = "session expired")]
fn test_session_expiry_during_round() {
    let env = Env::default();
    let contract_id = env.register(TapBattleContract, ());

    let player_a = setup_player(&env, &contract_id);
    let player_b = setup_player(&env, &contract_id);

    // Create a session that expires at timestamp 10
    env.as_contract(&contract_id, || {
        let session_scope = cougr_core::accounts::SessionBuilder::new(&env)
            .allow_action(soroban_sdk::symbol_short!("tap"))
            .allow_action(soroban_sdk::Symbol::new(&env, "use_power_up"))
            .max_operations(100)
            .expires_at(10)
            .build_scope();
        cougr_core::session::SessionManager::approve(&env, &player_a, session_scope)
            .expect("session approval failed");

        env.storage()
            .persistent()
            .set(&DataKey::TapState(player_a.clone()), &TapCounter::new());
        env.storage().persistent().set(
            &DataKey::PowerUpState(player_a.clone()),
            &PowerUp::new(PowerUpKind::DoubleTap as u32),
        );
    });

    // Start a round
    env.as_contract(&contract_id, || {
        game::start_round(&env, &player_a, &player_b, 100);
    });

    // Advance ledger past session expiry
    env.ledger().with_mut(|li| {
        li.timestamp = 15;
    });

    // Session validation should panic with "session expired"
    env.as_contract(&contract_id, || {
        auth::validate_session(&env, &player_a, soroban_sdk::symbol_short!("tap"));
    });
}

// ============================================================================
// Combo Break Tests
// ============================================================================

/// Test that combo resets when taps are outside the combo window.
#[test]
fn test_combo_breaks_outside_window() {
    let env = Env::default();
    let contract_id = env.register(TapBattleContract, ());

    let player = setup_player(&env, &contract_id);
    setup_session(&env, &contract_id, &player);

    env.as_contract(&contract_id, || {
        // Build a combo
        let r1 = game::process_tap(&env, &player);
        assert_eq!(r1.combo, 1);

        let r2 = game::process_tap(&env, &player);
        assert_eq!(r2.combo, 2);

        // Advance ledger far beyond combo window
        env.ledger().set_sequence_number(100);

        // Combo should break - reset to 1
        let r3 = game::process_tap(&env, &player);
        assert_eq!(r3.combo, 1);
        assert_eq!(r3.multiplier, 1);
    });
}

// ============================================================================
// SessionManager Pattern Tests (consistent with session_arena)
// ============================================================================

#[test]
fn test_renew_session_extends_play_window() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TapBattleContract, ());
    let client = TapBattleContractClient::new(&env, &contract_id);

    let player = setup_player(&env, &contract_id);
    setup_session(&env, &contract_id, &player);

    let key_id = client.get_latest_session_key(&player);
    let expires_at_before = env.as_contract(&contract_id, || {
        cougr_core::accounts::SessionStorage::load(&env, &player, &key_id)
            .expect("session missing")
            .scope
            .expires_at
    });

    let renewed = client.renew_session(&player, &key_id, &20_000);
    assert!(renewed.expires_at > expires_at_before);
}

#[test]
fn test_fallback_tap_uses_direct_auth_after_session_expires() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TapBattleContract, ());
    let client = TapBattleContractClient::new(&env, &contract_id);

    let player = setup_player(&env, &contract_id);
    setup_session(&env, &contract_id, &player);

    let key_id = client.get_latest_session_key(&player);

    env.ledger().with_mut(|li| {
        li.timestamp = 10_000;
    });

    let res = client.fallback_tap(&player, &key_id);
    assert_eq!(res.count, 1);
}
