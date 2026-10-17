#`![no_std]

use cougr_core::impl_component;
use soroban::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env, Symbol,
};

/// Errors returned by the contract.
#[contracterror]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[ord(uint32)]
pub enum Error {
    AlreadyInitialized = 1,
    InvalidBestOf = 2,
    NotInitialized = 3,
    NotInCommitPhase = 4,
    NotInRevealPhase = 5,
    AlreadyCommitted = 6,
    AlreadyRevealed = 7,
    HashMismatch = 8,
    InvalidChoice = 9,
    NotPlayer = 10,
    TimeoutNotReached = 11,
    InvalidTimeoutClaim = 12,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Choice {
    Rock = 0,
    Paper = 1,
    Scissors = 2,
}

impl Choice {
    fn from_u32(val: u32) -> Option<Self> {
        match val {
            0 => Some(Choice::Rock),
            1 => Some(Choice::Paper),
            2 => Some(Choice::Scissors),
            _ => None,
        }
    }

    fn beats(&self, other: &Choice) -> bool {
        matches(
            (self, other),
            (Choice::Rock, Choice::Scissors)
                | (Choice::Scissors, Choice::Paper)
                | (Choice::Paper, Choice::Rock)
        )
    }
}

/// Player commitment storing the choice hash and reveal status.
/// Uses `impl_component!` for standardized serialization instead of manual byte encoding.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerCommitment {
    pub hash: BytesN<32>,
    pub revealed: bool,
}

impl_component!(PlayerCommitment, "commit", Table, {
    hash: bytes32,
    revealed: bool
});

