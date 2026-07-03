use super::*;
use cougr_core::circuits::{fair_dice, test_fixtures, CircuitId};
use cougr_core::test::{GameHarness, Scenario};
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn init_duel_binds_seed_commitment() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, DiceDuel);
    let client = DiceDuelClient::new(harness.env(), harness.contract_id());
    let seed = test_fixtures::pipeline_public_bytes32(harness.env(), CircuitId::FairDice);

    let config = client.init_duel(&6, &seed);
    assert_eq!(config.sides, 6);
    assert_eq!(config.seed_commitment, seed);

    let spec = fair_dice(harness.env(), 6, &seed).unwrap();
    assert_eq!(spec.circuit_id, CircuitId::FairDice);
}

#[test]
fn submit_roll_accepts_pipeline_proof() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, DiceDuel);
    let client = DiceDuelClient::new(harness.env(), harness.contract_id());
    let player = Address::generate(harness.env());
    let seed = test_fixtures::pipeline_public_bytes32(harness.env(), CircuitId::FairDice);
    let proof = test_fixtures::pipeline_proof(harness.env(), CircuitId::FairDice);

    client.init_duel(&6, &seed);
    assert!(client.submit_roll(&player, &6, &5, &proof));
    let record = client.roll_record(&player);
    assert_eq!(record.roll, 6);
    assert_eq!(record.nonce, 5);
}

#[test]
fn scenario_two_player_rolls() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, DiceDuel);
    let client = DiceDuelClient::new(harness.env(), harness.contract_id());
    let seed = test_fixtures::pipeline_public_bytes32(harness.env(), CircuitId::FairDice);
    let proof = test_fixtures::pipeline_proof(harness.env(), CircuitId::FairDice);
    client.init_duel(&6, &seed);

    let p1 = Address::generate(harness.env());
    let p2 = Address::generate(harness.env());

    Scenario::new("dice duel")
        .players(2)
        .turns(2)
        .run(&harness, |player, turn, h| {
            let c = DiceDuelClient::new(h.env(), h.contract_id());
            let roller = if player.0 == 0 { &p1 } else { &p2 };
            if player.0 == 0 && turn.0 == 0 {
                assert!(c.submit_roll(roller, &6, &5, &proof));
            }
        });
}

#[test]
fn submit_roll_rejects_out_of_range_result() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, DiceDuel);
    let client = DiceDuelClient::new(harness.env(), harness.contract_id());
    let player = Address::generate(harness.env());
    let seed = test_fixtures::pipeline_public_bytes32(harness.env(), CircuitId::FairDice);
    let proof = test_fixtures::pipeline_proof(harness.env(), CircuitId::FairDice);

    client.init_duel(&6, &seed);
    assert!(!client.submit_roll(&player, &7, &5, &proof));
}
