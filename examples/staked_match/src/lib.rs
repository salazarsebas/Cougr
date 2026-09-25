#![no_std]

pub mod components;
pub mod systems;

use cougr_core::standards::ExecutionGuard;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, token, Address, Env,
};

pub use components::{MatchEscrow, MatchState};
use systems::{is_participant, mark_paid, payout_for};

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Escrow(u64),
    NextId,
}

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Error {
    MatchNotFound = 1,
    InvalidStake = 2,
    DeadlineInPast = 3,
    MatchNotOpen = 4,
    MatchNotActive = 5,
    JoinDeadlinePassed = 6,
    JoinDeadlineNotReached = 7,
    HostCannotJoinOwnMatch = 8,
    NotAParticipant = 9,
    WinnerNotAParticipant = 10,
    NothingToWithdraw = 11,
    WithdrawalInProgress = 12,
}

#[contract]
pub struct StakedMatch;

#[contractimpl]
impl StakedMatch {
    /// Opens a match and escrows the host's stake.
    ///
    /// `join_deadline` is a ledger timestamp. Once it passes with no opponent,
    /// `claim_timeout` refunds the host.
    pub fn create_match(
        env: Env,
        host: Address,
        token_id: Address,
        stake: i128,
        join_deadline: u64,
    ) -> Result<u64, Error> {
        host.require_auth();

        if stake <= 0 {
            return Err(Error::InvalidStake);
        }
        if join_deadline <= env.ledger().timestamp() {
            return Err(Error::DeadlineInPast);
        }

        token::Client::new(&env, &token_id).transfer(
            &host,
            &env.current_contract_address(),
            &stake,
        );

        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextId)
            .unwrap_or(0u64);

        let escrow = MatchEscrow {
            host,
            challenger: None,
            token: token_id,
            stake,
            pot: stake,
            join_deadline,
            state: MatchState::Open,
            winner: None,
            host_paid: false,
            challenger_paid: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Escrow(id), &escrow);
        env.storage().instance().set(&DataKey::NextId, &(id + 1));

        Ok(id)
    }

    /// Joins an open match and escrows the challenger's matching stake.
    pub fn join(env: Env, match_id: u64, challenger: Address) -> Result<(), Error> {
        challenger.require_auth();

        let mut escrow = Self::load(&env, match_id)?;

        if escrow.state != MatchState::Open {
            return Err(Error::MatchNotOpen);
        }
        if env.ledger().timestamp() >= escrow.join_deadline {
            return Err(Error::JoinDeadlinePassed);
        }
        if escrow.host == challenger {
            return Err(Error::HostCannotJoinOwnMatch);
        }

        token::Client::new(&env, &escrow.token).transfer(
            &challenger,
            &env.current_contract_address(),
            &escrow.stake,
        );

        escrow.pot += escrow.stake;
        escrow.challenger = Some(challenger);
        escrow.state = MatchState::Active;

        Self::save(&env, match_id, &escrow);
        Ok(())
    }

    /// Records an agreed winner. Both players must authorize, so neither can
    /// declare a result the other has not accepted.
    pub fn agree_result(env: Env, match_id: u64, winner: Address) -> Result<(), Error> {
        let mut escrow = Self::load(&env, match_id)?;

        if escrow.state != MatchState::Active {
            return Err(Error::MatchNotActive);
        }

        escrow.host.require_auth();
        match &escrow.challenger {
            Some(c) => c.require_auth(),
            None => return Err(Error::MatchNotActive),
        }

        if !is_participant(&escrow, &winner) {
            return Err(Error::WinnerNotAParticipant);
        }

        escrow.state = MatchState::Settled;
        escrow.winner = Some(winner);

        Self::save(&env, match_id, &escrow);
        Ok(())
    }

    /// Records an agreed draw. Both players must authorize, and each then
    /// withdraws their own stake.
    pub fn agree_draw(env: Env, match_id: u64) -> Result<(), Error> {
        let mut escrow = Self::load(&env, match_id)?;

        if escrow.state != MatchState::Active {
            return Err(Error::MatchNotActive);
        }

        escrow.host.require_auth();
        match &escrow.challenger {
            Some(c) => c.require_auth(),
            None => return Err(Error::MatchNotActive),
        }

        escrow.state = MatchState::Drawn;

        Self::save(&env, match_id, &escrow);
        Ok(())
    }

    /// Cancels a match nobody joined and refunds the host.
    pub fn cancel(env: Env, match_id: u64) -> Result<(), Error> {
        let mut escrow = Self::load(&env, match_id)?;

        if escrow.state != MatchState::Open {
            return Err(Error::MatchNotOpen);
        }

        escrow.host.require_auth();
        escrow.state = MatchState::Refunded;

        Self::save(&env, match_id, &escrow);
        Ok(())
    }

    /// Releases a match that nobody joined before the deadline.
    ///
    /// Deliberately permissionless: the funds can only move to the host, so a
    /// third party unsticking an abandoned match costs nobody anything.
    pub fn claim_timeout(env: Env, match_id: u64) -> Result<(), Error> {
        let mut escrow = Self::load(&env, match_id)?;

        if escrow.state != MatchState::Open {
            return Err(Error::MatchNotOpen);
        }
        if env.ledger().timestamp() < escrow.join_deadline {
            return Err(Error::JoinDeadlineNotReached);
        }

        escrow.state = MatchState::Refunded;

        Self::save(&env, match_id, &escrow);
        Ok(())
    }

    /// Pays the caller whatever the settled match owes them, once.
    pub fn withdraw(env: Env, match_id: u64, caller: Address) -> Result<i128, Error> {
        caller.require_auth();

        let guard = ExecutionGuard::new(symbol_short!("withdraw"));
        guard.enter(&env).map_err(|_| Error::WithdrawalInProgress)?;

        let mut escrow = Self::load(&env, match_id)?;

        if !is_participant(&escrow, &caller) {
            return Err(Error::NotAParticipant);
        }

        let amount = payout_for(&escrow, &caller).ok_or(Error::NothingToWithdraw)?;

        mark_paid(&mut escrow, &caller, amount);
        Self::save(&env, match_id, &escrow);

        token::Client::new(&env, &escrow.token).transfer(
            &env.current_contract_address(),
            &caller,
            &amount,
        );

        guard.exit(&env).map_err(|_| Error::WithdrawalInProgress)?;

        Ok(amount)
    }

    pub fn get_match(env: Env, match_id: u64) -> Result<MatchEscrow, Error> {
        Self::load(&env, match_id)
    }

    fn load(env: &Env, match_id: u64) -> Result<MatchEscrow, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Escrow(match_id))
            .ok_or(Error::MatchNotFound)
    }

    fn save(env: &Env, match_id: u64, escrow: &MatchEscrow) {
        env.storage()
            .persistent()
            .set(&DataKey::Escrow(match_id), escrow);
    }
}

#[cfg(test)]
mod test;
