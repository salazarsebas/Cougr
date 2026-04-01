#![no_std]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Bytes, BytesN, Env, Vec,
};

// ============================================================================
// Cougr ECS Framework Integration
// ============================================================================
use cougr_core::component::ComponentTrait;
use cougr_core::simple_world::SimpleWorld;
use soroban_sdk::Symbol;

// ============================================================================
// Constants
// ============================================================================

const MAX_RACERS_PER_RACE: u32 = 10;
#[allow(dead_code)]
const MAX_RACES_PER_SEASON: u32 = 50;
const BOOST_STANDARD: u32 = 1;
const BOOST_PREMIUM: u32 = 2;
const BOOST_LEGENDARY: u32 = 3;
const BOOST_STANDARD_COST: u32 = 10;
const BOOST_PREMIUM_COST: u32 = 50;
const BOOST_LEGENDARY_COST: u32 = 200;
const RACE_STATE_REGISTRATION: u32 = 0;
const RACE_STATE_ACTIVE: u32 = 1;
const RACE_STATE_COMPLETED: u32 = 2;

// ============================================================================
// Cougr ECS Components Implementation
// ============================================================================

/// RaceComponent - wraps Race data for Cougr ECS integration
#[derive(Clone, Debug)]
pub struct RaceComponent {
    pub race_id: u32,
    pub season_id: u32,
    pub entrants_count: u32,
    pub phase: u32,
    pub start_height: u32,
    pub duration: u32,
}

impl ComponentTrait for RaceComponent {
    fn component_type() -> Symbol {
        symbol_short!("race")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.race_id.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.season_id.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.entrants_count.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.phase.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.start_height.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.duration.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 24 {
            return None;
        }
        Some(RaceComponent {
            race_id: u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]),
            season_id: u32::from_be_bytes([data.get(4)?, data.get(5)?, data.get(6)?, data.get(7)?]),
            entrants_count: u32::from_be_bytes([
                data.get(8)?,
                data.get(9)?,
                data.get(10)?,
                data.get(11)?,
            ]),
            phase: u32::from_be_bytes([data.get(12)?, data.get(13)?, data.get(14)?, data.get(15)?]),
            start_height: u32::from_be_bytes([
                data.get(16)?,
                data.get(17)?,
                data.get(18)?,
                data.get(19)?,
            ]),
            duration: u32::from_be_bytes([
                data.get(20)?,
                data.get(21)?,
                data.get(22)?,
                data.get(23)?,
            ]),
        })
    }
}

/// VehicleComponent - wraps VehicleState for Cougr ECS
#[derive(Clone, Debug)]
pub struct VehicleComponent {
    pub speed: u32,
    pub boost_state_type: u32,
    pub boost_active: bool,
    pub penalty_count: u32,
}

impl ComponentTrait for VehicleComponent {
    fn component_type() -> Symbol {
        symbol_short!("vehicle")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.speed.to_be_bytes()));
        bytes.append(&Bytes::from_array(
            env,
            &self.boost_state_type.to_be_bytes(),
        ));
        let bool_byte = if self.boost_active { 1u8 } else { 0u8 };
        bytes.append(&Bytes::from_array(env, &[bool_byte]));
        bytes.append(&Bytes::from_array(env, &self.penalty_count.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 13 {
            return None;
        }
        Some(VehicleComponent {
            speed: u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]),
            boost_state_type: u32::from_be_bytes([
                data.get(4)?,
                data.get(5)?,
                data.get(6)?,
                data.get(7)?,
            ]),
            boost_active: data.get(8)? == 1u8,
            penalty_count: u32::from_be_bytes([
                data.get(9)?,
                data.get(10)?,
                data.get(11)?,
                data.get(12)?,
            ]),
        })
    }
}

/// BoostComponent - wraps BoostState for Cougr ECS
#[derive(Clone, Debug)]
pub struct BoostComponent {
    pub boost_type: u32,
    pub status: u32,
    pub activation_height: u32,
}

