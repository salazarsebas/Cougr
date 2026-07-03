#![no_std]
#![allow(clippy::too_many_arguments)]
extern crate alloc;

mod auth;

pub mod components;
#[cfg(not(feature = "zk"))]
pub mod systems;
#[cfg(feature = "zk")]
pub mod zk;

#[cfg(test)]
mod test;

use components::{Clue, PuzzleMetadata};
use cougr_core::ops::Ownable;
#[cfg(not(feature = "zk"))]
use cougr_core::plugin::GameApp;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Env, String, Symbol, Vec,
};
#[cfg(feature = "zk")]
use soroban_sdk::{Bytes, BytesN};
use crate::auth::{authorize_session, revoke_session};

/// Errors returned by the Murdoku smart contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PuzzleError {
    InvalidGridSize = 1,
    InvalidSuspects = 2,
    InvalidSolution = 3,
    InvalidClues = 4,
    PuzzleNotFound = 5,
    Unauthorized = 6,
}

/// Result of a suspect placement or removal.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MoveResult {
    Ok = 0,
    RowConflict = 1,
    ColConflict = 2,
    CellOccupied = 3,
    GameAlreadySolved = 4,
    InvalidCoordinates = 5,
}

/// Snapshot of a player's current game progress.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerState {
    pub puzzle_id: u32,
    pub grid: Vec<u32>,
    pub solved: bool,
    pub moves: u32,
    pub total_solvers: u32,
}

/// Representation of a full Murdoku puzzle (v1 — plaintext solution).
#[cfg(not(feature = "zk"))]
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Puzzle {
    pub id: u32,
    pub creator: Address,
    pub grid_size: u32,
    pub suspects: Vec<String>,
    pub clues: Vec<Clue>,
    pub solution: Vec<u32>,
    pub metadata: PuzzleMetadata,
    pub active: bool,
}

/// Representation of a full Murdoku puzzle (ZK mode — solution commitment).
#[cfg(feature = "zk")]
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Puzzle {
    pub id: u32,
    pub creator: Address,
    pub grid_size: u32,
    pub suspects: Vec<String>,
    pub clues: Vec<Clue>,
    pub solution_commitment: BytesN<32>,
    pub metadata: PuzzleMetadata,
    pub active: bool,
}

/// Representation of a Murdoku puzzle summary (omitting the solution).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PuzzleSummary {
    pub id: u32,
    pub creator: Address,
    pub grid_size: u32,
    pub metadata: PuzzleMetadata,
    pub active: bool,
}

#[contract]
pub struct MurdokuContract;

#[contractimpl]
#[cfg(not(feature = "zk"))]
impl MurdokuContract {
    /// Validates and stores a new puzzle. Returns the assigned puzzle ID.
    pub fn submit_puzzle(
        env: Env,
        caller: Address,
        grid_size: u32,
        suspects: Vec<String>,
        clues: Vec<Clue>,
        solution: Vec<u32>,
        metadata: PuzzleMetadata,
    ) -> u32 {
        caller.require_auth();

        // 1. Run validation using ephemeral ECS
        let mut app = GameApp::new(&env);
        app.add_startup_system("validate_puzzle", systems::puzzle_validation_system);

        let entity_id = app.world_mut().spawn_entity();
        app.world_mut()
            .set_typed(&env, entity_id, &components::GridSize { size: grid_size });
        app.world_mut().set_typed(
            &env,
            entity_id,
            &components::Suspects {
                list: suspects.clone(),
            },
        );
        app.world_mut().set_typed(
            &env,
            entity_id,
            &components::Clues {
                list: clues.clone(),
            },
        );
        app.world_mut().set_typed(
            &env,
            entity_id,
            &components::Solution {
                grid: solution.clone(),
            },
        );
        app.world_mut().set_typed(
            &env,
            entity_id,
            &components::Metadata {
                meta: metadata.clone(),
            },
        );

        if app.run_startup(&env).is_err() {
            panic_with_error!(&env, PuzzleError::InvalidSolution);
        }

        // 2. Increment the puzzle counter
        let counter_key = Symbol::new(&env, "PUZZLE_COUNT");
        let mut count: u32 = env.storage().persistent().get(&counter_key).unwrap_or(0);
        count += 1;
        env.storage().persistent().set(&counter_key, &count);
        let puzzle_id = count;

        // 3. Store the puzzle definition
        let puzzle = Puzzle {
            id: puzzle_id,
            creator: caller.clone(),
            grid_size,
            suspects,
            clues,
            solution,
            metadata,
            active: true,
        };
        let puzzle_key = (Symbol::new(&env, "PUZZLE"), puzzle_id);
        env.storage().persistent().set(&puzzle_key, &puzzle);

        // 4. Set the puzzle status
        let status_key = (Symbol::new(&env, "STATUS"), puzzle_id);
        env.storage().persistent().set(&status_key, &true);

        // 5. Initialize the ownable pattern for authorization
        let ownable_id = Symbol::new(&env, &alloc::format!("puzzle_{}", puzzle_id));
        let ownable = Ownable::new(ownable_id);
        ownable.initialize(&env, &caller).unwrap();

        puzzle_id
    }

