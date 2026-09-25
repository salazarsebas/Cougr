use soroban_sdk::{contracttype, Address};

/// Lifecycle of a single match escrow.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MatchState {
    Open = 0,
    Active = 1,
    Settled = 2,
    Drawn = 3,
    Refunded = 4,
}

/// Escrow record for one match.
///
/// `stake` is the per-player amount, so the pot is `stake * 2` once a
/// challenger has joined. Payouts decrement `pot` rather than recomputing it,
/// so a second withdrawal finds nothing left to send even if the paid flags
/// were somehow bypassed.
#[contracttype]
#[derive(Clone, Debug)]
pub struct MatchEscrow {
    pub host: Address,
    pub challenger: Option<Address>,
    pub token: Address,
    pub stake: i128,
    pub pot: i128,
    pub join_deadline: u64,
    pub state: MatchState,
    pub winner: Option<Address>,
    pub host_paid: bool,
    pub challenger_paid: bool,
}
