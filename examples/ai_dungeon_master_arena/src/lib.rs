#![no_std]

extern crate alloc;
use alloc::vec::Vec as RustVec;

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol, Vec,
};

// Use cougr_core for component traits and ZK types (which align with stellar-zk)
use cougr_core::zk::{experimental, Groth16Proof, Scalar, VerificationKey};

// --- Components ---

#[contracttype]
#[derive(Clone, Debug)]
pub struct RunStateComponent {
    pub player: Address,
    pub health: u32,
    pub max_health: u32,
    pub floor: u32,
    pub score: u32,
    pub finished: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct EncounterComponent {
    pub enemy_id: u32,
    pub enemy_health: u32,
    pub enemy_attack: u32,
    pub difficulty: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PremiumActionComponent {
    pub action_type: u32, // 1: Reroll, 2: Arbitration
    pub price: i128,
    pub active: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ProofStateComponent {
    pub last_verified_floor: u32,
    pub verified_hash: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardComponent {
    pub pending_rewards: i128,
}

// --- Inputs/State ---

#[contracttype]
#[derive(Clone, Debug)]
pub enum ActionInput {
    Attack = 0,
    Defend = 1,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ProofInput {
    pub proof: Groth16Proof,
    pub public_inputs: Vec<Scalar>,
    pub floor: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct GameState {
    pub run_state: RunStateComponent,
    pub encounter: Vec<EncounterComponent>,
    pub premium: Vec<PremiumActionComponent>,
    pub proof: ProofStateComponent,
    pub rewards: RewardComponent,
}

// --- Storage Keys ---

const RUN_KEY: Symbol = symbol_short!("RUN");
const ENCOUNTER_KEY: Symbol = symbol_short!("ENCNTR");
const PREMIUM_KEY: Symbol = symbol_short!("PREM");
const PROOF_KEY: Symbol = symbol_short!("PROOF");
const REWARD_KEY: Symbol = symbol_short!("REWARD");
const VK_KEY: Symbol = symbol_short!("VK");

// --- Contract ---

#[contract]
pub struct AIDungeonMasterArenaContract;

#[contractimpl]
impl AIDungeonMasterArenaContract {
    /// Initialize an arena run.
    pub fn init_run(env: Env, player: Address) {
        player.require_auth();

        let run = RunStateComponent {
            player: player.clone(),
            health: 100,
            max_health: 100,
            floor: 1,
            score: 0,
            finished: false,
        };

        let proof = ProofStateComponent {
            last_verified_floor: 0,
            verified_hash: BytesN::from_array(&env, &[0u8; 32]),
        };

        let rewards = RewardComponent { pending_rewards: 0 };

        env.storage().persistent().set(&RUN_KEY, &run);
        env.storage().persistent().set(&PROOF_KEY, &proof);
        env.storage().persistent().set(&REWARD_KEY, &rewards);

        // Generate first encounter
        Self::spawn_encounter(&env, 1);
    }

    /// Submit a turn-based action.
    pub fn submit_action(env: Env, player: Address, action: ActionInput) -> GameState {
        player.require_auth();

        let mut run: RunStateComponent = env.storage().persistent().get(&RUN_KEY).unwrap();
        if run.player != player || run.finished {
            panic!("Invalid or finished run");
        }

        let mut encounter: EncounterComponent =
            env.storage().persistent().get(&ENCOUNTER_KEY).unwrap();

        match action {
            ActionInput::Attack => {
                let damage = 25;
                encounter.enemy_health = encounter.enemy_health.saturating_sub(damage);
            }
            ActionInput::Defend => {
                // Reduces incoming damage next turn (simplified)
            }
        }

        if encounter.enemy_health == 0 {
            // Victory
            run.score += 10 * run.floor;
            run.floor += 1;
            env.storage().persistent().remove(&ENCOUNTER_KEY);

            // Check for premium action hook (x402)
            if run.floor.is_multiple_of(3) {
                let premium = PremiumActionComponent {
                    action_type: 1,   // Reroll
                    price: 1_000_000, // 0.1 XLM
                    active: true,
                };
                env.storage().persistent().set(&PREMIUM_KEY, &premium);
            } else {
                Self::spawn_encounter(&env, run.floor);
            }
        } else {
            // Enemy attacks
            let enemy_dmg = encounter.enemy_attack;
            run.health = run.health.saturating_sub(enemy_dmg);
            if run.health == 0 {
                run.finished = true;
            }
            env.storage().persistent().set(&ENCOUNTER_KEY, &encounter);
        }

        env.storage().persistent().set(&RUN_KEY, &run);
        Self::get_state(env)
    }

    /// Purchase a premium action (x402 flow).
    /// In a real x402 scenario, the client receives a 402 and pays via Stellar.
    /// This contract validates the payment or handles the transfer.
    pub fn purchase_premium_action(env: Env, player: Address, action_type: u32) -> GameState {
        player.require_auth();

        let mut premium: PremiumActionComponent =
            env.storage().persistent().get(&PREMIUM_KEY).unwrap();
        if !premium.active || premium.action_type != action_type {
            panic!("Action not available");
        }

        // Logic check: verify payment (in this example we skip actual token transfer for brevity,
        // but note that x402 usually happens before this call or as part of it).

        premium.active = false;
        env.storage().persistent().set(&PREMIUM_KEY, &premium);

        let mut run: RunStateComponent = env.storage().persistent().get(&RUN_KEY).unwrap();
        run.score += 100; // Bonus for premium
        env.storage().persistent().set(&RUN_KEY, &run);

        // Reward: skip to a harder but more rewarding floor
        Self::spawn_encounter(&env, run.floor + 2);

        Self::get_state(env)
    }

    /// Verify a proof-backed run state using stellar-zk (Groth16).
    pub fn verify_run_proof(env: Env, player: Address, proof_input: ProofInput) -> bool {
        player.require_auth();

        let vk: VerificationKey = env.storage().persistent().get(&VK_KEY).unwrap_or_else(|| {
            panic!("VK not set");
        });

        // Convert public_inputs to a slice for verification using a local Vec
        let count = (proof_input.public_inputs.len() as usize).min(4);
        let mut rust_inputs = RustVec::with_capacity(count);
        for i in 0..count {
            rust_inputs.push(proof_input.public_inputs.get_unchecked(i as u32));
        }

        // Use stellar-zk compatible verification logic from cougr-core
        let is_valid = experimental::verify_groth16(&env, &vk, &proof_input.proof, &rust_inputs)
            .unwrap_or(false);

        if is_valid {
            let mut proof_state: ProofStateComponent =
                env.storage().persistent().get(&PROOF_KEY).unwrap();
            proof_state.last_verified_floor = proof_input.floor;
            env.storage().persistent().set(&PROOF_KEY, &proof_state);

            let mut rewards: RewardComponent = env.storage().persistent().get(&REWARD_KEY).unwrap();
            rewards.pending_rewards += 500_000; // Reward for valid proof
            env.storage().persistent().set(&REWARD_KEY, &rewards);
        }

        is_valid
    }

    /// Get current arena state.
    pub fn get_state(env: Env) -> GameState {
        let run_state: RunStateComponent = env.storage().persistent().get(&RUN_KEY).unwrap();

        let mut encounter = Vec::new(&env);
        if let Some(e) = env.storage().persistent().get(&ENCOUNTER_KEY) {
            encounter.push_back(e);
        }

        let mut premium = Vec::new(&env);
        if let Some(p) = env.storage().persistent().get(&PREMIUM_KEY) {
            premium.push_back(p);
        }

        let proof = env.storage().persistent().get(&PROOF_KEY).unwrap();
        let rewards = env.storage().persistent().get(&REWARD_KEY).unwrap();

        GameState {
            run_state,
            encounter,
            premium,
            proof,
            rewards,
        }
    }

    /// Admin: Set verification key for stellar-zk proofs.
    pub fn set_vk(env: Env, vk: VerificationKey) {
        // In realistic scenario, check for admin auth
        env.storage().persistent().set(&VK_KEY, &vk);
    }

    // --- Private Systems ---

    fn spawn_encounter(env: &Env, floor: u32) {
        let encounter = EncounterComponent {
            enemy_id: 1, // Goblin
            enemy_health: 40 + (floor * 10),
            enemy_attack: 5 + floor,
            difficulty: floor,
        };
        env.storage().persistent().set(&ENCOUNTER_KEY, &encounter);
    }
}

mod test;