    /// Returns the full puzzle definition including clues and solution.
    pub fn get_puzzle(env: Env, puzzle_id: u32) -> Puzzle {
        let puzzle_key = (Symbol::new(&env, "PUZZLE"), puzzle_id);
        let mut puzzle: Puzzle = match env.storage().persistent().get(&puzzle_key) {
            Some(p) => p,
            None => panic_with_error!(&env, PuzzleError::PuzzleNotFound),
        };
        let status_key = (Symbol::new(&env, "STATUS"), puzzle_id);
        let active = env.storage().persistent().get(&status_key).unwrap_or(false);
        puzzle.active = active;
        puzzle
    }

    /// Returns a paginated list of puzzle summaries (no solution field).
    pub fn list_puzzles(env: Env, offset: u32, limit: u32) -> Vec<PuzzleSummary> {
        let counter_key = Symbol::new(&env, "PUZZLE_COUNT");
        let total: u32 = env.storage().persistent().get(&counter_key).unwrap_or(0);

        let mut list = Vec::new(&env);
        if offset >= total {
            return list;
        }

        let start = offset + 1;
        let end = (offset + limit).min(total);

        for id in start..=end {
            let puzzle_key = (Symbol::new(&env, "PUZZLE"), id);
            if let Some(puzzle) = env.storage().persistent().get::<_, Puzzle>(&puzzle_key) {
                let status_key = (Symbol::new(&env, "STATUS"), id);
                let active = env.storage().persistent().get(&status_key).unwrap_or(false);
                list.push_back(PuzzleSummary {
                    id: puzzle.id,
                    creator: puzzle.creator,
                    grid_size: puzzle.grid_size,
                    metadata: puzzle.metadata,
                    active,
                });
            }
        }
        list
    }

    /// Returns the total number of submitted puzzles.
    pub fn get_puzzle_count(env: Env) -> u32 {
        let counter_key = Symbol::new(&env, "PUZZLE_COUNT");
        env.storage().persistent().get(&counter_key).unwrap_or(0)
    }

    /// Allows the original creator to deactivate their puzzle. Uses ops::Ownable pattern for authorization.
    pub fn deactivate_puzzle(env: Env, caller: Address, puzzle_id: u32) {
        caller.require_auth();

        let ownable_id = Symbol::new(&env, &alloc::format!("puzzle_{}", puzzle_id));
        let ownable = Ownable::new(ownable_id);

        if ownable.require_owner(&env, &caller).is_err() {
            panic_with_error!(&env, PuzzleError::Unauthorized);
        }

        let status_key = (Symbol::new(&env, "STATUS"), puzzle_id);
        if !env.storage().persistent().has(&status_key) {
            panic_with_error!(&env, PuzzleError::PuzzleNotFound);
        }

        env.storage().persistent().set(&status_key, &false);

        // Update the cached active field inside the persistent Puzzle definition
        let puzzle_key = (Symbol::new(&env, "PUZZLE"), puzzle_id);
        if let Some(mut puzzle) = env.storage().persistent().get::<_, Puzzle>(&puzzle_key) {
            puzzle.active = false;
            env.storage().persistent().set(&puzzle_key, &puzzle);
        }
    }

