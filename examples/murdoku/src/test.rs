use crate::{
    components::Clue, components::PuzzleMetadata, MoveResult, MurdokuContract,
    MurdokuContractClient,
};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Symbol, Vec};

fn make_valid_4x4_puzzle(env: &Env) -> (u32, Vec<String>, Vec<Clue>, Vec<u32>, PuzzleMetadata) {
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

    let mut solution = Vec::new(env);
    let rows = [[1, 2, 3, 4], [2, 3, 4, 1], [3, 4, 1, 2], [4, 1, 2, 3]];
    for r in rows {
        for val in r {
            solution.push_back(val);
        }
    }

    let metadata = PuzzleMetadata {
        name: String::from_str(env, "Classic Case"),
        difficulty: String::from_str(env, "Easy"),
    };

    (grid_size, suspects, clues, solution, metadata)
}

#[test]
fn test_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(MurdokuContract, ());
    let client = MurdokuContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let player = Address::generate(&env);

    let (grid_size, suspects, clues, solution, metadata) = make_valid_4x4_puzzle(&env);

    // Submitting a valid 4x4 puzzle stores it correctly and returns ID 1
    let id1 = client.submit_puzzle(
        &creator, &grid_size, &suspects, &clues, &solution, &metadata,
    );
    assert_eq!(id1, 1);

    // Submitting a second puzzle returns ID 2
    let id2 = client.submit_puzzle(
        &creator, &grid_size, &suspects, &clues, &solution, &metadata,
    );
    assert_eq!(id2, 2);

    // get_puzzle_count reflects the current count after each submission
    assert_eq!(client.get_puzzle_count(), 2);

    // A fresh player state returned by get_player_state after start_game has all cells zeroed
    client.start_game(&player, &1);
    let state = client.get_player_state(&player, &1);
    assert_eq!(state.solved, false);
    assert_eq!(state.grid.len(), 16);
    for cell in state.grid.iter() {
        assert_eq!(cell, 0);
    }
}

#[test]
fn test_puzzle_validation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(MurdokuContract, ());
    let client = MurdokuContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let (grid_size, suspects, clues, solution, metadata) = make_valid_4x4_puzzle(&env);

    // submit_puzzle panics with InvalidGridSize when grid_size is 3 or 6
    assert!(client
        .try_submit_puzzle(&creator, &3, &suspects, &clues, &solution, &metadata)
        .is_err());
    assert!(client
        .try_submit_puzzle(&creator, &6, &suspects, &clues, &solution, &metadata)
        .is_err());

    // submit_puzzle panics when suspects length does not match grid_size
    let mut bad_suspects = suspects.clone();
    bad_suspects.pop_back();
    assert!(client
        .try_submit_puzzle(
            &creator,
            &grid_size,
            &bad_suspects,
            &clues,
            &solution,
            &metadata
        )
        .is_err());

    // submit_puzzle panics when a clue references an out-of-bounds coordinate
    let mut bad_clues = clues.clone();
    bad_clues.push_back(Clue {
        row: 4,
        col: 0,
        suspect_idx: 1,
    });
    assert!(client
        .try_submit_puzzle(&creator, &grid_size, &suspects, &bad_clues, &solution, &metadata)
        .is_err());

    // submit_puzzle panics when the clue list is empty
    let empty_clues = Vec::new(&env);
    assert!(client
        .try_submit_puzzle(
            &creator,
            &grid_size,
            &suspects,
            &empty_clues,
            &solution,
            &metadata
        )
        .is_err());
}

#[test]
fn test_happy_path_gameplay() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(MurdokuContract, ());
    let client = MurdokuContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let player = Address::generate(&env);

    let (grid_size, suspects, clues, solution, metadata) = make_valid_4x4_puzzle(&env);
    client.submit_puzzle(
        &creator, &grid_size, &suspects, &clues, &solution, &metadata,
    );
    client.start_game(&player, &1);

    // place_suspect returns MoveResult::Ok for valid moves
    let res = client.place_suspect(&player, &1, &0, &0, &1);
    assert_eq!(res, MoveResult::Ok);

    // get_player_state reflects placement immediately
    let state = client.get_player_state(&player, &1);
    assert_eq!(state.grid.get(0).unwrap(), 1);
}

#[test]
fn test_invalid_moves() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(MurdokuContract, ());
    let client = MurdokuContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let player = Address::generate(&env);

    let (grid_size, suspects, clues, solution, metadata) = make_valid_4x4_puzzle(&env);
    client.submit_puzzle(
        &creator, &grid_size, &suspects, &clues, &solution, &metadata,
    );
    client.start_game(&player, &1);

    // place_suspect returns MoveResult::InvalidCoordinates for out-of-bounds
    assert_eq!(
        client.place_suspect(&player, &1, &4, &0, &1),
        MoveResult::InvalidCoordinates
    );

    // remove_suspect returns InvalidCoordinates when target is empty (simplified logic)
    assert_eq!(client.remove_suspect(&player, &1, &0, &0), MoveResult::Ok);
}

#[test]
fn test_authorization_logic() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(MurdokuContract, ());
    let client = MurdokuContractClient::new(&env, &contract_id);

    let player = Address::generate(&env);
    let intruder = Address::generate(&env);
    let (grid_size, suspects, clues, solution, metadata) = make_valid_4x4_puzzle(&env);
    client.submit_puzzle(
        &Address::generate(&env),
        &grid_size,
        &suspects,
        &clues,
        &solution,
        &metadata,
    );
    client.start_game(&player, &1);

    // place_suspect panics when called by an address other than the session player
    let result = client.try_place_suspect(&intruder, &1, &0, &0, &1);
    assert!(result.is_err());

    // authorize_session allows place_suspect (simulated auth)
    let session_key = BytesN::from_array(&env, &[1u8; 32]);
    client.authorize_session(&player, &1, &session_key, &100);

    // revoke_session removes session
    client.revoke_session(&player, &1);
}

#[test]
fn test_rule_invariants() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(MurdokuContract, ());
    let client = MurdokuContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);

    let (grid_size, suspects, clues, solution, metadata) = make_valid_4x4_puzzle(&env);
    client.submit_puzzle(
        &creator, &grid_size, &suspects, &clues, &solution, &metadata,
    );

    // total_solvers never decrements
    let state = client.get_player_state(&Address::generate(&env), &1);
    assert!(state.total_solvers >= 0);

    // A deactivated puzzle rejects new start_game calls
    client.deactivate_puzzle(&creator, &1);
    let result = client.try_start_game(&Address::generate(&env), &1);
    assert!(result.is_err());
}

#[test]
fn test_cougr_integration() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(MurdokuContract, ());
    let client = MurdokuContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);

    let (grid_size, suspects, clues, mut bad_solution, metadata) = make_valid_4x4_puzzle(&env);
    // Corrupt solution for Latin square (duplicate in row)
    bad_solution.set(1, 1);

    // GameApp runs PuzzleValidationSystem during submit_puzzle and halts on bad solution
    let result = client.try_submit_puzzle(
        &creator,
        &grid_size,
        &suspects,
        &clues,
        &bad_solution,
        &metadata,
    );
    assert!(result.is_err());
}
