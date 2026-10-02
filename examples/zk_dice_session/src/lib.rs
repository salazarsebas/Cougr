//! zk_dice_session - session-scoped ZK dice example.
//!
//! This example composes two existing Cougr subsystems that have never been
//! used together before:
//!
//! * **ZK proof verification** – the `fair_dice` circuit from `examples/dice_duel`
//!   verifies each roll result on-chain without revealing the server seed.
//! * **Session keys** – the session lifecycle from `examples/session_arena`
//!   gates every proof submission behind a scoped key, so a player approves
//!   once per session and rolls many times without a wallet prompt.
//!
//! The combination means players can submit many rapid ZK-proven rolls within
//! a single approved session. No new circuit or account model is introduced –
//! only the wiring between the two.
//!
//! # Contract flow
//! 1. Owner calls [`ZkDiceSession::init_game`] to set the die face count and
//!    a seed commitment.
//! 2. Player calls [`ZkDiceSession::open_session`] (once per session window).
//!    This requires an owner signature and returns an `ActiveSession`.
//! 3. Player calls [`ZkDiceSession::roll`] any number of times with a Groth16
//!    proof and the `key_id` from step 2 – no wallet prompt required.
//! 4. Anyone can read a player's last accepted roll with
//!    [`ZkDiceSession::last_roll`].

#![no_std]

use cougr_core::accounts::{GameAction, SessionBuilder, SessionStorage};
use cougr_core::circuits::fair_dice;
use cougr_core::session::{ActiveSession, SessionManager};
use cougr_core::zk::Groth16Proof;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol,
};

// ─── Storage keys ────────────────────────────────────────────────────────────

fn config_key(_env: &Env) -> Symbol {
    symbol_short!("cfg")
}

fn roll_key(_env: &Env, player: &Address) -> (Symbol, Address) {
    (symbol_short!("roll"), player.clone())
}

// ─── Contract types ──────────────────────────────────────────────────────────

/// Game configuration stored on initialisation.
#[contracttype]
#[derive(Clone, Debug)]
pub struct GameConfig {
    /// Number of faces on the die (must be ≥ 2).
    pub sides: u32,
    /// Pedersen commitment to the server seed, used as the Groth16 VK binding.
    pub seed_commitment: BytesN<32>,
}

/// The last accepted roll for a player.
#[contracttype]
#[derive(Clone, Debug)]
pub struct RollRecord {
    /// The player who submitted the roll.
    pub player: Address,
    /// Verified roll result in `[1, sides]`.
    pub roll: u32,
    /// Anti-replay nonce carried inside the proof's public inputs.
    pub nonce: u32,
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
#[derive(Clone)]
pub struct ZkDiceSession;

#[contractimpl]
impl ZkDiceSession {
    // ── Initialisation ───────────────────────────────────────────────────────

    /// Initialise the game with a die face count and a server seed commitment.
    ///
    /// Reuses `fair_dice` from `src/circuits/fair_dice.rs` without
    /// modification – the call here validates parameters and caches the config.
    ///
    /// # Panics
    /// Panics if `sides < 2` (circuit contract).
    pub fn init_game(env: Env, sides: u32, seed_commitment: BytesN<32>) -> GameConfig {
        // Validate using the existing fair_dice builder – this checks sides ≥ 2.
        let _spec = fair_dice(&env, sides, &seed_commitment).expect("invalid game params");

        let config = GameConfig {
            sides,
            seed_commitment,
        };
        env.storage().instance().set(&config_key(&env), &config);
        config
    }

    // ── Session management ───────────────────────────────────────────────────

    /// Open a session that authorises the `"roll"` action for up to
    /// `max_rolls` rolls, expiring `expires_in` seconds from now.
    ///
    /// Reuses the session lifecycle from `examples/session_arena` unchanged.
    /// Requires an owner wallet signature (the only one the player ever makes
    /// per session).
    pub fn open_session(
        env: Env,
        owner: Address,
        max_rolls: u32,
        expires_in: u64,
    ) -> ActiveSession {
        // One wallet signature per session – the whole point of session keys.
        owner.require_auth();

        // Build a scope restricted to the single "roll" action.
        let scope = SessionBuilder::new(&env)
            .allow_action(symbol_short!("roll"))
            .max_operations(max_rolls)
            .expires_in(expires_in)
            .build_scope();

        let key = SessionManager::approve(&env, &owner, scope).expect("session approved");
        let status = SessionManager::status(&env, &owner, &key.key_id).expect("session status");
        ActiveSession::from_status(&status, key.scope.expires_at)
    }

    // ── Gameplay ─────────────────────────────────────────────────────────────

    /// Submit a Groth16 proof of a fair dice roll, authorized by the session
    /// key identified by `key_id`.
    ///
    /// Authorization flow:
    /// 1. Load the session key and run `SessionManager::execute_action` –
    ///    this validates expiry, budget, and action scope without a wallet
    ///    prompt.
    /// 2. Reconstruct the `fair_dice` circuit spec and call
    ///    `spec.verify_dice_roll` – **identical** to the call in
    ///    `examples/dice_duel/src/lib.rs`. The proof format is unchanged.
    /// 3. If both gates pass, persist the `RollRecord`.
    ///
    /// Returns `true` on success, `false` if the proof is invalid.
    ///
    /// # Panics
    /// Panics if the session is missing, expired, or its budget is exhausted.
    pub fn roll(
        env: Env,
        owner: Address,
        key_id: BytesN<32>,
        roll_result: u32,
        nonce: u32,
        proof: Groth16Proof,
    ) -> bool {
        // ── Gate 1: session authorization (no wallet prompt) ─────────────────
        let session = SessionStorage::load(&env, &owner, &key_id).expect("session missing");

        let action = GameAction {
            system_name: symbol_short!("roll"),
            data: soroban_sdk::Bytes::new(&env),
        };

        // consume one operation from the session budget
        SessionManager::execute_action(
            &env,
            &owner,
            &session,
            action,
            env.ledger().timestamp().saturating_add(60),
        )
        .expect("session rejected");

        // ── Gate 2: ZK proof verification (unchanged fair_dice path) ─────────
        let config: GameConfig = env
            .storage()
            .instance()
            .get(&config_key(&env))
            .expect("game not initialized");

        let spec =
            fair_dice(&env, config.sides, &config.seed_commitment).expect("circuit spec error");

        let ok = spec
            .verify_dice_roll(&env, &proof, roll_result, nonce)
            .unwrap_or(false);

        if ok {
            let record = RollRecord {
                player: owner.clone(),
                roll: roll_result,
                nonce,
            };
            env.storage()
                .instance()
                .set(&roll_key(&env, &owner), &record);
        }

        ok
    }

    // ── Queries ──────────────────────────────────────────────────────────────

    /// Return the last accepted roll for `player`.
    ///
    /// # Panics
    /// Panics if no roll has been accepted for the player yet.
    pub fn last_roll(env: Env, player: Address) -> RollRecord {
        env.storage()
            .instance()
            .get(&roll_key(&env, &player))
            .expect("no roll on record")
    }
}

#[cfg(test)]
mod tests;