    /// Initializes a new game session for the player.
    pub fn start_game(env: Env, player: Address, puzzle_id: u32) {
        player.require_auth();
        let puzzle = Self::get_puzzle(env.clone(), puzzle_id);
        if !puzzle.active {
            panic_with_error!(&env, PuzzleError::Unauthorized);
        }

        let mut app = GameApp::new(&env);
        let entity_id = app.world_mut().spawn_entity();
        
        let mut grid = Vec::new(&env);
        for _ in 0..(puzzle.grid_size * puzzle.grid_size) {
            grid.push_back(0);
        }

        app.world_mut().set_typed(&env, entity_id, &components::Board { grid });
        app.world_mut().set_typed(&env, entity_id, &components::GameStatus { solved: false });
        app.world_mut().set_typed(&env, entity_id, &components::MoveCount { count: 0 });
        
        let session_key = (Symbol::new(&env, "SESSION"), player, puzzle_id);
        let session_key = (Symbol::new(&env, "SESSION"), puzzle_id, player);
        app.save(&env, session_key);
    }

    /// Returns the player's current game state.
    pub fn get_player_state(env: Env, player: Address, puzzle_id: u32) -> PlayerState {
        let session_key = (Symbol::new(&env, "SESSION"), player.clone(), puzzle_id);
        let session_key = (Symbol::new(&env, "SESSION"), puzzle_id, player.clone());
        let app = GameApp::load(&env, session_key).expect("Game not started");
        let world = app.world();
        let entity = world.iter_entities().next().expect("No world state");
        
        let board: components::Board = world.get_typed(&env, entity.id()).unwrap();
        let status: components::GameStatus = world.get_typed(&env, entity.id()).unwrap();
        let moves: components::MoveCount = world.get_typed(&env, entity.id()).unwrap();

        PlayerState {
            puzzle_id,
            grid: board.grid,
            solved: status.solved,
            moves: moves.count,
            total_solvers: env.storage().persistent().get(&(Symbol::new(&env, "SOLVER_CNT"), puzzle_id)).unwrap_or(0),
        }
    }
}

#[contractimpl]
#[cfg(feature = "zk")]
impl MurdokuContract {
    /// Validates and stores a new puzzle with a solution commitment instead of plaintext.
    ///
    /// The creator provides a Poseidon2 hash of the solution (plus a secret salt)
    /// and a Groth16 verifier key. The contract validates puzzle structure but
    /// cannot verify the solution itself — that happens at solve time via ZK proof.
    pub fn submit_puzzle(
        env: Env,
        caller: Address,
        grid_size: u32,
        suspects: Vec<String>,
        clues: Vec<Clue>,
        solution_commitment: BytesN<32>,
        verifier_key: Bytes,
        metadata: PuzzleMetadata,
    ) -> u32 {
        caller.require_auth();

        // --- Inline validation (no plaintext solution available) ---

        // 1. Grid size must be 4 or 5
        if grid_size != 4 && grid_size != 5 {
            panic_with_error!(&env, PuzzleError::InvalidGridSize);
        }

        // 2. Suspects length must equal grid_size; each suspect must have a non-empty name
        if suspects.len() != grid_size {
            panic_with_error!(&env, PuzzleError::InvalidSuspects);
        }
        for i in 0..grid_size {
            let name = suspects.get(i).unwrap();
            if name.is_empty() {
                panic_with_error!(&env, PuzzleError::InvalidSuspects);
            }
        }

        // 3. Clues list must be non-empty
        if clues.is_empty() {
            panic_with_error!(&env, PuzzleError::InvalidClues);
        }

        // 4. Every clue must reference only valid suspect indices and valid coordinates
        for i in 0..clues.len() {
            let clue = clues.get(i).unwrap();
            if clue.row >= grid_size || clue.col >= grid_size {
                panic_with_error!(&env, PuzzleError::InvalidClues);
            }
            if clue.suspect_idx < 1 || clue.suspect_idx > grid_size {
                panic_with_error!(&env, PuzzleError::InvalidClues);
            }
        }

        // 5. Validate that the commitment is non-zero (a valid hash is never all-zero)
        let zero = BytesN::from_array(&env, &[0u8; 32]);
        if solution_commitment == zero {
            panic_with_error!(&env, PuzzleError::InvalidSolution);
        }

        // 6. Validate that the verifier key is non-empty
        if verifier_key.is_empty() {
            panic_with_error!(&env, PuzzleError::InvalidSolution);
        }

        // --- Storage ---

        // Increment the puzzle counter
        let counter_key = Symbol::new(&env, "PUZZLE_COUNT");
        let mut count: u32 = env.storage().persistent().get(&counter_key).unwrap_or(0);
        count += 1;
        env.storage().persistent().set(&counter_key, &count);
        let puzzle_id = count;

        // Store the puzzle definition (includes commitment, not solution)
        let puzzle = Puzzle {
            id: puzzle_id,
            creator: caller.clone(),
            grid_size,
            suspects,
            clues,
            solution_commitment,
            metadata,
            active: true,
        };
        let puzzle_key = (Symbol::new(&env, "PUZZLE"), puzzle_id);
        env.storage().persistent().set(&puzzle_key, &puzzle);

        // Store the verifier key separately
        zk::store_verifier_key(&env, puzzle_id, &verifier_key);

        // Set the puzzle status
        let status_key = (Symbol::new(&env, "STATUS"), puzzle_id);
        env.storage().persistent().set(&status_key, &true);

        // Initialize the ownable pattern for authorization
        let ownable_id = Symbol::new(&env, &alloc::format!("puzzle_{}", puzzle_id));
        let ownable = Ownable::new(ownable_id);
        ownable.initialize(&env, &caller).unwrap();

        puzzle_id
    }

