use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, Vec,
};

struct TestSetup {
    env: Env,
    contract_id: Address,
    token_id: Address,
    players: Vec<Address>,
}

fn setup(player_count: u32) -> TestSetup {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract_v2(admin);
    let token_id = token.address();
    let asset = StellarAssetClient::new(&env, &token_id);
    let mut players = Vec::new(&env);
    for _ in 0..player_count {
        let player = Address::generate(&env);
        asset.mint(&player, &1_000);
        players.push_back(player);
    }
    let entrants = players.clone();
    let contract_id = env.register(StakedTournament, ());
    StakedTournamentClient::new(&env, &contract_id).initialize(
        &Address::generate(&env),
        &entrants,
        &token_id,
        &10,
        &60,
    );

    TestSetup {
        env,
        contract_id,
        token_id,
        players,
    }
}

fn tournament<'a>(setup: &'a TestSetup) -> StakedTournamentClient<'a> {
    StakedTournamentClient::new(&setup.env, &setup.contract_id)
}

fn token_balance(setup: &TestSetup, account: &Address) -> i128 {
    TokenClient::new(&setup.env, &setup.token_id).balance(account)
}

#[test]
fn four_player_bracket_settles_each_match_independently() {
    let setup = setup(4);
    let client = tournament(&setup);
    let p0 = setup.players.get(0).unwrap();
    let p1 = setup.players.get(1).unwrap();
    let p2 = setup.players.get(2).unwrap();
    let p3 = setup.players.get(3).unwrap();

    client.stake_match(&p0, &0);
    client.stake_match(&p1, &0);
    client.stake_match(&p2, &1);
    client.stake_match(&p3, &1);
    assert_eq!(token_balance(&setup, &setup.contract_id), 40);

    client.resolve_match(&p0, &0);
    assert_eq!(token_balance(&setup, &setup.contract_id), 20);
    assert_eq!(token_balance(&setup, &p1), 990);
    assert_eq!(token_balance(&setup, &p2), 990);
    assert_eq!(client.get_match(&0).winner, Some(p0.clone()));
    assert_eq!(client.get_match(&2).player_a, Some(p0.clone()));

    client.resolve_match(&p2, &1);
    assert_eq!(token_balance(&setup, &setup.contract_id), 0);
    assert_eq!(client.get_match(&1).winner, Some(p2.clone()));
    assert_eq!(client.get_match(&2).player_b, Some(p2.clone()));

    client.stake_match(&p0, &2);
    client.stake_match(&p2, &2);
    client.resolve_match(&p0, &2);

    assert_eq!(client.get_match(&2).winner, Some(p0.clone()));
    assert_eq!(client.champion(), Some(p0.clone()));
    assert_eq!(token_balance(&setup, &p0), 1_020);
    assert_eq!(token_balance(&setup, &p2), 1_000);
    assert_eq!(token_balance(&setup, &setup.contract_id), 0);
}

#[test]
fn no_show_refund_advances_present_player_through_the_bracket() {
    let setup = setup(4);
    let client = tournament(&setup);
    let p0 = setup.players.get(0).unwrap();
    let p1 = setup.players.get(1).unwrap();
    let p2 = setup.players.get(2).unwrap();
    let p3 = setup.players.get(3).unwrap();

    client.stake_match(&p0, &0);
    client.stake_match(&p2, &1);
    client.stake_match(&p3, &1);
    setup.env.ledger().set_timestamp(61);

    client.forfeit_no_show(&0);
    assert_eq!(token_balance(&setup, &p0), 1_000);
    assert_eq!(client.get_match(&2).player_a, Some(p0.clone()));
    assert_eq!(token_balance(&setup, &p1), 1_000);

    client.resolve_match(&p2, &1);
    client.stake_match(&p0, &2);
    client.stake_match(&p2, &2);
    client.resolve_match(&p0, &2);

    assert_eq!(client.champion(), Some(p0.clone()));
    assert_eq!(token_balance(&setup, &setup.contract_id), 0);
}

#[test]
#[should_panic]
fn settled_match_cannot_be_settled_twice() {
    let setup = setup(4);
    let client = tournament(&setup);
    let p0 = setup.players.get(0).unwrap();
    let p1 = setup.players.get(1).unwrap();

    client.stake_match(&p0, &0);
    client.stake_match(&p1, &0);
    client.resolve_match(&p0, &0);
    client.resolve_match(&p0, &0);
}

#[test]
fn eight_player_bracket_has_seven_match_slots() {
    let setup = setup(8);
    assert_eq!(tournament(&setup).get_match(&6).next_match, None);
    assert_eq!(tournament(&setup).get_match(&0).next_match, Some(4));
    assert_eq!(tournament(&setup).get_match(&4).next_match, Some(6));
}