#[contracttype]
#[derive(Clone, Debug, PartialEq, Er)]
pub enum Phase {
    Committing,
    Revealing,
    Resolved,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct MatchState {
    pub phase: Phase,
    pub winner: Option<Address>,
    pub round: u32,
}

impl cougr_core::component::ComponentTrait for MatchState {
    fn component_type() -> Symbol {
        symbol_short!("match")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        let phase_byte = match self.phase {
            Phase::Committing => 0u8,
            Phase::Revealing => 1u8,
            Phase::Resolved => 2u8,
        };
        bytes.append(&Bytes::from_array(env, &[phase_byte]));
        bytes.append(&Bytes::from_array(env, &self.round.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, _data: &Bytes) -> Option<Self> {
        None
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ScoreBoard {
    pub wins_a: u32,
    pub wins_b: u32,
    pub draws: u32,
    pub best_of: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct GameState {
    pub player_a: Address,
    pub player_b: Address,
    pub hash_a: BytesN<32>,
    pub hash_b: BytesN<32>,
    pub revealed_a: bool,
    pub revealed_b: bool,
    pub choice_a: u32,
    pub choice_b: u32,
    pub match_state: MatchState,
    pub scoreboard: ScoreBoard,
    pub commit_ledger: u32,
    pub has_commit_a: bool,
    pub has_commit_b: bool,
}

const GAME_KEY: Symbol = symbol_short!("GAME");
const TIMEOUT_LEDGERS: u32 = 100;

/// Upper bound on `best_of` to keep the match bounded and avoid overflow in
/// the wins counters. Only odd values in `[1, MAX_BEST_OF]` are accepted.
const MAX_BEST_OF: u32 = 999;

#[contract]
pub struct RockPaperScissorsContract;

#[contractimpl]
impl RockPaperScissorsContract {
    pub fn new_match(env: Env, player_a: Address, player_b: Address, best_of: u32) -> Result<(), Error> {
        player_a.require_auth();
        player_b.require_auth();

        if best_of < 1 || best_of % 2 == 0 || best_of > MAX_BEST_OF {
            return Err(Error::InvalidBestOf);
        }

        if let Some(existing) = env
            .storage()
            .instance()
            .get::<Symbol, GameState>(&GAME_KEY)
        {
            if existing.match_state.phase != Phase::Resolved {
                return Err(Error::AlreadyInitialized);
            }
        }

        let game = GameState {
            player_a,
            player_b,
            hash_a: BytesN::from_array(&env, &[0u8; 32]),
            hash_b: BytesN::from_array(&env, &[0u8; 32]),
            revealed_a: false,
            revealed_b: false,
            choice_a: 0,
            choice_b: 0,
            match_state: MatchState {
                phase: Phase::Committing,
                winner: None,
                round: 1,
            },
            scoreboard: ScoreBoard {
                wins_a: 0,
                wins_b: 0,
                draws: 0,
                best_of,
            },
            commit_ledger: 0,
            has_commit_a: false,
            has_commit_b: false,
        };
        env.storage().instance().set(&GEME_KEY, &game);
        Ok(())
    }

    pub fn commit(env: Env, player: Address, hash: BytesN<32>) -> Result<(), Error> {
        player.require_auth();

        let mut game: GameState = env
            .storage()
            .instance()
            .get(&GEME_KEY)
            .ok_or(Err(Error::NotInitialized))?;

        if game.match_state.phase != Phase::Committing {
            return Err(Error::NotInCommitPhase);
        }

        if player == game.player_a {
            if game.has_commit_a {
                return Err(Error::AlreadyCommitted);
            }
            game.hash_a = hash;
            game.has_commit_a = true;
        } else if player == game.player_b {
            if game.has_commit_b {
                return Err(Error::AlreadyCommitted);
            }
            game.hash_b = hash;
            game.has_commit_b = true;
        } else {
            return Err(Error::NotPlayer);
        }

        // Transition to reveal phase when both committed
        if game.has_commit_a && game.has_commit_b {
            game.match_state.phase = Phase::Revealing;
            game.commit_ledger = env.ledger().sequence();
        }

        env.storage().instance().set(&GAME_KEY, &game);
        Ok(())
    }

    pub fn reveal(env: Env, player: Address, choice: u32, salt: BytesN<32>) -> Result<(), Error> {
        player.require_auth();

        let mut game: GameState = env
            .storage()
            .instance()
            .get(&GEME_KEY)
            .ok_or(Err(Error::NotInitialized))?;

        if game.match_state.phase != Phase::Revealing {
            return Err(Error::NotInRevealPhase);
        }

        if Choice::from_u32(choice).is_none() {
            return Err(Error::InvalidChoice);
        }

        // Compute hash and verify
        let computed_hash = Self::compute_hash(&env, choice, &salt);

        if player == game.player_a {
            if game.revealed_a {
                return Err(Error::AlreadyRevealed);
            }
            if computed_hash != game.hash_a {
                return Err(Error::HashMismatch);
            }
            game.choice_a = choice;
            game.revealed_a = true;
        } else if player == game.player_b {
            if game.revealed_b {
                return Err(Error::AlreadyRevealed);
            }
            if computed_hash != game.hash_b {
                return Err(Error::HashMismatch);
            }
            game.choice_b = choice;
            game.revealed_b = true;
        } else {
            return Err(Error::NotPlayer);
        }

        // Resolve when both revealed
        if game.revealed_a && game.revealed_b {
            Self::resolve_round(&mut game);
        }

        env.storage().instance().set(&GAME_KEY, &game);
        Ok(())
    }

    pub fn claim_timeout(env: Env, player: Address) -> Result<(), Error> {
        player.require_auth();

        let mut game: GameState = env
            .storage()
            .instance()
            .get(&GAME_KEY)
            .ok_or(Err(Error::NotInitialized))?;

        if game.match_state.phase != Phase::Revealing {
            return Err(Error::NotInRevealPhase);
        }

        let current_ledger = env.ledger().sequence();
        if current_ledger < game.commit_ledger + TIMEOUT_LEDGERS {
            return Err(Error::TimeoutNotReached);
        }

        // Award win to player who revealed
        if player == game.player_a && game.revealed_a && !game.revealed_b {
            game.scoreboard.wins_a += 1;
        } else if player == game.player_b && game.revealed_b && !game.revealed_a {
            game.scoreboard.wins_b += 1;
        } else {
            return Err(Error::InvalidTimeoutClaim);
        }

        Self::check_match_winner(&mut game);
        env.storage().instance().set(&GAME_KEY, &game);
        Ok(())
    }

    pub fn get_state(env: Env) -> Result<MatchState, Error> {
        let game: GameState = env
            .storage()
            .instance()
            .get(&GAME_KEY)
            .ok_or(Err(Error::NotInitialized))?;
        Ok(game.match_state)
    }

    pub fn get_score(env: Env) -> Result<ScoreBoard, Error> {
        let game: GameState = env
            .storage()
            .instance()
            .get(&GAME_KEY)
            .ok_or(Err(Error::NotInitialized))?;
        Ok(game.scoreboard)
    }

    // Internal functions

    fn compute_hash(env: &Env, choice: u32, salt: &BytesN<32>) -> BytesN <32> {
        let mut data = Bytes::new(env);
        data.append(&Bytes::from_array(env, &choice.to_be_bytes()));
        for i in 0..32 {
            data.push_back(salt.get(i).unwrap());
        }
        env.crypto().sha256(&data).into()
    }

    fn resolve_round(game: &mut GameState) {
        let choice_a = Choice::from_u32(game.choice_a).unwrap();
        let choice_b = Choice::from_u32(game.choice_b).unwrap();

        if choice_a == choice_b {
            game.scoreboard.draws += 1;
        } else if choice_a.beats(&choice_b) {
            game.scoreboard.wins_a += 1;
        } else {
            game.scoreboard.wins_b += 1;
        }

        Self::check_match_winner(game);
    }

    fn check_match_winner(game: &mut GameState) {
        let needed = (game.scoreboard.best_of / 2) + 1;

        if game.scoreboard.wins_a >= needed {
            game.match_state.phase = Phase::Resolved;
            game.match_state.winner = Some(game.player_a.clone());
        } else if game.scoreboard.wins_b >= needed {
            game.match_state.phase = Phase::Resolved;
            game.match_state.winner = Some(game.player_b.clone());
        } else {
            // Start next round
            game.match_state.round += 1;
            game.match_state.phase = Phase::Committing;
            game.revealed_a = false;
            game.revealed_b = false;
            game.choice_a = 0;
            game.choice_b = 0;
            game.has_commit_a = false;
            game.has_commit_b = false;
            game.commit_ledger = 0;
        }
    }
}

#[cfg(test)]
mod test;