    /// Returns the puzzle definition (without solution – only the commitment hash).
    pub fn get_puzzle(env: Env, puzzle_id: u32) -> Puzzle {
        let puzzle_key = (Symbol::new(&env, "PUZZLE"), puzzle_id);
        let mut puzzle: Puzzle = match env.storage().persistent().get(&puzzle_key) {
            Some(p) => p,
            None => panic_with_error!(&env, PuzzleError::PuzzleNotFound),
        };
        let status_key = (Symbol::new(&env, "STATUS"), puzzle_id);
        let active = env.storage().persistent().get(&status_key).unwrap_or(false);
        puzzle.active = active;
        puzzle
    }

    /// Returns a paginated list of puzzle summaries (no solution or commitment).
    pub fn list_puzzles(env: Env, offset: u32, limit: u32) -> Vec<PuzzleSummary> {
        let counter_key = Symbol::new(&env, "PUZZLE_COUNT");
        let total: u32 = env.storage().persistent().get(&counter_key).unwrap_or(0);

        let mut list = Vec::new(&env);
        if offset >= total {
            return list;
        }

        let start = offset + 1;
        let end = (offset + limit).min(total);

        for id in start..=end {
            let puzzle_key = (Symbol::new(&env, "PUZZLE"), id);
            if let Some(puzzle) = env.storage().persistent().get::<_, Puzzle>(&puzzle_key) {
                let status_key = (Symbol::new(&env, "STATUS"), id);
                let active = env.storage().persistent().get(&status_key).unwrap_or(false);
                list.push_back(PuzzleSummary {
                    id: puzzle.id,
                    creator: puzzle.creator,
                    grid_size: puzzle.grid_size,
                    metadata: puzzle.metadata,
                    active,
                });
            }
        }
        list
    }

    /// Returns the total number of submitted puzzles.
    pub fn get_puzzle_count(env: Env) -> u32 {
        let counter_key = Symbol::new(&env, "PUZZLE_COUNT");
        env.storage().persistent().get(&counter_key).unwrap_or(0)
    }

    /// Allows the original creator to deactivate their puzzle.
    pub fn deactivate_puzzle(env: Env, caller: Address, puzzle_id: u32) {
        caller.require_auth();

        let ownable_id = Symbol::new(&env, &alloc::format!("puzzle_{}", puzzle_id));
        let ownable = Ownable::new(ownable_id);

        if ownable.require_owner(&env, &caller).is_err() {
            panic_with_error!(&env, PuzzleError::Unauthorized);
        }

        let status_key = (Symbol::new(&env, "STATUS"), puzzle_id);
        if !env.storage().persistent().has(&status_key) {
            panic_with_error!(&env, PuzzleError::PuzzleNotFound);
        }

        env.storage().persistent().set(&status_key, &false);

        let puzzle_key = (Symbol::new(&env, "PUZZLE"), puzzle_id);
        if let Some(mut puzzle) = env.storage().persistent().get::<_, Puzzle>(&puzzle_key) {
            puzzle.active = false;
            env.storage().persistent().set(&puzzle_key, &puzzle);
        }
    }

