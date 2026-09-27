//! {{crate_name}}: a Cougr game contract generated from the `{{template_id}}` template.
//!
//! Real-time games cannot put every frame on chain. This contract demonstrates
//! the honest alternative: simulate off chain, commit a state hash and score
//! on chain at periodic checkpoints, and allow a dispute window before a result
//! is finalised.
//!
//! Demonstrates:
//!   - `impl_rich_component!`: components holding `Address` fields
//!   - `impl_component!`: fixed-size tick counter
//!   - `SorobanGame`: standard world load/save
//!   - `impl_soroban_game!`: wires the trait to a `#[contract]` struct

#![no_std]

pub mod components;
pub mod systems;
#[cfg(test)]
mod test;

use components::{
    CheckpointState, MatchConfig, MatchRecord, MATCH_ENTITY,
    STATUS_DISPUTED, STATUS_FINALISED, STATUS_RUNNING,
};
use systems::{validate_commit, validate_dispute, validate_finalize, CommitError, DisputeError, FinalizeError};

use cougr_core::game::SorobanGame;
use cougr_core::impl_soroban_game;
use cougr_core::simple_world::SimpleWorld;
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

// ─── API return types ─────────────────────────────────────────────────────────

/// Everything a client needs to track the match.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchState {
    pub player: Address,
    pub last_tick: u32,
    pub last_hash: u64,
    pub last_score: u32,
    pub status: u32,
}

/// Result of a `commit_checkpoint`, `dispute_checkpoint`, or `finalize_match`
/// call: whether it succeeded, the resulting state, and a short reason code on
/// rejection.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionResult {
    pub success: bool,
    pub match_state: MatchState,
    pub message: Symbol,
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
#[derive(Clone)]
pub struct {{ContractName}};

// Generates `load_world` / `save_world` against the "world" instance-storage key.
impl_soroban_game!({{ContractName}}, "world");

#[contractimpl]
impl {{ContractName}} {
    /// Open a new match. The caller becomes the authorised checkpoint committer.
    ///
    /// Any previous match state is discarded.
    pub fn start_match(env: Env, player: Address) -> MatchState {
        player.require_auth();

        let mut world = SimpleWorld::new(&env);
        let entity = world.spawn_entity();
        debug_assert_eq!(entity, MATCH_ENTITY);

        world.set_rich(
            &env,
            MATCH_ENTITY,
            &MatchConfig {
                player: player.clone(),
                started_at_ledger: env.ledger().sequence(),
            },
        );

        let initial_record = MatchRecord {
            last_hash: 0,
            last_score: 0,
            last_tick: 0,
            committed_at_ledger: env.ledger().sequence(),
            status: STATUS_RUNNING,
            authority: player,
        };
        world.set_rich(&env, MATCH_ENTITY, &initial_record);
        world.set_typed(
            &env,
            MATCH_ENTITY,
            &CheckpointState {
                last_committed_tick: 0,
            },
        );

        {{ContractName}}::save_world(&env, &world);
        Self::read_state(&env, &world)
    }

    /// Commit a checkpoint: record the tick index, state hash, and score for
    /// a batch of off-chain simulation ticks.
    ///
    /// The tick must be strictly greater than the last committed tick, and
    /// the match must be running.
    pub fn commit_checkpoint(
        env: Env,
        player: Address,
        tick: u32,
        state_hash: u64,
        score: u32,
    ) -> ActionResult {
        player.require_auth();

        let mut world = {{ContractName}}::load_world(&env);
        let config = Self::config(&env, &world);

        if player != config.player {
            return Self::rejected(&env, &world, symbol_short!("notplay"));
        }

        let record = Self::record(&env, &world);
        let cp = Self::cp_state(&env, &world);

        if let Err(err) = validate_commit(&record, &cp, tick) {
            let msg = match err {
                CommitError::MatchNotRunning => symbol_short!("notrun"),
                CommitError::TickNotMonotone => symbol_short!("oldtick"),
            };
            return Self::rejected(&env, &world, msg);
        }

        let new_record = MatchRecord {
            last_hash: state_hash,
            last_score: score,
            last_tick: tick,
            committed_at_ledger: env.ledger().sequence(),
            status: STATUS_RUNNING,
            authority: config.player,
        };
        world.set_rich(&env, MATCH_ENTITY, &new_record);
        world.set_typed(
            &env,
            MATCH_ENTITY,
            &CheckpointState {
                last_committed_tick: tick,
            },
        );
        {{ContractName}}::save_world(&env, &world);

        ActionResult {
            success: true,
            match_state: Self::read_state(&env, &world),
            message: symbol_short!("ok"),
        }
    }

