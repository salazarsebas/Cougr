//! season_ladder - a competitive season driven by the Cougr standards layer.
//!
//! Players register themselves, the season admin records head-to-head
//! results, and an integer-only rating is updated after every result.
//! Administration (who may report results, who may pause) comes from
//! `cougr_core::standards::{Ownable, Pausable}` instead of a private admin
//! flag: while the season is paused, registration and result reporting are
//! frozen until the owner unpauses.

#![no_std]

use cougr_core::standards::{Ownable, Pausable};
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Vec};

/// Rating every player starts the season with.
pub const START_RATING: i64 = 1_000;

/// Elo-style K-factor: the maximum swing for a single result when the
/// outcome is completely unexpected.
pub const K_FACTOR: i64 = 32;

/// Ratings never fall below this, so expected-score math stays well-defined.
pub const RATING_FLOOR: i64 = 100;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerRecord {
    pub rating: i64,
    pub matches: u32,
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchResult {
    pub home: Address,
    pub away: Address,
    pub home_goals: u32,
    pub away_goals: u32,
    pub home_delta: i64,
    pub away_delta: i64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum DataKey {
    Player(Address),
    Fixture(Address, Address, u32),
    PairMatches(Address, Address),
    Roster,
}

/// The season admin, shared by every guarded entrypoint.
fn ownable() -> Ownable {
    Ownable::new(symbol_short!("ladder"))
}

/// The emergency stop that freezes registration and result reporting.
fn pausable() -> Pausable {
    Pausable::new(symbol_short!("ladder"))
}

fn roster(env: &Env) -> Vec<Address> {
    env.storage()
        .instance()
        .get(&DataKey::Roster)
        .unwrap_or_else(|| Vec::new(env))
}

fn matches_played(env: &Env, home: &Address, away: &Address) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::PairMatches(home.clone(), away.clone()))
        .unwrap_or(0)
}

/// Expected score for `rating` against `opponent`, in parts-per-million
/// (1_000_000 = certain win, 0 = certain loss). Integer-only Elo
/// expectation: `1_000_000 * rating / (rating + opponent)`.
fn expected_score_ppm(rating: i64, opponent: i64) -> i64 {
    1_000_000 * rating / (rating + opponent)
}

/// Integer-only rating swing for a player on `score_ppm`. Truncates toward
/// zero, so the magnitude never exceeds `K_FACTOR`.
fn rating_delta(rating: i64, opponent: i64, score_ppm: i64) -> i64 {
    (K_FACTOR * (score_ppm - expected_score_ppm(rating, opponent))) / 1_000_000
}

/// Result score in parts-per-million: win = 1_000_000, draw = 500_000,
/// loss = 0.
fn score_ppm(home_goals: u32, away_goals: u32) -> i64 {
    if home_goals > away_goals {
        1_000_000
    } else if home_goals < away_goals {
        0
    } else {
        500_000
    }
}

#[contract]
#[derive(Clone)]
pub struct SeasonLadder;

#[contractimpl]
impl SeasonLadder {
    /// One-time setup: binds the season to an admin via `Ownable`.
    pub fn initialize(env: Env, admin: Address) -> Address {
        ownable()
            .initialize(&env, &admin)
            .expect("season admin already set");
        admin
    }

    /// Self-registration at [`START_RATING`]. Frozen while paused.
    pub fn register_player(env: Env, player: Address) -> PlayerRecord {
        pausable()
            .require_not_paused(&env)
            .expect("season is paused");
        player.require_auth();
        if env
            .storage()
            .instance()
            .has(&DataKey::Player(player.clone()))
        {
            panic!("player already registered");
        }
        let record = PlayerRecord {
            rating: START_RATING,
            matches: 0,
            wins: 0,
            draws: 0,
            losses: 0,
        };
        env.storage()
            .instance()
            .set(&DataKey::Player(player.clone()), &record);
        let mut list = roster(&env);
        list.push_back(player.clone());
        env.storage().instance().set(&DataKey::Roster, &list);
        record
    }