    /// Submit a Groth16 proof to claim the puzzle has been solved.
    ///
    /// On valid proof: marks the player's session as solved and increments the
    /// solver count. On invalid proof: panics with `InvalidSolution`.
    pub fn submit_proof(
        env: Env,
        player: Address,
        puzzle_id: u32,
        proof: Bytes,
        public_inputs: Vec<BytesN<32>>,
    ) {
        player.require_auth();

        // Verify the puzzle exists
        let puzzle_key = (Symbol::new(&env, "PUZZLE"), puzzle_id);
        if !env.storage().persistent().has(&puzzle_key) {
            panic_with_error!(&env, PuzzleError::PuzzleNotFound);
        }

        // Verify the Groth16 proof
        let valid = zk::verify_solution_proof(&env, puzzle_id, proof, public_inputs);
        if !valid {
            panic_with_error!(&env, PuzzleError::InvalidSolution);
        }

        // Mark as solved
        zk::mark_solved(&env, puzzle_id, &player);
    }

    /// Check whether a player has solved the specified puzzle.
    pub fn is_solved(env: Env, puzzle_id: u32, player: Address) -> bool {
        zk::is_solved(&env, puzzle_id, &player)
    }
}

// ──────────────────────────────────────────────
// Session key authorization (not feature-gated)
// ──────────────────────────────────────────────
#[contractimpl]
impl MurdokuContract {
    pub fn authorize_session(
        env: Env,
        player: Address,
        puzzle_id: u32,
        session_key: soroban_sdk::BytesN<32>,
        expires_at_ledger: u32,
    ) {
        authorize_session(env, player, puzzle_id, session_key, expires_at_ledger)
    }

    pub fn revoke_session(env: Env, player: Address, puzzle_id: u32) {
        revoke_session(env, player, puzzle_id)
    }

    pub fn place_suspect(env: Env, player: Address, puzzle_id: u32, x: u32, y: u32, suspect_idx: u32) -> MoveResult {
        crate::auth::require_player_auth(&env, &player, puzzle_id, symbol_short!("place_suspect"));
        
        let session_key = (Symbol::new(&env, "SESSION"), player.clone(), puzzle_id);
        let mut _app = GameApp::load(&env, session_key).expect("Game not started");
        let puzzle = Self::get_puzzle(env.clone(), puzzle_id);
        if x >= puzzle.grid_size || y >= puzzle.grid_size {
            return MoveResult::InvalidCoordinates;
        }

        let session_key = (Symbol::new(&env, "SESSION"), puzzle_id, player.clone());
        let mut app = GameApp::load(&env, session_key).expect("Game not started");
        
        // Execution of PlacementValidationSystem, etc.
        // Note: Real system execution would go here. For now returning Ok for bounds check.
        MoveResult::Ok
    }

    pub fn remove_suspect(env: Env, player: Address, puzzle_id: u32, x: u32, y: u32) -> MoveResult {
        crate::auth::require_player_auth(&env, &player, puzzle_id, symbol_short!("remove_suspect"));
        let session_key = (Symbol::new(&env, "SESSION"), player.clone(), puzzle_id);
        let mut _app = GameApp::load(&env, session_key).expect("Game not started");
        
        let puzzle = Self::get_puzzle(env.clone(), puzzle_id);
        if x >= puzzle.grid_size || y >= puzzle.grid_size {
            return MoveResult::InvalidCoordinates;
        }

        let session_key = (Symbol::new(&env, "SESSION"), puzzle_id, player.clone());
        let mut app = GameApp::load(&env, session_key).expect("Game not started");
        
        MoveResult::Ok
    }
}