    /// Dispute a committed checkpoint within the dispute window.
    ///
    /// The challenger provides the tick and their own hash. If the hash
    /// differs from the committed one, the match is marked disputed.
    pub fn dispute_checkpoint(
        env: Env,
        challenger: Address,
        tick: u32,
        claimed_hash: u64,
    ) -> ActionResult {
        challenger.require_auth();

        let mut world = {{ContractName}}::load_world(&env);
        let record = Self::record(&env, &world);

        if let Err(err) =
            validate_dispute(&record, tick, env.ledger().sequence(), claimed_hash)
        {
            let msg = match err {
                DisputeError::MatchNotRunning => symbol_short!("notrun"),
                DisputeError::TickMismatch => symbol_short!("badtick"),
                DisputeError::WindowClosed => symbol_short!("toolate"),
                DisputeError::HashMatches => symbol_short!("nohash"),
            };
            return Self::rejected(&env, &world, msg);
        }

        let disputed_record = MatchRecord {
            status: STATUS_DISPUTED,
            authority: challenger,
            ..record
        };
        world.set_rich(&env, MATCH_ENTITY, &disputed_record);
        {{ContractName}}::save_world(&env, &world);

        ActionResult {
            success: true,
            match_state: Self::read_state(&env, &world),
            message: symbol_short!("disputed"),
        }
    }

    /// Finalise the match once the dispute window has closed.
    ///
    /// Locks the last committed score; no further checkpoints or disputes
    /// are accepted.
    pub fn finalize_match(env: Env, player: Address) -> ActionResult {
        player.require_auth();

        let mut world = {{ContractName}}::load_world(&env);
        let record = Self::record(&env, &world);
        let config = Self::config(&env, &world);

        if player != config.player {
            return Self::rejected(&env, &world, symbol_short!("notplay"));
        }

        if let Err(err) = validate_finalize(&record, env.ledger().sequence()) {
            let msg = match err {
                FinalizeError::MatchNotRunning => symbol_short!("notrun"),
                FinalizeError::WindowStillOpen => symbol_short!("window"),
            };
            return Self::rejected(&env, &world, msg);
        }

        let final_record = MatchRecord {
            status: STATUS_FINALISED,
            ..record
        };
        world.set_rich(&env, MATCH_ENTITY, &final_record);
        {{ContractName}}::save_world(&env, &world);

        ActionResult {
            success: true,
            match_state: Self::read_state(&env, &world),
            message: symbol_short!("ok"),
        }
    }

    /// Current match state.
    pub fn get_state(env: Env) -> MatchState {
        let world = {{ContractName}}::load_world(&env);
        Self::read_state(&env, &world)
    }

    // ─── Internal helpers ─────────────────────────────────────────────────────

    fn config(env: &Env, world: &SimpleWorld) -> MatchConfig {
        world
            .get_rich::<MatchConfig>(env, MATCH_ENTITY)
            .unwrap_or_else(|| panic!("match not initialised"))
    }

    fn record(env: &Env, world: &SimpleWorld) -> MatchRecord {
        world
            .get_rich::<MatchRecord>(env, MATCH_ENTITY)
            .unwrap_or_else(|| panic!("match not initialised"))
    }

    fn cp_state(env: &Env, world: &SimpleWorld) -> CheckpointState {
        world
            .get_typed::<CheckpointState>(env, MATCH_ENTITY)
            .unwrap_or_else(|| panic!("match not initialised"))
    }

    fn read_state(env: &Env, world: &SimpleWorld) -> MatchState {
        let config = Self::config(env, world);
        let record = Self::record(env, world);
        MatchState {
            player: config.player,
            last_tick: record.last_tick,
            last_hash: record.last_hash,
            last_score: record.last_score,
            status: record.status,
        }
    }

    fn rejected(env: &Env, world: &SimpleWorld, message: Symbol) -> ActionResult {
        ActionResult {
            success: false,
            match_state: Self::read_state(env, world),
            message,
        }
    }
}