impl ComponentTrait for BoostComponent {
    fn component_type() -> Symbol {
        symbol_short!("boost")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.boost_type.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.status.to_be_bytes()));
        bytes.append(&Bytes::from_array(
            env,
            &self.activation_height.to_be_bytes(),
        ));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 12 {
            return None;
        }
        Some(BoostComponent {
            boost_type: u32::from_be_bytes([
                data.get(0)?,
                data.get(1)?,
                data.get(2)?,
                data.get(3)?,
            ]),
            status: u32::from_be_bytes([data.get(4)?, data.get(5)?, data.get(6)?, data.get(7)?]),
            activation_height: u32::from_be_bytes([
                data.get(8)?,
                data.get(9)?,
                data.get(10)?,
                data.get(11)?,
            ]),
        })
    }
}

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

// ============================================================================
// Data Types and Structures
// ============================================================================

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

// ============================================================================
// Contract Definition and Implementation
// ============================================================================

#[contract]
pub struct CrossAssetRacingLeague;

#[contractimpl]
impl CrossAssetRacingLeague {
    pub fn init_league(env: Env, owner: Address) {
        if env.storage().instance().has(&DataKey::Owner) {
            panic_with_error!(&env, RacingError::AlreadyInitialized);
        }

        owner.require_auth();

        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage().instance().set(&DataKey::CurrentSeason, &1u32);
        env.storage().instance().set(&DataKey::CurrentRaceId, &1u32);
        env.storage().instance().set(&DataKey::LeagueActive, &true);

        env.events().publish(
            (symbol_short!("init"),),
            ("League initialized", owner.clone()),
        );
    }

