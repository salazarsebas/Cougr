use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Env;

fn setup() -> (Env, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SeasonLadder, ());
    (env, contract_id)
}

fn client<'a>(env: &'a Env, contract_id: &'a Address) -> SeasonLadderClient<'a> {
    SeasonLadderClient::new(env, contract_id)
}

fn registered_pair(env: &Env, contract_id: &Address) -> (Address, Address) {
    let alice = Address::generate(env);
    let bob = Address::generate(env);
    let c = client(env, contract_id);
    c.register_player(&alice);
    c.register_player(&bob);
    (alice, bob)
}

/// Reports `(home, away)` as fixture `match_no` with the given score.
#[allow(clippy::too_many_arguments)]
fn report(
    env: &Env,
    contract_id: &Address,
    admin: &Address,
    home: &Address,
    away: &Address,
    match_no: u32,
    home_goals: u32,
    away_goals: u32,
) -> MatchResult {
    client(env, contract_id).report_result(admin, home, away, &match_no, &home_goals, &away_goals)
}

// ── Initialization ──────────────────────────────────────────────────────────

#[test]
fn initialize_binds_the_owner() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);
    client(&env, &contract_id).initialize(&admin);

    assert_eq!(client(&env, &contract_id).owner(), Some(admin));
    assert!(!client(&env, &contract_id).is_paused());
}

// ── Happy path: result + first match boundary ───────────────────────────────

#[test]
fn first_result_between_even_players_splits_sixteen() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);
    client(&env, &contract_id).initialize(&admin);
    let (alice, bob) = registered_pair(&env, &contract_id);

    // First match boundary: both start at 1000, expected score is exactly
    // 500_000 ppm, so the swing is K_FACTOR / 2 = 16.
    let result = report(&env, &contract_id, &admin, &alice, &bob, 1, 2, 1);
    assert_eq!(result.home_delta, 16);
    assert_eq!(result.away_delta, -16);

    let home = client(&env, &contract_id).player(&alice);
    let away = client(&env, &contract_id).player(&bob);
    assert_eq!(home.rating, 1_016);
    assert_eq!(away.rating, 984);
    assert_eq!((home.wins, home.draws, home.losses), (1, 0, 0));
    assert_eq!((away.wins, away.draws, away.losses), (0, 0, 1));
    assert_eq!(home.matches, 1);

    // Ratings are conserved when no floor clamp triggers.
    assert_eq!(home.rating + away.rating, 2 * START_RATING);

    // The stored fixture round-trips through the view.
    let stored = client(&env, &contract_id).fixture(&alice, &bob, &1);
    assert_eq!(stored.home_goals, 2);
    assert_eq!(stored.away_goals, 1);
    assert_eq!(client(&env, &contract_id).matches_played(&alice, &bob), 1);
}

#[test]
fn draw_at_even_ratings_changes_nothing() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);
    client(&env, &contract_id).initialize(&admin);
    let (alice, bob) = registered_pair(&env, &contract_id);

    let result = report(&env, &contract_id, &admin, &alice, &bob, 1, 1, 1);
    assert_eq!(result.home_delta, 0);
    assert_eq!(result.away_delta, 0);
    assert_eq!(client(&env, &contract_id).rating(&alice), START_RATING);
    assert_eq!(client(&env, &contract_id).rating(&bob), START_RATING);
    assert_eq!(client(&env, &contract_id).player(&alice).draws, 1);
}

// ── Rating boundaries: upset and repeated pair ──────────────────────────────

#[test]
fn upset_win_pays_more_than_the_even_rating_win() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);
    client(&env, &contract_id).initialize(&admin);
    let (alice, bob) = registered_pair(&env, &contract_id);

    // Widen the gap: alice wins five straight. Favourite wins shrink from
    // 16 (even) to 15 as the gap grows.
    let first = report(&env, &contract_id, &admin, &alice, &bob, 1, 1, 0);
    assert_eq!(first.home_delta, 16);
    for match_no in 2..=4 {
        report(&env, &contract_id, &admin, &alice, &bob, match_no, 1, 0);
    }
    let fifth = report(&env, &contract_id, &admin, &alice, &bob, 5, 1, 0);
    assert_eq!(fifth.home_delta, 15);

    let alice_before = client(&env, &contract_id).rating(&alice);
    let bob_before = client(&env, &contract_id).rating(&bob);
    assert!(alice_before > bob_before);

    // The upset: lower-rated bob beats higher-rated alice, and the swing is
    // larger than the 16 an even matchup pays for a win.
    let upset = report(&env, &contract_id, &admin, &alice, &bob, 6, 0, 3);
    assert_eq!(upset.away_delta, -upset.home_delta);
    assert!(upset.away_delta > 16, "upset win must outpay an even win");
    assert_eq!(
        client(&env, &contract_id).rating(&alice),
        alice_before + upset.home_delta
    );
    assert_eq!(
        client(&env, &contract_id).rating(&bob),
        bob_before + upset.away_delta
    );
}

