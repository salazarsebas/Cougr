//! Integration tests for `cougr_core::test` sandbox helpers.

#![cfg(feature = "testutils")]

use cougr_core::game::SorobanGame;
use cougr_core::impl_soroban_game;
use cougr_core::test::{GameHarness, ReplayLog, Scenario, SnapshotAssert, TurnIndex, WorldFixture};
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
#[derive(Clone)]
pub struct SandboxGame;

impl_soroban_game!(SandboxGame, "world");

#[contractimpl]
impl SandboxGame {
    pub fn spawn(env: Env) -> u32 {
        let mut world = SandboxGame::load_world(&env);
        let id = world.spawn_entity();
        SandboxGame::save_world(&env, &world);
        id
    }

    pub fn entity_count(env: Env) -> u32 {
        SandboxGame::load_world(&env)
            .next_entity_id()
            .saturating_sub(1)
    }
}

#[test]
fn harness_registers_contract_and_builds_client() {
    let env = Env::default();
    let harness = GameHarness::new(env, SandboxGame);
    let client = SandboxGameClient::new(harness.env(), harness.contract_id());
    assert_eq!(client.entity_count(), 0);
}

#[test]
fn scenario_drives_multiple_spawns() {
    let env = Env::default();
    let harness = GameHarness::new(env, SandboxGame);

    Scenario::new("spawn four")
        .players(2)
        .turns(4)
        .run_and_assert(
            &harness,
            |_player, _turn, h| {
                SandboxGameClient::new(h.env(), h.contract_id()).spawn();
            },
            |h| {
                assert_eq!(
                    SandboxGameClient::new(h.env(), h.contract_id()).entity_count(),
                    4
                );
            },
        );
}

#[test]
fn world_fixture_injects_prebuilt_state() {
    let env = Env::default();
    let harness = GameHarness::new(env, SandboxGame);

    let mut fixture = WorldFixture::empty(harness.env());
    fixture.spawn_entity();
    fixture.spawn_entity();
    fixture.inject::<SandboxGame>(&harness);

    assert_eq!(
        SandboxGameClient::new(harness.env(), harness.contract_id()).entity_count(),
        2
    );
}

#[test]
fn replay_log_records_checkpoints_and_forks() {
    let env = Env::default();
    let harness = GameHarness::new(env, SandboxGame);
    let mut log = ReplayLog::new();

    SandboxGameClient::new(harness.env(), harness.contract_id()).spawn();
    log.record::<SandboxGame>(TurnIndex(0), &harness);

    SandboxGameClient::new(harness.env(), harness.contract_id()).spawn();
    log.record::<SandboxGame>(TurnIndex(1), &harness);

    log.assert_differs_at(0, 1);
    log.assert_same_at(1, 1);

    let forked = log.fork_from::<SandboxGame>(&harness, 0);
    SnapshotAssert::assert_entity_count(forked.world(), 1);
}

#[test]
fn mock_players_rotate_through_scenario() {
    let env = Env::default();
    let mut harness = GameHarness::new(env, SandboxGame);
    harness.mock_players(3);
    harness.mock_all_auths();

    let mut slots = [0u32; 6];
    let mut idx = 0usize;
    Scenario::new("player rotation")
        .players(3)
        .turns(6)
        .run(&harness, |player, _turn, h| {
            slots[idx] = player.0;
            idx += 1;
            let _ = h.player(player);
        });

    assert_eq!(slots, [0, 1, 2, 0, 1, 2]);
}