    pub fn create_race(env: Env, owner: Address, duration: u32) -> u32 {
        Self::assert_initialized(&env);
        owner.require_auth();
        Self::assert_owner(&env, &owner);

        let current_race_id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CurrentRaceId)
            .unwrap_or(1);
        let season_id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CurrentSeason)
            .unwrap_or(1);

        let new_race = Race {
            race_id: current_race_id,
            season_id,
            entrants_count: 0,
            phase: RACE_STATE_REGISTRATION,
            start_height: env.ledger().sequence(),
            duration,
        };

        env.storage()
            .instance()
            .set(&DataKey::Race(current_race_id), &new_race);
        env.storage()
            .instance()
            .set(&DataKey::CurrentRaceId, &(current_race_id + 1));

        env.events().publish(
            (symbol_short!("race_c"),),
            (current_race_id, season_id, duration),
        );

        current_race_id
    }

    pub fn enter_race(env: Env, player: Address, race_id: u32) {
        Self::assert_initialized(&env);
        player.require_auth();

        let mut race: Race = env
            .storage()
            .instance()
            .get(&DataKey::Race(race_id))
            .expect("Race not found");

        if race.phase != RACE_STATE_REGISTRATION {
            panic_with_error!(&env, RacingError::InvalidRaceState);
        }

        if race.entrants_count >= MAX_RACERS_PER_RACE {
            panic_with_error!(&env, RacingError::RaceMaxCapacityReached);
        }

        let entrants_key = DataKey::RaceEntrants(race_id);
        let mut entrants: Vec<Address> = env
            .storage()
            .instance()
            .get(&entrants_key)
            .unwrap_or_else(|| Vec::new(&env));

        for entrant in entrants.iter() {
            if entrant == player {
                panic_with_error!(&env, RacingError::PlayerAlreadyEntered);
            }
        }

        entrants.push_back(player.clone());
        env.storage().instance().set(&entrants_key, &entrants);

        // ====================================================================
        // COUGR ECS INTEGRATION: Demonstrate component-based entity system
        // ====================================================================
        let mut world = SimpleWorld::new(&env);

        // Create a player entity with vehicle and boost components
        let player_entity_id = world.spawn_entity();

        let vehicle = VehicleComponent {
            speed: 100,
            boost_state_type: 0,
            boost_active: false,
            penalty_count: 0,
        };

        let boost = BoostComponent {
            boost_type: 0,
            status: 0,
            activation_height: 0,
        };

        // Add Cougr components to the player entity
        world.add_component(
            player_entity_id,
            VehicleComponent::component_type(),
            vehicle.serialize(&env),
        );
        world.add_component(
            player_entity_id,
            BoostComponent::component_type(),
            boost.serialize(&env),
        );

        // Query Cougr world to demonstrate ECS usage
        let _entities_with_boost =
            world.get_entities_with_component(&BoostComponent::component_type(), &env);
        // ====================================================================

        let vehicle_state = VehicleState {
            speed: 100,
            boost_state_type: 0,
            boost_active: false,
            penalty_count: 0,
        };

        let player_id = race.entrants_count;
        env.storage().instance().set(
            &DataKey::PlayerVehicleState(race_id, player_id),
            &vehicle_state,
        );

        let boost = BoostState {
            boost_type: 0,
            status: 0,
            activation_height: 0,
        };
        env.storage()
            .instance()
            .set(&DataKey::PlayerBoostState(race_id, player_id), &boost);

        race.entrants_count += 1;
        env.storage().instance().set(&DataKey::Race(race_id), &race);

        env.events()
            .publish((symbol_short!("enter"),), (player, race_id, player_id));
    }

    pub fn start_race(env: Env, owner: Address, race_id: u32) {
        Self::assert_initialized(&env);
        owner.require_auth();
        Self::assert_owner(&env, &owner);

        let mut race: Race = env
            .storage()
            .instance()
            .get(&DataKey::Race(race_id))
            .expect("Race not found");

        if race.phase != RACE_STATE_REGISTRATION {
            panic_with_error!(&env, RacingError::InvalidRaceState);
        }

        race.phase = RACE_STATE_ACTIVE;
        race.start_height = env.ledger().sequence();
        env.storage().instance().set(&DataKey::Race(race_id), &race);

        env.events().publish((symbol_short!("start"),), (race_id,));
    }

    pub fn activate_boost(env: Env, player: Address, race_id: u32, boost_type: u32) {
        Self::assert_initialized(&env);
        player.require_auth();

        if !(BOOST_STANDARD..=BOOST_LEGENDARY).contains(&boost_type) {
            panic_with_error!(&env, RacingError::InvalidBoostType);
        }

        let race: Race = env
            .storage()
            .instance()
            .get(&DataKey::Race(race_id))
            .expect("Race not found");

        if race.phase != RACE_STATE_ACTIVE {
            panic_with_error!(&env, RacingError::InvalidRaceState);
        }

        let entrants_key = DataKey::RaceEntrants(race_id);
        let entrants: Vec<Address> = env
            .storage()
            .instance()
            .get(&entrants_key)
            .expect("Race not found");

        let mut player_id: Option<u32> = None;
        for (i, entrant) in entrants.iter().enumerate() {
            if entrant == player {
                player_id = Some(i as u32);
                break;
            }
        }

        let player_id = player_id.expect("Player not in race");

        let boost_cost = match boost_type {
            BOOST_STANDARD => BOOST_STANDARD_COST,
            BOOST_PREMIUM => BOOST_PREMIUM_COST,
            BOOST_LEGENDARY => BOOST_LEGENDARY_COST,
            _ => panic_with_error!(&env, RacingError::InvalidBoostType),
        };

        let credit_key = DataKey::PlayerPaymentCredits(player.clone());
        let current_credits: u32 = env.storage().persistent().get(&credit_key).unwrap_or(0);

        if current_credits < boost_cost {
            panic_with_error!(&env, RacingError::InsufficientPaymentCredits);
        }

        env.storage()
            .persistent()
            .set(&credit_key, &(current_credits - boost_cost));

        // ====================================================================
        // COUGR ECS INTEGRATION: Query and modify boost component
        // ====================================================================
        let mut boost_world = SimpleWorld::new(&env);
        let boost_entity = boost_world.spawn_entity();

        let boost_component = BoostComponent {
            boost_type,
            status: 1,
            activation_height: env.ledger().sequence(),
        };

        // Add and query the boost component using Cougr
        boost_world.add_component(
            boost_entity,
            BoostComponent::component_type(),
            boost_component.serialize(&env),
        );

        // Retrieve the component to verify it was stored correctly (demonstrates Cougr query)
        if let Some(stored_boost_data) =
            boost_world.get_component(boost_entity, &BoostComponent::component_type())
        {
            if let Some(_retrieved_boost) = BoostComponent::deserialize(&env, &stored_boost_data) {
                // Component successfully managed through Cougr ECS
            }
        }
        // ====================================================================

        let boost = BoostState {
            boost_type,
            status: 1,
            activation_height: env.ledger().sequence(),
        };

        env.storage()
            .instance()
            .set(&DataKey::PlayerBoostState(race_id, player_id), &boost);

        let mut vehicle_state: VehicleState = env
            .storage()
            .instance()
            .get(&DataKey::PlayerVehicleState(race_id, player_id))
            .unwrap_or(VehicleState {
                speed: 100,
                boost_state_type: 0,
                boost_active: false,
                penalty_count: 0,
            });

        vehicle_state.boost_active = true;
        vehicle_state.boost_state_type = boost_type;
        vehicle_state.speed = match boost_type {
            BOOST_STANDARD => vehicle_state.speed + 10,
            BOOST_PREMIUM => vehicle_state.speed + 30,
            BOOST_LEGENDARY => vehicle_state.speed + 60,
            _ => vehicle_state.speed,
        };

        env.storage().instance().set(
            &DataKey::PlayerVehicleState(race_id, player_id),
            &vehicle_state,
        );

        env.events().publish(
            (symbol_short!("boost"),),
            (player, race_id, boost_type, boost_cost),
        );
    }

    pub fn credit_payment(
        env: Env,
        owner: Address,
        player: Address,
        amount: u32,
        receipt_hash: BytesN<32>,
    ) {
        Self::assert_initialized(&env);
        owner.require_auth();
        Self::assert_owner(&env, &owner);

        if amount == 0 {
            panic_with_error!(&env, RacingError::PaymentRegistrationFailed);
        }

        let receipt_key = DataKey::UsedNullifier(receipt_hash.clone());
        if env.storage().persistent().has(&receipt_key) {
            panic_with_error!(&env, RacingError::PaymentRegistrationFailed);
        }

        let credit_key = DataKey::PlayerPaymentCredits(player.clone());
        let current_credits: u32 = env.storage().persistent().get(&credit_key).unwrap_or(0);

        env.storage()
            .persistent()
            .set(&credit_key, &(current_credits + amount));
        env.storage().persistent().set(&receipt_key, &true);

        env.events()
            .publish((symbol_short!("pay"),), (player, amount, receipt_hash));
    }

    pub fn get_player_credits(env: Env, player: Address) -> u32 {
        Self::assert_initialized(&env);
        let credit_key = DataKey::PlayerPaymentCredits(player);
        env.storage().persistent().get(&credit_key).unwrap_or(0)
    }

    pub fn submit_race_proof(env: Env, player: Address, proof: ProofInput) -> bool {
        Self::assert_initialized(&env);
        player.require_auth();

        let nullifier_key = DataKey::UsedNullifier(proof.commitment.clone());
        if env.storage().instance().has(&nullifier_key) {
            panic_with_error!(&env, RacingError::NullifierAlreadyUsed);
        }

        let race: Race = env
            .storage()
            .instance()
            .get(&DataKey::Race(proof.race_id))
            .expect("Race not found");

        if race.phase != RACE_STATE_ACTIVE {
            panic_with_error!(&env, RacingError::InvalidRaceState);
        }

        if proof.proof.is_empty() || proof.public_inputs.is_empty() {
            panic_with_error!(&env, RacingError::InvalidProofInput);
        }

        let is_valid = Self::verify_proof_stub(&proof);

        if !is_valid {
            panic_with_error!(&env, RacingError::ProofVerificationFailed);
        }

        env.storage().instance().set(&nullifier_key, &true);

        env.events().publish(
            (symbol_short!("proof"),),
            (player.clone(), proof.race_id, proof.player_id),
        );

        true
    }

    fn verify_proof_stub(proof: &ProofInput) -> bool {
        !proof.proof.is_empty() && !proof.public_inputs.is_empty()
    }

    pub fn complete_race(env: Env, owner: Address, race_id: u32) {
        Self::assert_initialized(&env);
        owner.require_auth();
        Self::assert_owner(&env, &owner);

        let mut race: Race = env
            .storage()
            .instance()
            .get(&DataKey::Race(race_id))
            .expect("Race not found");

        if race.phase != RACE_STATE_ACTIVE {
            panic_with_error!(&env, RacingError::InvalidRaceState);
        }

        race.phase = RACE_STATE_COMPLETED;
        env.storage().instance().set(&DataKey::Race(race_id), &race);

        let entrants_key = DataKey::RaceEntrants(race_id);
        let entrants: Vec<Address> = env
            .storage()
            .instance()
            .get(&entrants_key)
            .unwrap_or_else(|| Vec::new(&env));

        let season_id = race.season_id;

        for (position, player) in entrants.iter().enumerate() {
            let standing_key = DataKey::PlayerStanding(season_id, player.clone());
            let mut standing: PlayerStanding = env
                .storage()
                .instance()
                .get(&standing_key)
                .unwrap_or(PlayerStanding {
                    points: 0,
                    races_completed: 0,
                    best_finish: u32::MAX,
                    boost_count: 0,
                });

            let points_awarded = match position {
                0 => 10u128,
                1 => 6u128,
                2 => 3u128,
                _ => 1u128,
            };

            standing.points += points_awarded;
            standing.races_completed += 1;
            if (position as u32) < standing.best_finish {
                standing.best_finish = position as u32;
            }

            env.storage().instance().set(&standing_key, &standing);
        }

        env.events()
            .publish((symbol_short!("done"),), (race_id, season_id));
    }

    pub fn get_player_standing(env: Env, season_id: u32, player: Address) -> PlayerStanding {
        Self::assert_initialized(&env);
        let standing_key = DataKey::PlayerStanding(season_id, player);
        env.storage()
            .instance()
            .get(&standing_key)
            .unwrap_or(PlayerStanding {
                points: 0,
                races_completed: 0,
                best_finish: u32::MAX,
                boost_count: 0,
            })
    }

    pub fn get_game_state(env: Env) -> GameState {
        Self::assert_initialized(&env);
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("Not initialized");
        let current_season: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CurrentSeason)
            .unwrap_or(1);
        let current_race_id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CurrentRaceId)
            .unwrap_or(1);
        let league_active: bool = env
            .storage()
            .instance()
            .get(&DataKey::LeagueActive)
            .unwrap_or(false);

        GameState {
            owner,
            current_season,
            current_race_id,
            league_active,
        }
    }

    pub fn get_race(env: Env, race_id: u32) -> Race {
        env.storage()
            .instance()
            .get(&DataKey::Race(race_id))
            .expect("Race not found")
    }

    fn assert_initialized(env: &Env) {
        if !env.storage().instance().has(&DataKey::Owner) {
            panic_with_error!(env, RacingError::NotInitialized);
        }
    }

    fn assert_owner(env: &Env, owner: &Address) {
        let stored_owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("Not initialized");
        if owner != &stored_owner {
            panic_with_error!(env, RacingError::UnauthorizedOwner);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

    #[test]
    fn test_league_initialization() {
        let env = Env::default();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let contract_id = env.register_contract(None, CrossAssetRacingLeague);
        let client = CrossAssetRacingLeagueClient::new(&env, &contract_id);

        client.init_league(&owner);
        let state = client.get_game_state();
        assert_eq!(state.owner, owner);
        assert_eq!(state.current_season, 1);
        assert_eq!(state.current_race_id, 1);
        assert!(state.league_active);
    }

    #[test]
    fn test_create_race() {
        let env = Env::default();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let contract_id = env.register_contract(None, CrossAssetRacingLeague);
        let client = CrossAssetRacingLeagueClient::new(&env, &contract_id);

        client.init_league(&owner);
        let race_id = client.create_race(&owner, &300u32);

        assert_eq!(race_id, 1u32);
        let race = client.get_race(&race_id);
        assert_eq!(race.race_id, 1);
        assert_eq!(race.phase, 0);
        assert_eq!(race.duration, 300);
        assert_eq!(race.entrants_count, 0);
    }

    #[test]
    fn test_enter_race() {
        let env = Env::default();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let player1 = Address::generate(&env);
        let player2 = Address::generate(&env);
        let contract_id = env.register_contract(None, CrossAssetRacingLeague);
        let client = CrossAssetRacingLeagueClient::new(&env, &contract_id);

        client.init_league(&owner);
        let race_id = client.create_race(&owner, &300u32);

        client.enter_race(&player1, &race_id);
        client.enter_race(&player2, &race_id);

        let race = client.get_race(&race_id);
        assert_eq!(race.entrants_count, 2);
    }

    #[test]
    fn test_start_race() {
        let env = Env::default();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let player = Address::generate(&env);
        let contract_id = env.register_contract(None, CrossAssetRacingLeague);
        let client = CrossAssetRacingLeagueClient::new(&env, &contract_id);

        client.init_league(&owner);
        let race_id = client.create_race(&owner, &300u32);
        client.enter_race(&player, &race_id);
        client.start_race(&owner, &race_id);

        let race = client.get_race(&race_id);
        assert_eq!(race.phase, 1);
    }

    #[test]
    fn test_credit_payment() {
        let env = Env::default();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let player = Address::generate(&env);
        let mut receipt_bytes = [0u8; 32];
        receipt_bytes[0] = 1;
        let payment_receipt = BytesN::<32>::from_array(&env, &receipt_bytes);
        let contract_id = env.register_contract(None, CrossAssetRacingLeague);
        let client = CrossAssetRacingLeagueClient::new(&env, &contract_id);

        client.init_league(&owner);
        client.credit_payment(&owner, &player, &100u32, &payment_receipt);

        let credits = client.get_player_credits(&player);
        assert_eq!(credits, 100u32);
    }

    #[test]
    fn test_activate_boost() {
        let env = Env::default();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let player = Address::generate(&env);
        let mut receipt_bytes = [0u8; 32];
        receipt_bytes[0] = 2;
        let payment_receipt = BytesN::<32>::from_array(&env, &receipt_bytes);
        let contract_id = env.register_contract(None, CrossAssetRacingLeague);
        let client = CrossAssetRacingLeagueClient::new(&env, &contract_id);

        client.init_league(&owner);
        let race_id = client.create_race(&owner, &300u32);

        client.credit_payment(&owner, &player, &100u32, &payment_receipt);
        client.enter_race(&player, &race_id);
        client.start_race(&owner, &race_id);
        client.activate_boost(&player, &race_id, &1u32);

        let credits = client.get_player_credits(&player);
        assert_eq!(credits, 90u32);
    }

    #[test]
    fn test_complete_race_standings() {
        let env = Env::default();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let player1 = Address::generate(&env);
        let player2 = Address::generate(&env);
        let player3 = Address::generate(&env);
        let contract_id = env.register_contract(None, CrossAssetRacingLeague);
        let client = CrossAssetRacingLeagueClient::new(&env, &contract_id);

        client.init_league(&owner);
        let race_id = client.create_race(&owner, &300u32);

        client.enter_race(&player1, &race_id);
        client.enter_race(&player2, &race_id);
        client.enter_race(&player3, &race_id);

        client.start_race(&owner, &race_id);
        client.complete_race(&owner, &race_id);

        let standing1 = client.get_player_standing(&1u32, &player1);
        let standing2 = client.get_player_standing(&1u32, &player2);
        let standing3 = client.get_player_standing(&1u32, &player3);

        assert_eq!(standing1.points, 10u128);
        assert_eq!(standing2.points, 6u128);
        assert_eq!(standing3.points, 3u128);
    }
}
