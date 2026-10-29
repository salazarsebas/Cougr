use crate::CrossAssetRacingLeague;
use crate::CrossAssetRacingLeagueClient;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

#[test]
fn test_league_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let contract_id = env.register_contract(None, CrossAssetRacingLeague);
    let client = CrossAssetRacingLeagueClient::new(&env, &contract_id);

    client.init_league(&owner);
    let state = client.get_game_state();
    assert_eq!(state.owner, owner);
    assert_eq!(state.current_season, 1);
    assert_eq!(state.current_race_id, 1);
    assert!(state.league_active);
}

#[test]
fn test_create_race() {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let contract_id = env.register_contract(None, CrossAssetRacingLeague);
    let client = CrossAssetRacingLeagueClient::new(&env, &contract_id);

    client.init_league(&owner);
    let race_id = client.create_race(&owner, &300u32);

    assert_eq!(race_id, 1u32);
    let race = client.get_race(&race_id);
    assert_eq!(race.race_id, 1);
    assert_eq!(race.phase, 0);
    assert_eq!(race.duration, 300);
    assert_eq!(race.entrants_count, 0);
}

#[test]
fn test_enter_race() {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);
    let contract_id = env.register_contract(None, CrossAssetRacingLeague);
    let client = CrossAssetRacingLeagueClient::new(&env, &contract_id);

    client.init_league(&owner);
    let race_id = client.create_race(&owner, &300u32);

    client.enter_race(&player1, &race_id);
    client.enter_race(&player2, &race_id);

    let race = client.get_race(&race_id);
    assert_eq!(race.entrants_count, 2);
}

#[test]
fn test_start_race() {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let player = Address::generate(&env);
    let contract_id = env.register_contract(None, CrossAssetRacingLeague);
    let client = CrossAssetRacingLeagueClient::new(&env, &contract_id);

    client.init_league(&owner);
    let race_id = client.create_race(&owner, &300u32);
    client.enter_race(&player, &race_id);
    client.start_race(&owner, &race_id);

    let race = client.get_race(&race_id);
    assert_eq!(race.phase, 1);
}

#[test]
fn test_credit_payment() {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let player = Address::generate(&env);
    let mut receipt_bytes = [0u8; 32];
    receipt_bytes[0] = 1;
    let payment_receipt = BytesN::<32>::from_array(&env, &receipt_bytes);
    let contract_id = env.register_contract(None, CrossAssetRacingLeague);
    let client = CrossAssetRacingLeagueClient::new(&env, &contract_id);

    client.init_league(&owner);
    client.credit_payment(&owner, &player, &100u32, &payment_receipt);

    let credits = client.get_player_credits(&player);
    assert_eq!(credits, 100u32);
}

#[test]
fn test_activate_boost() {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let player = Address::generate(&env);
    let mut receipt_bytes = [0u8; 32];
    receipt_bytes[0] = 2;
    let payment_receipt = BytesN::<32>::from_array(&env, &receipt_bytes);
    let contract_id = env.register_contract(None, CrossAssetRacingLeague);
    let client = CrossAssetRacingLeagueClient::new(&env, &contract_id);

    client.init_league(&owner);
    let race_id = client.create_race(&owner, &300u32);

    client.credit_payment(&owner, &player, &100u32, &payment_receipt);
    client.enter_race(&player, &race_id);
    client.start_race(&owner, &race_id);
    client.activate_boost(&player, &race_id, &1u32);

    let credits = client.get_player_credits(&player);
    assert_eq!(credits, 90u32);
}

#[test]
fn test_complete_race_standings() {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);
    let player3 = Address::generate(&env);
    let contract_id = env.register_contract(None, CrossAssetRacingLeague);
    let client = CrossAssetRacingLeagueClient::new(&env, &contract_id);

    client.init_league(&owner);
    let race_id = client.create_race(&owner, &300u32);

    client.enter_race(&player1, &race_id);
    client.enter_race(&player2, &race_id);
    client.enter_race(&player3, &race_id);

    client.start_race(&owner, &race_id);
    client.complete_race(&owner, &race_id);

    let standing1 = client.get_player_standing(&1u32, &player1);
    let standing2 = client.get_player_standing(&1u32, &player2);
    let standing3 = client.get_player_standing(&1u32, &player3);

    assert_eq!(standing1.points, 10u128);
    assert_eq!(standing2.points, 6u128);
    assert_eq!(standing3.points, 3u128);
}