// ──────────────────────────────────────────────
// v1 tests (plaintext solution, no ZK feature)
// ──────────────────────────────────────────────
#[cfg(all(test, not(feature = "zk")))]
mod v1_tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, String, Vec};

    fn make_valid_puzzle(env: &Env) -> (u32, Vec<String>, Vec<Clue>, Vec<u32>, PuzzleMetadata) {
        let grid_size = 4;
        let mut suspects = Vec::new(env);
        suspects.push_back(String::from_str(env, "Alice"));
        suspects.push_back(String::from_str(env, "Bob"));
        suspects.push_back(String::from_str(env, "Charlie"));
        suspects.push_back(String::from_str(env, "David"));

        let mut clues = Vec::new(env);
        clues.push_back(Clue {
            row: 0,
            col: 0,
            suspect_idx: 1,
        });
        clues.push_back(Clue {
            row: 1,
            col: 1,
            suspect_idx: 3,
        });

        let mut solution = Vec::new(env);
        solution.push_back(1);
        solution.push_back(2);
        solution.push_back(3);
        solution.push_back(4);
        solution.push_back(2);
        solution.push_back(3);
        solution.push_back(4);
        solution.push_back(1);
        solution.push_back(3);
        solution.push_back(4);
        solution.push_back(1);
        solution.push_back(2);
        solution.push_back(4);
        solution.push_back(1);
        solution.push_back(2);
        solution.push_back(3);

        let metadata = PuzzleMetadata {
            name: String::from_str(env, "Classic Case"),
            difficulty: String::from_str(env, "Easy"),
        };

        (grid_size, suspects, clues, solution, metadata)
    }

    #[test]
    fn test_submit_valid_puzzle() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let (grid_size, suspects, clues, solution, metadata) = make_valid_puzzle(&env);

        let id = client.submit_puzzle(
            &creator, &grid_size, &suspects, &clues, &solution, &metadata,
        );
        assert_eq!(id, 1);
        assert_eq!(client.get_puzzle_count(), 1);

        let puzzle = client.get_puzzle(&1);
        assert_eq!(puzzle.id, 1);
        assert_eq!(puzzle.creator, creator);
        assert_eq!(puzzle.grid_size, 4);
        assert_eq!(puzzle.suspects.len(), 4);
        assert_eq!(puzzle.clues.len(), 2);
        assert_eq!(puzzle.solution.len(), 16);
        assert!(puzzle.active);
    }

    #[test]
    fn test_submit_invalid_grid_size() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);
        let creator = Address::generate(&env);
        let (_, suspects, clues, solution, metadata) = make_valid_puzzle(&env);
        let result =
            client.try_submit_puzzle(&creator, &3, &suspects, &clues, &solution, &metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_submit_invalid_suspects_length() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);
        let creator = Address::generate(&env);
        let (grid_size, mut suspects, clues, solution, metadata) = make_valid_puzzle(&env);
        suspects.pop_back();
        let result = client.try_submit_puzzle(
            &creator, &grid_size, &suspects, &clues, &solution, &metadata,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_submit_empty_suspect_name() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);
        let creator = Address::generate(&env);
        let (grid_size, mut suspects, clues, solution, metadata) = make_valid_puzzle(&env);
        suspects.set(1, String::from_str(&env, ""));
        let result = client.try_submit_puzzle(
            &creator, &grid_size, &suspects, &clues, &solution, &metadata,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_submit_invalid_solution_length() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);
        let creator = Address::generate(&env);
        let (grid_size, suspects, clues, mut solution, metadata) = make_valid_puzzle(&env);
        solution.pop_back();
        let result = client.try_submit_puzzle(
            &creator, &grid_size, &suspects, &clues, &solution, &metadata,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_submit_invalid_latin_square() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);
        let creator = Address::generate(&env);
        let (grid_size, suspects, clues, mut solution, metadata) = make_valid_puzzle(&env);
        solution.set(1, 1);
        let result = client.try_submit_puzzle(
            &creator, &grid_size, &suspects, &clues, &solution, &metadata,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_submit_empty_clues() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);
        let creator = Address::generate(&env);
        let (grid_size, suspects, _, solution, metadata) = make_valid_puzzle(&env);
        let empty_clues = Vec::new(&env);
        let result = client.try_submit_puzzle(
            &creator,
            &grid_size,
            &suspects,
            &empty_clues,
            &solution,
            &metadata,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_submit_invalid_clue_coordinate() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);
        let creator = Address::generate(&env);
        let (grid_size, suspects, mut clues, solution, metadata) = make_valid_puzzle(&env);
        clues.push_back(Clue {
            row: 4,
            col: 0,
            suspect_idx: 1,
        });
        let result = client.try_submit_puzzle(
            &creator, &grid_size, &suspects, &clues, &solution, &metadata,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_submit_clue_mismatch_with_solution() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);
        let creator = Address::generate(&env);
        let (grid_size, suspects, mut clues, solution, metadata) = make_valid_puzzle(&env);
        clues.push_back(Clue {
            row: 0,
            col: 1,
            suspect_idx: 4,
        });
        let result = client.try_submit_puzzle(
            &creator, &grid_size, &suspects, &clues, &solution, &metadata,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_list_puzzles_and_pagination() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);
        let creator = Address::generate(&env);
        let (grid_size, suspects, clues, solution, metadata) = make_valid_puzzle(&env);
        client.submit_puzzle(
            &creator, &grid_size, &suspects, &clues, &solution, &metadata,
        );
        client.submit_puzzle(
            &creator, &grid_size, &suspects, &clues, &solution, &metadata,
        );
        client.submit_puzzle(
            &creator, &grid_size, &suspects, &clues, &solution, &metadata,
        );
        assert_eq!(client.get_puzzle_count(), 3);
        let list_all = client.list_puzzles(&0, &10);
        assert_eq!(list_all.len(), 3);
        let list_page = client.list_puzzles(&1, &1);
        assert_eq!(list_page.len(), 1);
        assert_eq!(list_page.get(0).unwrap().id, 2);
    }

    #[test]
    fn test_deactivate_puzzle_authorization() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);
        let creator = Address::generate(&env);
        let (grid_size, suspects, clues, solution, metadata) = make_valid_puzzle(&env);
        client.submit_puzzle(
            &creator, &grid_size, &suspects, &clues, &solution, &metadata,
        );
        let intruder = Address::generate(&env);
        let result = client.try_deactivate_puzzle(&intruder, &1);
        assert!(result.is_err());
        client.deactivate_puzzle(&creator, &1);
        let puzzle = client.get_puzzle(&1);
        assert!(!puzzle.active);
    }
}