#[test]
fn repeated_pair_uses_the_updated_ratings() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);
    client(&env, &contract_id).initialize(&admin);
    let (alice, bob) = registered_pair(&env, &contract_id);

    let first = report(&env, &contract_id, &admin, &alice, &bob, 1, 2, 0);
    // Second fixture of the same pair: alice is now the favourite, so the
    // same outcome pays a smaller swing.
    let second = report(&env, &contract_id, &admin, &alice, &bob, 2, 2, 0);
    assert_eq!(first.home_delta, 16);
    assert_eq!(second.home_delta, 15);
    assert_ne!(first.home_delta, second.home_delta);

    assert_eq!(client(&env, &contract_id).matches_played(&alice, &bob), 2);
    assert_eq!(client(&env, &contract_id).player(&alice).matches, 2);
    // Sum of the mirrored deltas still conserves the total rating.
    assert_eq!(
        client(&env, &contract_id).rating(&alice) + client(&env, &contract_id).rating(&bob),
        2 * START_RATING
    );
}

// ── Duplicate results ───────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "duplicate result for this fixture")]
fn duplicate_result_is_rejected() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);
    client(&env, &contract_id).initialize(&admin);
    let (alice, bob) = registered_pair(&env, &contract_id);

    report(&env, &contract_id, &admin, &alice, &bob, 1, 1, 0);
    // Fixture 1 already exists: re-reporting it must be rejected rather
    // than silently double-counting the rating update.
    report(&env, &contract_id, &admin, &alice, &bob, 1, 3, 2);
}

#[test]
#[should_panic(expected = "match number out of order")]
fn skipping_a_fixture_is_rejected() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);
    client(&env, &contract_id).initialize(&admin);
    let (alice, bob) = registered_pair(&env, &contract_id);

    report(&env, &contract_id, &admin, &alice, &bob, 1, 1, 0);
    report(&env, &contract_id, &admin, &alice, &bob, 3, 1, 0);
}

// ── Pause / unpause ─────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "season is paused")]
fn paused_season_rejects_results() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);
    let c = client(&env, &contract_id);
    c.initialize(&admin);
    let (alice, bob) = registered_pair(&env, &contract_id);

    c.pause(&admin);
    assert!(c.is_paused());
    report(&env, &contract_id, &admin, &alice, &bob, 1, 1, 0);
}

#[test]
#[should_panic(expected = "season is paused")]
fn paused_season_rejects_registration() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);
    let c = client(&env, &contract_id);
    c.initialize(&admin);
    c.pause(&admin);

    c.register_player(&Address::generate(&env));
}

#[test]
fn unpause_resumes_reporting_and_registration() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);
    let c = client(&env, &contract_id);
    c.initialize(&admin);
    let (alice, bob) = registered_pair(&env, &contract_id);

    c.pause(&admin);
    assert!(c.is_paused());
    c.unpause(&admin);
    assert!(!c.is_paused());

    // The season continues exactly where it left off: fixture 1 is still
    // the next one to report.
    let result = report(&env, &contract_id, &admin, &alice, &bob, 1, 1, 0);
    assert_eq!(result.home_delta, 16);

    let latecomer = Address::generate(&env);
    c.register_player(&latecomer);
    assert_eq!(c.rating(&latecomer), START_RATING);
}

// ── Authorization ───────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "only the season admin can report results")]
fn non_admin_cannot_report_results() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);
    let interloper = Address::generate(&env);
    let c = client(&env, &contract_id);
    c.initialize(&admin);
    let (alice, bob) = registered_pair(&env, &contract_id);

    c.report_result(&interloper, &alice, &bob, &1, &1, &0);
}

#[test]
#[should_panic(expected = "only the season admin can pause")]
fn non_admin_cannot_pause() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);
    let interloper = Address::generate(&env);
    let c = client(&env, &contract_id);
    c.initialize(&admin);

    c.pause(&interloper);
}

#[test]
#[should_panic(expected = "player already registered")]
fn duplicate_registration_is_rejected() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);
    let c = client(&env, &contract_id);
    c.initialize(&admin);
    let player = Address::generate(&env);

    c.register_player(&player);
    c.register_player(&player);
}

#[test]
#[should_panic(expected = "a team cannot play itself")]
fn self_fixture_is_rejected() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);
    let c = client(&env, &contract_id);
    c.initialize(&admin);
    let (alice, _bob) = registered_pair(&env, &contract_id);

    c.report_result(&admin, &alice, &alice, &1, &1, &0);
}

// ── Standings ───────────────────────────────────────────────────────────────

#[test]
fn standings_rank_players_by_rating() {
    let (env, contract_id) = setup();
    let admin = Address::generate(&env);
    let c = client(&env, &contract_id);
    c.initialize(&admin);
    let (alice, bob) = registered_pair(&env, &contract_id);
    let carol = Address::generate(&env);
    c.register_player(&carol);

    report(&env, &contract_id, &admin, &alice, &bob, 1, 1, 0);

    let table = c.standings();
    assert_eq!(table.len(), 3);
    assert_eq!(table.get(0).unwrap().0, alice);
    assert_eq!(table.get(0).unwrap().1, 1_016);
    // Bob lost and carol never played, so bob (984) ranks below the
    // still-unbeaten-but-unplayed carol (1000).
    assert_eq!(table.get(1).unwrap().0, carol);
    assert_eq!(table.get(2).unwrap().0, bob);
}
