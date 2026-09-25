extern crate std;

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

use crate::{Error, MatchEscrow, MatchState, StakedMatch, StakedMatchClient};

const STAKE: i128 = 100;
const DEADLINE: u64 = 1_000;

struct Fixture<'a> {
    env: Env,
    client: StakedMatchClient<'a>,
    token: token::Client<'a>,
    host: Address,
    challenger: Address,
    stranger: Address,
}

fn setup<'a>() -> Fixture<'a> {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(0);

    let issuer = Address::generate(&env);
    let asset = env.register_stellar_asset_contract_v2(issuer.clone());
    let token_id = asset.address();

    let host = Address::generate(&env);
    let challenger = Address::generate(&env);
    let stranger = Address::generate(&env);

    let minter = token::StellarAssetClient::new(&env, &token_id);
    minter.mint(&host, &1_000);
    minter.mint(&challenger, &1_000);

    let contract_id = env.register(StakedMatch, ());
    let client = StakedMatchClient::new(&env, &contract_id);

    Fixture {
        token: token::Client::new(&env, &token_id),
        env,
        client,
        host,
        challenger,
        stranger,
    }
}

fn open_match(f: &Fixture) -> u64 {
    f.client
        .create_match(&f.host, &f.token.address, &STAKE, &DEADLINE)
}

fn active_match(f: &Fixture) -> u64 {
    let id = open_match(f);
    f.client.join(&id, &f.challenger);
    id
}

/// Threat: a winner is paid less or more than the pot both players staked.
#[test]
fn winner_withdraws_the_whole_pot() {
    let f = setup();
    let id = active_match(&f);

    f.client.agree_result(&id, &f.host);
    let paid = f.client.withdraw(&id, &f.host);

    assert_eq!(paid, STAKE * 2);
    assert_eq!(f.token.balance(&f.host), 1_000 - STAKE + STAKE * 2);
    assert_eq!(f.token.balance(&f.challenger), 1_000 - STAKE);
}

/// Threat: a draw pays one player twice, or strands the pot in the contract.
#[test]
fn draw_returns_each_stake_exactly_once() {
    let f = setup();
    let id = active_match(&f);

    f.client.agree_draw(&id);

    assert_eq!(f.client.withdraw(&id, &f.host), STAKE);
    assert_eq!(f.client.withdraw(&id, &f.challenger), STAKE);
    assert_eq!(f.token.balance(&f.host), 1_000);
    assert_eq!(f.token.balance(&f.challenger), 1_000);
    assert_eq!(f.token.balance(&f.client.address), 0);
}

/// Threat: a host cancels after an opponent has already committed a stake.
#[test]
fn cancel_refunds_host_and_is_rejected_once_joined() {
    let f = setup();
    let id = open_match(&f);

    f.client.cancel(&id);
    assert_eq!(f.client.withdraw(&id, &f.host), STAKE);
    assert_eq!(f.token.balance(&f.host), 1_000);

    let joined = active_match(&f);
    assert_eq!(f.client.try_cancel(&joined), Err(Ok(Error::MatchNotOpen)));
}

/// Threat: an abandoned match locks the host's stake forever, or a timeout can
/// be claimed early to escape a match an opponent is about to join.
#[test]
fn timeout_refunds_only_after_the_deadline() {
    let f = setup();
    let id = open_match(&f);

    assert_eq!(
        f.client.try_claim_timeout(&id),
        Err(Ok(Error::JoinDeadlineNotReached))
    );

    f.env.ledger().set_timestamp(DEADLINE);
    f.client.claim_timeout(&id);

    assert_eq!(f.client.withdraw(&id, &f.host), STAKE);
    assert_eq!(f.token.balance(&f.host), 1_000);
}

/// Threat: a settled match is settled again, overwriting the agreed winner, or
/// the winner withdraws the pot a second time.
#[test]
fn double_settle_and_double_withdraw_are_rejected() {
    let f = setup();
    let id = active_match(&f);

    f.client.agree_result(&id, &f.host);
    assert_eq!(
        f.client.try_agree_result(&id, &f.challenger),
        Err(Ok(Error::MatchNotActive))
    );
    assert_eq!(f.client.try_agree_draw(&id), Err(Ok(Error::MatchNotActive)));

    f.client.withdraw(&id, &f.host);
    assert_eq!(
        f.client.try_withdraw(&id, &f.host),
        Err(Ok(Error::NothingToWithdraw))
    );
    assert_eq!(f.token.balance(&f.client.address), 0);
}

/// Threat: an address that never staked drains a settled pot, or the loser
/// withdraws despite having agreed to the result.
#[test]
fn stranger_and_loser_cannot_withdraw() {
    let f = setup();
    let id = active_match(&f);

    f.client.agree_result(&id, &f.host);

    assert_eq!(
        f.client.try_withdraw(&id, &f.stranger),
        Err(Ok(Error::NotAParticipant))
    );
    assert_eq!(
        f.client.try_withdraw(&id, &f.challenger),
        Err(Ok(Error::NothingToWithdraw))
    );
    assert_eq!(f.token.balance(&f.client.address), STAKE * 2);
}

/// Threat: the host joins their own match to fake an opponent.
#[test]
fn host_cannot_join_their_own_match() {
    let f = setup();
    let id = open_match(&f);

    assert_eq!(
        f.client.try_join(&id, &f.host),
        Err(Ok(Error::HostCannotJoinOwnMatch))
    );
}

/// Threat: an opponent joins after the host is entitled to a refund.
#[test]
fn join_is_rejected_after_the_deadline() {
    let f = setup();
    let id = open_match(&f);

    f.env.ledger().set_timestamp(DEADLINE);
    assert_eq!(
        f.client.try_join(&id, &f.challenger),
        Err(Ok(Error::JoinDeadlinePassed))
    );
}

/// Threat: a match is opened with a stake or deadline that cannot settle.
#[test]
fn invalid_stake_and_deadline_are_rejected() {
    let f = setup();

    assert_eq!(
        f.client
            .try_create_match(&f.host, &f.token.address, &0, &DEADLINE),
        Err(Ok(Error::InvalidStake))
    );
    assert_eq!(
        f.client
            .try_create_match(&f.host, &f.token.address, &STAKE, &0),
        Err(Ok(Error::DeadlineInPast))
    );
}

/// Threat: a winner is named who never staked into the match.
#[test]
fn winner_must_be_a_participant() {
    let f = setup();
    let id = active_match(&f);

    assert_eq!(
        f.client.try_agree_result(&id, &f.stranger),
        Err(Ok(Error::WinnerNotAParticipant))
    );
}

/// Threat: the escrow reports a state that does not match what was staked.
#[test]
fn escrow_state_tracks_the_pot() {
    let f = setup();
    let id = open_match(&f);

    let open: MatchEscrow = f.client.get_match(&id);
    assert_eq!(open.state, MatchState::Open);
    assert_eq!(open.pot, STAKE);
    assert_eq!(open.challenger, None);

    f.client.join(&id, &f.challenger);
    let active: MatchEscrow = f.client.get_match(&id);
    assert_eq!(active.state, MatchState::Active);
    assert_eq!(active.pot, STAKE * 2);
    assert_eq!(active.challenger, Some(f.challenger.clone()));
}
