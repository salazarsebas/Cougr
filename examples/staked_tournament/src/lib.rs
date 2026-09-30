#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Vec};

#[cfg(test)]
mod test;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    State,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchRecord {
    pub player_a: Option<Address>,
    pub player_b: Option<Address>,
    pub next_match: Option<u32>,
    pub advances_to_a: bool,
    pub paid_a: bool,
    pub paid_b: bool,
    pub winner: Option<Address>,
    pub deadline: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TournamentState {
    pub token: Address,
    pub stake: i128,
    pub timeout_seconds: u64,
    pub matches: Vec<MatchRecord>,
    pub champion: Option<Address>,
}

fn load_state(env: &Env) -> TournamentState {
    env.storage()
        .instance()
        .get(&DataKey::State)
        .expect("tournament is not initialized")
}

fn save_state(env: &Env, state: &TournamentState) {
    env.storage().instance().set(&DataKey::State, state);
}

fn match_at(state: &TournamentState, match_id: u32) -> MatchRecord {
    state.matches.get(match_id).expect("match does not exist")
}

fn record_winner(env: &Env, state: &mut TournamentState, match_id: u32, winner: Address) {
    let mut match_record = match_at(state, match_id);
    if match_record.winner.is_some() {
        panic!("match is already settled");
    }
    match_record.winner = Some(winner.clone());
    state.matches.set(match_id, match_record.clone());

    if let Some(next_match_id) = match_record.next_match {
        let mut next_match = match_at(state, next_match_id);
        if match_record.advances_to_a {
            if next_match.player_a.is_some() {
                panic!("next-round slot is already filled");
            }
            next_match.player_a = Some(winner);
        } else {
            if next_match.player_b.is_some() {
                panic!("next-round slot is already filled");
            }
            next_match.player_b = Some(winner);
        }
        if next_match.player_a.is_some() && next_match.player_b.is_some() {
            next_match.deadline = env
                .ledger()
                .timestamp()
                .checked_add(state.timeout_seconds)
                .expect("match deadline overflow");
        }
        state.matches.set(next_match_id, next_match);
    } else {
        state.champion = Some(winner);
    }

    save_state(env, state);
}

#[contract]
pub struct StakedTournament;

#[contractimpl]
impl StakedTournament {
    pub fn initialize(
        env: Env,
        creator: Address,
        entrants: Vec<Address>,
        token_address: Address,
        stake: i128,
        timeout_seconds: u64,
    ) {
        if env.storage().instance().has(&DataKey::State) {
            panic!("tournament is already initialized");
        }
        creator.require_auth();

        let entrant_count = entrants.len();
        if entrant_count != 4 && entrant_count != 8 {
            panic!("tournament requires four or eight entrants");
        }
        if stake <= 0 || stake.checked_mul(2).is_none() {
            panic!("stake must be positive and safely payable");
        }
        if timeout_seconds == 0 {
            panic!("timeout must be positive");
        }
        for left in 0..entrant_count {
            for right in left + 1..entrant_count {
                if entrants.get(left) == entrants.get(right) {
                    panic!("entrants must be unique");
                }
            }
        }

        let mut matches = Vec::new(&env);
        let mut round_offset = 0;
        let mut round_match_count = entrant_count / 2;
        while round_match_count > 0 {
            for local_match in 0..round_match_count {
                let is_first_round = round_offset == 0;
                let player_a = if is_first_round {
                    Some(entrants.get(local_match * 2).unwrap())
                } else {
                    None
                };
                let player_b = if is_first_round {
                    Some(entrants.get(local_match * 2 + 1).unwrap())
                } else {
                    None
                };
                let next_match = if round_match_count == 1 {
                    None
                } else {
                    Some(round_offset + round_match_count + local_match / 2)
                };
                matches.push_back(MatchRecord {
                    player_a,
                    player_b,
                    next_match,
                    advances_to_a: local_match % 2 == 0,
                    paid_a: false,
                    paid_b: false,
                    winner: None,
                    deadline: if is_first_round {
                        env.ledger()
                            .timestamp()
                            .checked_add(timeout_seconds)
                            .expect("match deadline overflow")
                    } else {
                        0
                    },
                });
            }
            round_offset += round_match_count;
            round_match_count /= 2;
        }

        save_state(
            &env,
            &TournamentState {
                token: token_address,
                stake,
                timeout_seconds,
                matches,
                champion: None,
            },
        );
    }

    pub fn stake_match(env: Env, player: Address, match_id: u32) {
        player.require_auth();
        let mut state = load_state(&env);
        let mut match_record = match_at(&state, match_id);
        if match_record.winner.is_some() {
            panic!("match is already settled");
        }
        if env.ledger().timestamp() >= match_record.deadline {
            panic!("match staking deadline has passed");
        }

        let paid = if match_record.player_a.as_ref() == Some(&player) {
            &mut match_record.paid_a
        } else if match_record.player_b.as_ref() == Some(&player) {
            &mut match_record.paid_b
        } else {
            panic!("player is not assigned to this match");
        };
        if *paid {
            panic!("player has already staked for this match");
        }

        token::Client::new(&env, &state.token).transfer(
            &player,
            &env.current_contract_address(),
            &state.stake,
        );
        *paid = true;
        state.matches.set(match_id, match_record);
        save_state(&env, &state);
    }

    pub fn resolve_match(env: Env, winner: Address, match_id: u32) {
        winner.require_auth();
        let mut state = load_state(&env);
        let match_record = match_at(&state, match_id);
        if match_record.winner.is_some() {
            panic!("match is already settled");
        }
        if !match_record.paid_a || !match_record.paid_b {
            panic!("both players must stake before settlement");
        }
        if match_record.player_a.as_ref() != Some(&winner)
            && match_record.player_b.as_ref() != Some(&winner)
        {
            panic!("winner is not assigned to this match");
        }

        let payout = state.stake * 2;
        token::Client::new(&env, &state.token).transfer(
            &env.current_contract_address(),
            &winner,
            &payout,
        );
        record_winner(&env, &mut state, match_id, winner);
    }

    pub fn forfeit_no_show(env: Env, match_id: u32) {
        let mut state = load_state(&env);
        let match_record = match_at(&state, match_id);
        if match_record.winner.is_some() {
            panic!("match is already settled");
        }
        if env.ledger().timestamp() < match_record.deadline {
            panic!("match deadline has not passed");
        }
        if match_record.paid_a == match_record.paid_b {
            panic!("exactly one player must have staked");
        }

        let winner = if match_record.paid_a {
            match_record.player_a.unwrap()
        } else {
            match_record.player_b.unwrap()
        };
        token::Client::new(&env, &state.token).transfer(
            &env.current_contract_address(),
            &winner,
            &state.stake,
        );
        record_winner(&env, &mut state, match_id, winner);
    }

    pub fn get_match(env: Env, match_id: u32) -> MatchRecord {
        match_at(&load_state(&env), match_id)
    }

    pub fn champion(env: Env) -> Option<Address> {
        load_state(&env).champion
    }
}