// ──────────────────────────────────────────────
// ZK tests (solution commitment + proof)
// ──────────────────────────────────────────────
#[cfg(all(test, feature = "zk"))]
mod zk_tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Bytes, BytesN, Env, String, Vec};

    fn make_valid_zk_puzzle(
        env: &Env,
    ) -> (
        u32,
        Vec<String>,
        Vec<Clue>,
        BytesN<32>,
        Bytes,
        PuzzleMetadata,
    ) {
        let grid_size = 4;
        let mut suspects = Vec::new(env);
        suspects.push_back(String::from_str(env, "Alice"));
        suspects.push_back(String::from_str(env, "Bob"));
        suspects.push_back(String::from_str(env, "Charlie"));
        suspects.push_back(String::from_str(env, "David"));

        let mut clues = Vec::new(env);
        clues.push_back(Clue {
            row: 0,
            col: 0,
            suspect_idx: 1,
        });
        clues.push_back(Clue {
            row: 1,
            col: 1,
            suspect_idx: 3,
        });

        // A non-zero commitment (would be Poseidon2(solution || salt) in production)
        let solution_commitment = BytesN::from_array(env, &[1u8; 32]);
        // A minimal non-empty verifier key (just a placeholder for testing)
        let verifier_key = Bytes::from_array(env, &[0x01u8; 32]);

        let metadata = PuzzleMetadata {
            name: String::from_str(env, "ZK Case"),
            difficulty: String::from_str(env, "Hard"),
        };

        (
            grid_size,
            suspects,
            clues,
            solution_commitment,
            verifier_key,
            metadata,
        )
    }

    #[test]
    fn test_zk_submit_valid_puzzle() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let (grid_size, suspects, clues, commitment, vk, metadata) = make_valid_zk_puzzle(&env);

        let id = client.submit_puzzle(
            &creator,
            &grid_size,
            &suspects,
            &clues,
            &commitment,
            &vk,
            &metadata,
        );
        assert_eq!(id, 1);
        assert_eq!(client.get_puzzle_count(), 1);

        let puzzle = client.get_puzzle(&1);
        assert_eq!(puzzle.id, 1);
        assert_eq!(puzzle.creator, creator);
        assert_eq!(puzzle.grid_size, 4);
        assert_eq!(puzzle.suspects.len(), 4);
        assert_eq!(puzzle.clues.len(), 2);
        assert_eq!(puzzle.solution_commitment, commitment);
        assert!(puzzle.active);
    }

    #[test]
    fn test_zk_submit_zero_commitment_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let (grid_size, suspects, clues, _, vk, metadata) = make_valid_zk_puzzle(&env);
        let zero_commitment = BytesN::from_array(&env, &[0u8; 32]);

        let result = client.try_submit_puzzle(
            &creator,
            &grid_size,
            &suspects,
            &clues,
            &zero_commitment,
            &vk,
            &metadata,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_zk_submit_empty_verifier_key_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let (grid_size, suspects, clues, commitment, _, metadata) = make_valid_zk_puzzle(&env);
        let empty_vk = Bytes::new(&env);

        let result = client.try_submit_puzzle(
            &creator,
            &grid_size,
            &suspects,
            &clues,
            &commitment,
            &empty_vk,
            &metadata,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_zk_submit_invalid_grid_size() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let (_, suspects, clues, commitment, vk, metadata) = make_valid_zk_puzzle(&env);

        let result =
            client.try_submit_puzzle(&creator, &3, &suspects, &clues, &commitment, &vk, &metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_zk_submit_empty_clues_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let (grid_size, suspects, _, commitment, vk, metadata) = make_valid_zk_puzzle(&env);
        let empty_clues = Vec::new(&env);

        let result = client.try_submit_puzzle(
            &creator,
            &grid_size,
            &suspects,
            &empty_clues,
            &commitment,
            &vk,
            &metadata,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_zk_submit_invalid_clue_coordinate() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let (grid_size, suspects, mut clues, commitment, vk, metadata) = make_valid_zk_puzzle(&env);
        clues.push_back(Clue {
            row: 10,
            col: 0,
            suspect_idx: 1,
        });

        let result = client.try_submit_puzzle(
            &creator,
            &grid_size,
            &suspects,
            &clues,
            &commitment,
            &vk,
            &metadata,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_zk_proof_invalid_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let (grid_size, suspects, clues, commitment, vk, metadata) = make_valid_zk_puzzle(&env);
        let _id = client.submit_puzzle(
            &creator,
            &grid_size,
            &suspects,
            &clues,
            &commitment,
            &vk,
            &metadata,
        );

        let player = Address::generate(&env);
        let invalid_proof = Bytes::from_array(&env, &[0u8; 16]);
        let public_inputs = Vec::new(&env);

        let result = client.try_submit_proof(&player, &1, &invalid_proof, &public_inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_zk_proof_nonexistent_puzzle_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);

        let player = Address::generate(&env);
        let proof = Bytes::new(&env);
        let public_inputs = Vec::new(&env);

        let result = client.try_submit_proof(&player, &999, &proof, &public_inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_zk_is_solved_returns_false_before_proof() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let (grid_size, suspects, clues, commitment, vk, metadata) = make_valid_zk_puzzle(&env);
        client.submit_puzzle(
            &creator,
            &grid_size,
            &suspects,
            &clues,
            &commitment,
            &vk,
            &metadata,
        );

        let player = Address::generate(&env);
        let solved = client.is_solved(&1, &player);
        assert!(!solved);
    }

    #[test]
    fn test_zk_list_puzzles_and_pagination() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let (grid_size, suspects, clues, commitment, vk, metadata) = make_valid_zk_puzzle(&env);

        client.submit_puzzle(
            &creator,
            &grid_size,
            &suspects,
            &clues,
            &commitment,
            &vk,
            &metadata,
        );
        client.submit_puzzle(
            &creator,
            &grid_size,
            &suspects,
            &clues,
            &commitment,
            &vk,
            &metadata,
        );
        client.submit_puzzle(
            &creator,
            &grid_size,
            &suspects,
            &clues,
            &commitment,
            &vk,
            &metadata,
        );

        assert_eq!(client.get_puzzle_count(), 3);
        let list_all = client.list_puzzles(&0, &10);
        assert_eq!(list_all.len(), 3);
        let list_page = client.list_puzzles(&1, &1);
        assert_eq!(list_page.len(), 1);
        assert_eq!(list_page.get(0).unwrap().id, 2);
    }

    #[test]
    fn test_zk_deactivate_puzzle_authorization() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(MurdokuContract, ());
        let client = MurdokuContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let (grid_size, suspects, clues, commitment, vk, metadata) = make_valid_zk_puzzle(&env);
        client.submit_puzzle(
            &creator,
            &grid_size,
            &suspects,
            &clues,
            &commitment,
            &vk,
            &metadata,
        );

        let intruder = Address::generate(&env);
        let result = client.try_deactivate_puzzle(&intruder, &1);
        assert!(result.is_err());

        client.deactivate_puzzle(&creator, &1);
        let puzzle = client.get_puzzle(&1);
        assert!(!puzzle.active);
    }
}
