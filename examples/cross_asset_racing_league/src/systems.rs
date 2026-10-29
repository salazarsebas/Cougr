use crate::components::{BoostComponent, VehicleComponent};
use crate::helpers::{assert_initialized, assert_owner, verify_proof_stub};
use crate::types::{
    BoostState, DataKey, PlayerStanding, ProofInput, Race, RacingError, VehicleState,
    BOOST_LEGENDARY, BOOST_LEGENDARY_COST, BOOST_PREMIUM, BOOST_PREMIUM_COST, BOOST_STANDARD,
    BOOST_STANDARD_COST, MAX_RACERS_PER_RACE, RACE_STATE_ACTIVE, RACE_STATE_COMPLETED,
    RACE_STATE_REGISTRATION,
};
use cougr_core::component::ComponentTrait;
use cougr_core::simple_world::SimpleWorld;
use soroban_sdk::{panic_with_error, symbol_short, Address, BytesN, Env, Vec};

pub struct RaceSystem;

impl RaceSystem {
    pub fn create_race(env: &Env, owner: Address, duration: u32) -> u32 {
        assert_initialized(env);
        owner.require_auth();
        assert_owner(env, &owner);

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

    pub fn enter_race(env: &Env, player: Address, race_id: u32) {
        assert_initialized(env);
        player.require_auth();

        let mut race: Race = env
            .storage()
            .instance()
            .get(&DataKey::Race(race_id))
            .expect("Race not found");

        if race.phase != RACE_STATE_REGISTRATION {
            panic_with_error!(env, RacingError::InvalidRaceState);
        }

        if race.entrants_count >= MAX_RACERS_PER_RACE {
            panic_with_error!(env, RacingError::RaceMaxCapacityReached);
        }

        let entrants_key = DataKey::RaceEntrants(race_id);
        let mut entrants: Vec<Address> = env
            .storage()
            .instance()
            .get(&entrants_key)
            .unwrap_or_else(|| Vec::new(env));

        for entrant in entrants.iter() {
            if entrant == player {
                panic_with_error!(env, RacingError::PlayerAlreadyEntered);
            }
        }

        entrants.push_back(player.clone());
        env.storage().instance().set(&entrants_key, &entrants);

        // Cougr ECS Integration
        let mut world = SimpleWorld::new(env);
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

        world.add_component(
            player_entity_id,
            VehicleComponent::component_type(),
            vehicle.serialize(env),
        );
        world.add_component(
            player_entity_id,
            BoostComponent::component_type(),
            boost.serialize(env),
        );

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

        let boost_state = BoostState {
            boost_type: 0,
            status: 0,
            activation_height: 0,
        };
        env.storage()
            .instance()
            .set(&DataKey::PlayerBoostState(race_id, player_id), &boost_state);

        race.entrants_count += 1;
        env.storage().instance().set(&DataKey::Race(race_id), &race);

        env.events()
            .publish((symbol_short!("enter"),), (player, race_id, player_id));
    }

    pub fn start_race(env: &Env, owner: Address, race_id: u32) {
        assert_initialized(env);
        owner.require_auth();
        assert_owner(env, &owner);

        let mut race: Race = env
            .storage()
            .instance()
            .get(&DataKey::Race(race_id))
            .expect("Race not found");

        if race.phase != RACE_STATE_REGISTRATION {
            panic_with_error!(env, RacingError::InvalidRaceState);
        }

        race.phase = RACE_STATE_ACTIVE;
        race.start_height = env.ledger().sequence();
        env.storage().instance().set(&DataKey::Race(race_id), &race);

        env.events().publish((symbol_short!("start"),), (race_id,));
    }

    pub fn activate_boost(env: &Env, player: Address, race_id: u32, boost_type: u32) {
        assert_initialized(env);
        player.require_auth();

        if !(BOOST_STANDARD..=BOOST_LEGENDARY).contains(&boost_type) {
            panic_with_error!(env, RacingError::InvalidBoostType);
        }

        let race: Race = env
            .storage()
            .instance()
            .get(&DataKey::Race(race_id))
            .expect("Race not found");

        if race.phase != RACE_STATE_ACTIVE {
            panic_with_error!(env, RacingError::InvalidRaceState);
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
            _ => panic_with_error!(env, RacingError::InvalidBoostType),
        };

        let credit_key = DataKey::PlayerPaymentCredits(player.clone());
        let current_credits: u32 = env.storage().persistent().get(&credit_key).unwrap_or(0);

        if current_credits < boost_cost {
            panic_with_error!(env, RacingError::InsufficientPaymentCredits);
        }

        env.storage()
            .persistent()
            .set(&credit_key, &(current_credits - boost_cost));

        // Cougr ECS Integration
        let mut boost_world = SimpleWorld::new(env);
        let boost_entity = boost_world.spawn_entity();
        let boost_comp = BoostComponent {
            boost_type,
            status: 1,
            activation_height: env.ledger().sequence(),
        };
        boost_world.add_component(
            boost_entity,
            BoostComponent::component_type(),
            boost_comp.serialize(env),
        );

        let boost_state = BoostState {
            boost_type,
            status: 1,
            activation_height: env.ledger().sequence(),
        };
        env.storage()
            .instance()
            .set(&DataKey::PlayerBoostState(race_id, player_id), &boost_state);

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

    pub fn complete_race(env: &Env, owner: Address, race_id: u32) {
        assert_initialized(env);
        owner.require_auth();
        assert_owner(env, &owner);

        let mut race: Race = env
            .storage()
            .instance()
            .get(&DataKey::Race(race_id))
            .expect("Race not found");

        if race.phase != RACE_STATE_ACTIVE {
            panic_with_error!(env, RacingError::InvalidRaceState);
        }

        race.phase = RACE_STATE_COMPLETED;
        env.storage().instance().set(&DataKey::Race(race_id), &race);

        let entrants_key = DataKey::RaceEntrants(race_id);
        let entrants: Vec<Address> = env
            .storage()
            .instance()
            .get(&entrants_key)
            .unwrap_or_else(|| Vec::new(env));

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

    pub fn submit_race_proof(env: &Env, player: Address, proof: ProofInput) -> bool {
        assert_initialized(env);
        player.require_auth();

        let nullifier_key = DataKey::UsedNullifier(proof.commitment.clone());
        if env.storage().instance().has(&nullifier_key) {
            panic_with_error!(env, RacingError::NullifierAlreadyUsed);
        }

        let race: Race = env
            .storage()
            .instance()
            .get(&DataKey::Race(proof.race_id))
            .expect("Race not found");

        if race.phase != RACE_STATE_ACTIVE {
            panic_with_error!(env, RacingError::InvalidRaceState);
        }

        if proof.proof.is_empty() || proof.public_inputs.is_empty() {
            panic_with_error!(env, RacingError::InvalidProofInput);
        }

        let is_valid = verify_proof_stub(&proof);

        if !is_valid {
            panic_with_error!(env, RacingError::ProofVerificationFailed);
        }

        env.storage().instance().set(&nullifier_key, &true);

        env.events().publish(
            (symbol_short!("proof"),),
            (player.clone(), proof.race_id, proof.player_id),
        );

        true
    }
}

pub struct PaymentSystem;

impl PaymentSystem {
    pub fn credit_payment(
        env: &Env,
        owner: Address,
        player: Address,
        amount: u32,
        receipt_hash: BytesN<32>,
    ) {
        assert_initialized(env);
        owner.require_auth();
        assert_owner(env, &owner);

        if amount == 0 {
            panic_with_error!(env, RacingError::PaymentRegistrationFailed);
        }

        let receipt_key = DataKey::UsedNullifier(receipt_hash.clone());
        if env.storage().persistent().has(&receipt_key) {
            panic_with_error!(env, RacingError::PaymentRegistrationFailed);
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
}