    /// Records fixture `match_no` (1-based, per ordered pair) and updates
    /// both ratings with the integer-only rule. Owner-only, frozen while
    /// paused, and a fixture may only be reported once per match number.
    #[allow(clippy::too_many_arguments)]
    pub fn report_result(
        env: Env,
        reporter: Address,
        home: Address,
        away: Address,
        match_no: u32,
        home_goals: u32,
        away_goals: u32,
    ) -> MatchResult {
        ownable()
            .require_owner(&env, &reporter)
            .expect("only the season admin can report results");
        pausable()
            .require_not_paused(&env)
            .expect("season is paused");
        if home == away {
            panic!("a team cannot play itself");
        }
        let mut home_player: PlayerRecord = env
            .storage()
            .instance()
            .get(&DataKey::Player(home.clone()))
            .expect("player not registered");
        let mut away_player: PlayerRecord = env
            .storage()
            .instance()
            .get(&DataKey::Player(away.clone()))
            .expect("player not registered");

        let played = matches_played(&env, &home, &away);
        if match_no <= played {
            panic!("duplicate result for this fixture");
        }
        if match_no != played + 1 {
            panic!("match number out of order");
        }

        let outcome = score_ppm(home_goals, away_goals);
        let home_delta = rating_delta(home_player.rating, away_player.rating, outcome);
        let new_home_rating = (home_player.rating + home_delta).max(RATING_FLOOR);
        let new_away_rating = (away_player.rating - home_delta).max(RATING_FLOOR);
        // Report what actually applied: the floor clamp can shrink a delta.
        let applied_home_delta = new_home_rating - home_player.rating;
        let applied_away_delta = new_away_rating - away_player.rating;

        home_player.rating = new_home_rating;
        away_player.rating = new_away_rating;
        home_player.matches += 1;
        away_player.matches += 1;
        if outcome == 1_000_000 {
            home_player.wins += 1;
            away_player.losses += 1;
        } else if outcome == 0 {
            home_player.losses += 1;
            away_player.wins += 1;
        } else {
            home_player.draws += 1;
            away_player.draws += 1;
        }

        env.storage()
            .instance()
            .set(&DataKey::Player(home.clone()), &home_player);
        env.storage()
            .instance()
            .set(&DataKey::Player(away.clone()), &away_player);
        env.storage().instance().set(
            &DataKey::PairMatches(home.clone(), away.clone()),
            &(played + 1),
        );

        let result = MatchResult {
            home: home.clone(),
            away: away.clone(),
            home_goals,
            away_goals,
            home_delta: applied_home_delta,
            away_delta: applied_away_delta,
        };
        env.storage().instance().set(
            &DataKey::Fixture(home.clone(), away.clone(), match_no),
            &result,
        );
        result
    }

    /// Owner-only emergency stop. Freezes registration and reporting.
    pub fn pause(env: Env, admin: Address) {
        ownable()
            .require_owner(&env, &admin)
            .expect("only the season admin can pause");
        pausable()
            .pause(&env, &admin)
            .expect("season already paused");
    }

    /// Owner-only resume; results recorded after unpausing behave as if the
    /// pause never happened.
    pub fn unpause(env: Env, admin: Address) {
        ownable()
            .require_owner(&env, &admin)
            .expect("only the season admin can unpause");
        pausable()
            .unpause(&env, &admin)
            .expect("season is not paused");
    }

    pub fn is_paused(env: Env) -> bool {
        pausable().is_paused(&env)
    }

    pub fn owner(env: Env) -> Option<Address> {
        ownable().owner(&env)
    }

    pub fn player(env: Env, id: Address) -> PlayerRecord {
        env.storage()
            .instance()
            .get(&DataKey::Player(id))
            .expect("player not registered")
    }

    pub fn rating(env: Env, id: Address) -> i64 {
        Self::player(env, id).rating
    }

    pub fn matches_played(env: Env, home: Address, away: Address) -> u32 {
        matches_played(&env, &home, &away)
    }

    pub fn fixture(env: Env, home: Address, away: Address, match_no: u32) -> MatchResult {
        env.storage()
            .instance()
            .get(&DataKey::Fixture(home, away, match_no))
            .expect("fixture not found")
    }

    /// Standings ordered by rating, highest first. Ties keep registration
    /// order (stable).
    pub fn standings(env: Env) -> Vec<(Address, i64)> {
        let list = roster(&env);
        let mut rows: Vec<(Address, i64)> = Vec::new(&env);
        for id in list.iter() {
            let record: PlayerRecord = env
                .storage()
                .instance()
                .get(&DataKey::Player(id.clone()))
                .expect("player not registered");
            rows.push_back((id, record.rating));
        }
        // Insertion sort: the roster is small in an example, and this keeps
        // the ladder deterministic without pulling in a sorting dependency.
        let len = rows.len();
        for i in 1..len {
            let mut j = i;
            while j > 0 {
                let below = rows.get(j - 1).unwrap();
                let above = rows.get(j).unwrap();
                if above.1 > below.1 {
                    rows.set(j, below);
                    rows.set(j - 1, above);
                    j -= 1;
                } else {
                    break;
                }
            }
        }
        rows
    }
}

#[cfg(test)]
mod test;
