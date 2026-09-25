//! Soroban resource budget for one pinned turn-based scenario.
//!
//! `GameHarness` reports the budget dimensions `env.cost_estimate().resources()`
//! meters, and this test is the gate: the scenario below runs the shape a
//! turn-based game does - one match, two players alternating validated writes -
//! and every dimension has to stay inside the committed baseline in
//! `tests/turn_based_resource_baseline.csv`.
//!
//! The report is a regression signal for this scenario, not a fee quote. The
//! sandbox registers a Rust contract, so VM instantiation, Wasm reads, and rent
//! bumps are not part of the numbers. The CLI's testing guide has the details.

#![cfg(feature = "testutils")]

use cougr_core::test::{GameHarness, PlayerSlot, ResourceBudget, ResourceReport, Scenario};
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

/// Documented slack, next to the baseline it applies to:
///
/// - 10% on the metered CPU, memory, and byte dimensions, which move with the
///   compiler and the host version.
/// - 0 extra ledger entries. Entry counts are exact integers rather than noisy
///   measurements, so a new write or an O(n) loop fails the gate. Raise this
///   only together with the baseline, and say why in the PR.
const TOLERANCE_PERCENT: u32 = 10;
const EXTRA_ENTRIES: u32 = 0;

/// Committed baseline, one row per pinned scenario.
const BASELINE: &str = include_str!("turn_based_resource_baseline.csv");

/// `scenario | turn owner | board`: the smallest realistic turn-based match.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchState {
    pub x: Address,
    pub o: Address,
    pub is_x_turn: bool,
    pub moves: u32,
    pub cells: Vec<u32>,
}

/// Instance-storage key for the single match this probe contract keeps.
const MATCH: Symbol = symbol_short!("MATCH");

/// Stands in for `cli/templates/turn-based`: validate, then write, then hand the
/// turn over, with the match in instance storage instead of a world.
#[contract]
#[derive(Clone)]
pub struct TurnBasedProbe;

fn load(env: &Env) -> MatchState {
    env.storage()
        .instance()
        .get(&MATCH)
        .expect("match is not initialised")
}

#[contractimpl]
impl TurnBasedProbe {
    pub fn init_match(env: Env, x: Address, o: Address) -> MatchState {
        x.require_auth();

        let mut cells = Vec::new(&env);
        for _ in 0..9 {
            cells.push_back(0);
        }

        let state = MatchState {
            x,
            o,
            is_x_turn: true,
            moves: 0,
            cells,
        };
        env.storage().instance().set(&MATCH, &state);
        state
    }

    /// Play `position` for the caller. Returns the move count.
    pub fn make_move(env: Env, player: Address, position: u32) -> u32 {
        player.require_auth();

        let mut state = load(&env);
        assert!(state.moves < 9, "match is over");
        assert!(position < 9, "position is out of bounds");
        assert_eq!(state.cells.get(position), Some(0), "position is occupied");

        let expected = if state.is_x_turn {
            state.x.clone()
        } else {
            state.o.clone()
        };
        assert_eq!(player, expected, "not this player's turn");

        state
            .cells
            .set(position, if state.is_x_turn { 1 } else { 2 });
        state.is_x_turn = !state.is_x_turn;
        state.moves += 1;

        env.storage().instance().set(&MATCH, &state);
        state.moves
    }

    pub fn get_state(env: Env) -> MatchState {
        load(&env)
    }
}

/// The committed budget for `scenario`, or a panic naming the missing row.
fn budget_for(scenario: &str) -> ResourceBudget {
    for line in BASELINE.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, row) = line
            .split_once(',')
            .expect("a baseline row starts with the scenario name");
        if name.trim() != scenario {
            continue;
        }
        let baseline = ResourceReport::from_csv(row).expect("a baseline row holds eight numbers");
        return ResourceBudget::new(baseline, TOLERANCE_PERCENT, EXTRA_ENTRIES);
    }

    panic!("tests/turn_based_resource_baseline.csv has no row for {scenario}");
}

#[test]
fn turn_based_match_stays_inside_its_resource_budget() {
    let env = Env::default();
    let mut harness = GameHarness::new(env, TurnBasedProbe);
    harness.mock_all_auths();
    harness.mock_players(2);

    let x = harness.player(PlayerSlot(0)).clone();
    let o = harness.player(PlayerSlot(1)).clone();
    TurnBasedProbeClient::new(harness.env(), harness.contract_id()).init_match(&x, &o);

    let init_report = harness.assert_resource_budget("init_match", &budget_for("init_match"));
    println!("cougr resource budget init_match: {}", init_report.to_csv());

    // Metering is per top-level invocation, so every turn is checked against the
    // committed single-move budget rather than only the last one.
    let move_budget = budget_for("alternating_move");
    let mut last_move = init_report;
    let scenario = Scenario::new("turn_based_alternating_match")
        .players(2)
        .turns(9);
    scenario.run(&harness, |slot, turn, harness| {
        let client = TurnBasedProbeClient::new(harness.env(), harness.contract_id());
        client.make_move(harness.player(slot), &turn.0);
        last_move = harness.assert_resource_budget("alternating_move", &move_budget);
    });
    println!(
        "cougr resource budget alternating_move: {}",
        last_move.to_csv()
    );

    let client = TurnBasedProbeClient::new(harness.env(), harness.contract_id());
    assert_eq!(client.get_state().moves, 9);
    assert!(init_report.write_entries >= 1, "the match must be stored");
    assert!(last_move.instructions > 0, "a move must cost instructions");
}
