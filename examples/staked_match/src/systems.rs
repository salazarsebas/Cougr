use soroban_sdk::Address;

use crate::components::{MatchEscrow, MatchState};

/// Amount owed to `caller` for a match that has reached a payout state.
///
/// Returns `None` when the caller has no claim, which covers a stranger, a
/// loser, and a participant who has already been paid. Callers treat `None` as
/// an error rather than a zero transfer so a second withdrawal cannot succeed
/// silently.
pub fn payout_for(escrow: &MatchEscrow, caller: &Address) -> Option<i128> {
    let is_host = escrow.host == *caller;
    let is_challenger = match &escrow.challenger {
        Some(c) => c == caller,
        None => false,
    };

    if !is_host && !is_challenger {
        return None;
    }

    if is_host && escrow.host_paid {
        return None;
    }
    if is_challenger && escrow.challenger_paid {
        return None;
    }

    match escrow.state {
        MatchState::Settled => match &escrow.winner {
            Some(w) if w == caller => Some(escrow.pot),
            _ => None,
        },
        MatchState::Drawn => Some(escrow.stake),
        MatchState::Refunded => {
            if is_host {
                Some(escrow.pot)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Records that `caller` has been paid `amount`, draining the pot by the same
/// amount so the escrow can never pay out more than was staked into it.
pub fn mark_paid(escrow: &mut MatchEscrow, caller: &Address, amount: i128) {
    if escrow.host == *caller {
        escrow.host_paid = true;
    } else {
        escrow.challenger_paid = true;
    }
    escrow.pot -= amount;
}

/// True when `candidate` is one of the two players in the match.
pub fn is_participant(escrow: &MatchEscrow, candidate: &Address) -> bool {
    if escrow.host == *candidate {
        return true;
    }
    match &escrow.challenger {
        Some(c) => c == candidate,
        None => false,
    }
}
