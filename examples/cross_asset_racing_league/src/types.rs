pub const MAX_RACERS_PER_RACE: u32 = 10;
#[allow(dead_code)]
pub const MAX_RACES_PER_SEASON: u32 = 50;
pub const BOOST_STANDARD: u32 = 1;
pub const BOOST_PREMIUM: u32 = 2;
pub const BOOST_LEGENDARY: u32 = 3;
pub const BOOST_STANDARD_COST: u32 = 10;
pub const BOOST_PREMIUM_COST: u32 = 50;
pub const BOOST_LEGENDARY_COST: u32 = 200;
pub const RACE_STATE_REGISTRATION: u32 = 0;
pub const RACE_STATE_ACTIVE: u32 = 1;
pub const RACE_STATE_COMPLETED: u32 = 2;

use soroban_sdk::{contracterror, contracttype, Address, Bytes, BytesN};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum RacingError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    UnauthorizedOwner = 3,
    RaceNotFound = 4,
    RaceAlreadyActive = 5,
    RaceFinished = 6,
    PlayerAlreadyEntered = 7,
    PlayerNotFound = 8,
    RaceMaxCapacityReached = 9,
    InvalidBoostType = 10,
    InsufficientPaymentCredits = 11,
    BoostAlreadyActive = 12,
    InvalidProofInput = 13,
    ProofVerificationFailed = 14,
    NullifierAlreadyUsed = 15,
    StandingsUpdateFailed = 16,
    InvalidRaceState = 17,
    PaymentRegistrationFailed = 18,
}

#[contracttype]
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub struct BoostState {
    pub boost_type: u32,
    pub status: u32,
    pub activation_height: u32,
}

#[contracttype]
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub struct VehicleState {
    pub speed: u32,
    pub boost_state_type: u32,
    pub boost_active: bool,
    pub penalty_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Race {
    pub race_id: u32,
    pub season_id: u32,
    pub entrants_count: u32,
    pub phase: u32,
    pub start_height: u32,
    pub duration: u32,
}

#[contracttype]
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub struct PlayerStanding {
    pub points: u128,
    pub races_completed: u32,
    pub best_finish: u32,
    pub boost_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofInput {
    pub proof: BytesN<256>,
    pub public_inputs: Bytes,
    pub commitment: BytesN<32>,
    pub race_id: u32,
    pub player_id: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    pub owner: Address,
    pub current_season: u32,
    pub current_race_id: u32,
    pub league_active: bool,
}

#[contracttype]
pub enum DataKey {
    Owner,
    CurrentSeason,
    CurrentRaceId,
    LeagueActive,
    Race(u32),
    RaceEntrants(u32),
    PlayerVehicleState(u32, u32),
    PlayerBoostState(u32, u32),
    PlayerPaymentCredits(Address),
    PlayerStanding(u32, Address),
    UsedNullifier(BytesN<32>),
    VerificationKey,
}
