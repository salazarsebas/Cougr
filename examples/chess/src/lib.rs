#![no_std]

mod components;
mod systems;
#[cfg(test)]
mod test;
mod zk;

use components::{
    BoardState, GameState, GameStatus, MoveResult, ProofRecord, TurnState, GAME_KEY, VK_KEY,
};
use systems::{apply_move, check_endgame, compute_state_hash, init_board};
use zk::{build_move_circuit, decode_proof};

use cougr_core::privacy::VerificationKey;
use soroban_sdk::{contract, contractimpl, Address, Bytes, Env};

#[contract]
pub struct ChessContract;

#[contractimpl]
impl ChessContract {
    /// Initialize a new chess game
    pub fn new_game(env: Env, white: Address, black: Address) {
        let board = init_board(&env);
        let state_hash = compute_state_hash(&env, &board);

        let game_state = GameState {
            white: white.clone(),
            black: black.clone(),
            board: BoardState {
                state_hash,
                pieces: board,
            },
            turn: TurnState {
                current: white,
                move_count: 0,
                status: GameStatus::Playing,
            },
            proof_record: ProofRecord {
                last_proof: Bytes::new(&env),
                verified: false,
            },
        };

        env.storage().instance().set(&GAME_KEY, &game_state);
    }

    /// Submit a move with ZK proof
    pub fn submit_move(env: Env, player: Address, from: u32, to: u32, proof: Bytes) -> MoveResult {
        player.require_auth();

        let mut game: GameState = env
            .storage()
            .instance()
            .get(&GAME_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        // TurnSystem: validate player
        if game.turn.current != player {
            return MoveResult::WrongTurn;
        }

        if game.turn.status != GameStatus::Playing {
            return MoveResult::GameOver;
        }

        // ProofVerificationSystem: verify the move proof
        let vk: VerificationKey = env
            .storage()
            .instance()
            .get(&VK_KEY)
            .unwrap_or_else(|| panic!("VK not set"));

        let groth16_proof = decode_proof(&env, &proof);
        let circuit = build_move_circuit(&env, &vk, &game.board.state_hash, from, to);

        let verified = circuit.verify(&env, &groth16_proof).unwrap_or(false);

        if !verified {
            return MoveResult::InvalidProof;
        }

        // BoardUpdateSystem: apply the move
        apply_move(&mut game, from, to);

        // Update state hash
        game.board.state_hash = compute_state_hash(&env, &game.board.pieces);

        // Update proof record
        game.proof_record.last_proof = proof;
        game.proof_record.verified = true;

        // TurnSystem: switch turn
        game.turn.move_count += 1;
        game.turn.current = if game.turn.current == game.white {
            game.black.clone()
        } else {
            game.white.clone()
        };

        // EndGameSystem: check for checkmate (simplified: king captured)
        check_endgame(&mut game);

        env.storage().instance().set(&GAME_KEY, &game);
        MoveResult::Success
    }

    /// Resign the game
    pub fn resign(env: Env, player: Address) {
        player.require_auth();

        let mut game: GameState = env
            .storage()
            .instance()
            .get(&GAME_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        if player == game.white || player == game.black {
            game.turn.status = GameStatus::Resigned;
            env.storage().instance().set(&GAME_KEY, &game);
        }
    }

    /// Get the current board state
    pub fn get_board(env: Env) -> BoardState {
        let game: GameState = env
            .storage()
            .instance()
            .get(&GAME_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));
        game.board
    }

    /// Get the current game state
    pub fn get_state(env: Env) -> GameState {
        env.storage()
            .instance()
            .get(&GAME_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"))
    }

    /// Set the verification key (admin function)
    pub fn set_vk(env: Env, vk: VerificationKey) {
        env.storage().instance().set(&VK_KEY, &vk);
    }
}
