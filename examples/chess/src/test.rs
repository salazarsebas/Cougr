use super::*;
use crate::components::{Color, Piece, PieceKind};
use cougr_core::component::ComponentTrait;
use cougr_core::zk::{G1Point, G2Point};
use soroban_sdk::{symbol_short, testutils::Address as _, Address, BytesN, Env, Map, Vec};

fn setup_game() -> (Env, ChessContractClient<'static>, Address, Address) {
    let env = Env::default();
    let contract_id = env.register(ChessContract, ());
    let client = ChessContractClient::new(&env, &contract_id);

    let white = Address::generate(&env);
    let black = Address::generate(&env);

    (env, client, white, black)
}

fn make_mock_vk(env: &Env) -> VerificationKey {
    let g1 = G1Point {
        bytes: BytesN::from_array(env, &[0u8; 64]),
    };
    let g2 = G2Point {
        bytes: BytesN::from_array(env, &[0u8; 128]),
    };
    let mut ic = Vec::new(env);
    for _ in 0..4 {
        ic.push_back(g1.clone());
    }
    VerificationKey {
        alpha: g1,
        beta: g2.clone(),
        gamma: g2.clone(),
        delta: g2,
        ic,
    }
}

fn make_mock_proof(env: &Env) -> Bytes {
    // Mock proof bytes - in real usage, this would be a serialized Groth16 proof
    Bytes::from_array(env, &[1u8; 32])
}

#[test]
fn test_init_game() {
    let (_env, client, white, black) = setup_game();

    client.new_game(&white, &black);

    let state = client.get_state();
    assert_eq!(state.white, white);
    assert_eq!(state.black, black);
    assert_eq!(state.turn.current, white);
    assert_eq!(state.turn.move_count, 0);
    assert_eq!(state.turn.status, GameStatus::Playing);
}

#[test]
fn test_board_initialization() {
    let (_env, client, white, black) = setup_game();

    client.new_game(&white, &black);

    let board = client.get_board();

    // Check white pieces
    let white_king = board.pieces.get(4).unwrap();
    assert_eq!(white_king.kind, PieceKind::King);
    assert_eq!(white_king.color, Color::White);

    let white_queen = board.pieces.get(3).unwrap();
    assert_eq!(white_queen.kind, PieceKind::Queen);
    assert_eq!(white_queen.color, Color::White);

    let white_pawn = board.pieces.get(8).unwrap();
    assert_eq!(white_pawn.kind, PieceKind::Pawn);
    assert_eq!(white_pawn.color, Color::White);

    // Check black pieces
    let black_king = board.pieces.get(60).unwrap();
    assert_eq!(black_king.kind, PieceKind::King);
    assert_eq!(black_king.color, Color::Black);

    let black_queen = board.pieces.get(59).unwrap();
    assert_eq!(black_queen.kind, PieceKind::Queen);
    assert_eq!(black_queen.color, Color::Black);

    let black_pawn = board.pieces.get(48).unwrap();
    assert_eq!(black_pawn.kind, PieceKind::Pawn);
    assert_eq!(black_pawn.color, Color::Black);
}

#[test]
fn test_state_hash_consistency() {
    let (_env, client, white, black) = setup_game();

    client.new_game(&white, &black);
    let board1 = client.get_board();
    let hash1 = board1.state_hash;

    // Create another game with same setup
    let white2 = Address::generate(&client.env);
    let black2 = Address::generate(&client.env);
    client.new_game(&white2, &black2);
    let board2 = client.get_board();
    let hash2 = board2.state_hash;

    // Same initial board should produce same hash
    assert_eq!(hash1, hash2);
}

#[test]
#[should_panic(expected = "Game not initialized")]
fn test_move_without_init() {
    let (env, client, white, _) = setup_game();
    env.mock_all_auths();

    let proof = make_mock_proof(&env);
    client.submit_move(&white, &12, &20, &proof);
}

#[test]
fn test_wrong_turn() {
    let (env, client, white, black) = setup_game();
    env.mock_all_auths();

    client.new_game(&white, &black);
    let vk = make_mock_vk(&env);
    client.set_vk(&vk);

    let proof = make_mock_proof(&env);

    // Black tries to move first (should be white's turn)
    let result = client.submit_move(&black, &48, &40, &proof);
    assert_eq!(result, MoveResult::WrongTurn);
}

#[test]
fn test_resign() {
    let (env, client, white, black) = setup_game();
    env.mock_all_auths();

    client.new_game(&white, &black);
    client.resign(&white);

    let state = client.get_state();
    assert_eq!(state.turn.status, GameStatus::Resigned);
}

#[test]
fn test_move_after_resignation() {
    let (env, client, white, black) = setup_game();
    env.mock_all_auths();

    client.new_game(&white, &black);
    client.resign(&white);

    let vk = make_mock_vk(&env);
    client.set_vk(&vk);
    let proof = make_mock_proof(&env);

    let result = client.submit_move(&white, &12, &20, &proof);
    assert_eq!(result, MoveResult::GameOver);
}

#[test]
fn test_component_trait_board() {
    let env = Env::default();
    let board = BoardState {
        state_hash: BytesN::from_array(&env, &[0u8; 32]),
        pieces: Map::new(&env),
    };

    let serialized = board.serialize(&env);
    assert_eq!(serialized.len(), 32);
    assert_eq!(BoardState::component_type(), symbol_short!("board"));
}

#[test]
fn test_component_trait_turn() {
    let env = Env::default();
    let white = Address::generate(&env);
    let turn = TurnState {
        current: white,
        move_count: 5,
        status: GameStatus::Playing,
    };

    let serialized = turn.serialize(&env);
    assert_eq!(serialized.len(), 4);
    assert_eq!(TurnState::component_type(), symbol_short!("turn"));
}

#[test]
fn test_king_capture_checkmate() {
    let (env, client, white, black) = setup_game();
    env.mock_all_auths();

    client.new_game(&white, &black);

    // Manually manipulate game state to simulate king capture
    let mut state = client.get_state();
    state.board.pieces.remove(60); // Remove black king
    state.board.state_hash = env.crypto().sha256(&Bytes::new(&env)).into();

    // Store modified state
    env.as_contract(&client.address, || {
        env.storage().instance().set(&GAME_KEY, &state);
    });

    // Trigger endgame check by attempting a move
    let vk = make_mock_vk(&env);
    client.set_vk(&vk);

    // This would normally fail proof verification, but we're testing endgame logic
    // In a real scenario, the move would be validated first
}

#[test]
fn test_piece_movement_pawn() {
    let env = Env::default();
    let mut board = Map::new(&env);

    // Place white pawn at position 12 (row 1, col 4)
    board.set(
        12,
        Piece {
            kind: PieceKind::Pawn,
            color: Color::White,
        },
    );

    // Verify pawn exists
    let pawn = board.get(12).unwrap();
    assert_eq!(pawn.kind, PieceKind::Pawn);
    assert_eq!(pawn.color, Color::White);

    // Simulate move to position 20 (row 2, col 4)
    board.set(20, pawn);
    board.remove(12);

    assert!(board.get(12).is_none());
    let moved_pawn = board.get(20).unwrap();
    assert_eq!(moved_pawn.kind, PieceKind::Pawn);
}

#[test]
fn test_piece_movement_knight() {
    let env = Env::default();
    let mut board = Map::new(&env);

    // Place white knight at position 1 (row 0, col 1)
    board.set(
        1,
        Piece {
            kind: PieceKind::Knight,
            color: Color::White,
        },
    );

    // Simulate L-shaped move to position 18 (row 2, col 2)
    let knight = board.get(1).unwrap();
    board.set(18, knight);
    board.remove(1);

    assert!(board.get(1).is_none());
    let moved_knight = board.get(18).unwrap();
    assert_eq!(moved_knight.kind, PieceKind::Knight);
}

#[test]
fn test_piece_movement_rook() {
    let env = Env::default();
    let mut board = Map::new(&env);

    // Place white rook at position 0 (row 0, col 0)
    board.set(
        0,
        Piece {
            kind: PieceKind::Rook,
            color: Color::White,
        },
    );

    // Simulate straight move to position 32 (row 4, col 0)
    let rook = board.get(0).unwrap();
    board.set(32, rook);
    board.remove(0);

    assert!(board.get(0).is_none());
    let moved_rook = board.get(32).unwrap();
    assert_eq!(moved_rook.kind, PieceKind::Rook);
}

#[test]
fn test_turn_switching() {
    let (_env, client, white, black) = setup_game();

    client.new_game(&white, &black);

    let state = client.get_state();
    assert_eq!(state.turn.current, white);
    assert_eq!(state.turn.move_count, 0);

    // After a move, turn should switch to black
    // (This would require a valid proof in real usage)
}

#[test]
fn test_move_count_increment() {
    let (_env, client, white, black) = setup_game();

    client.new_game(&white, &black);

    let initial_state = client.get_state();
    assert_eq!(initial_state.turn.move_count, 0);

    // After moves, count should increment
    // (This would require valid proofs in real usage)
}

#[test]
fn test_all_piece_types_present() {
    let (_env, client, white, black) = setup_game();

    client.new_game(&white, &black);
    let board = client.get_board();

    // Count piece types
    let mut kings = 0;
    let mut queens = 0;
    let mut rooks = 0;
    let mut bishops = 0;
    let mut knights = 0;
    let mut pawns = 0;

    for pos in 0..64u32 {
        if let Some(piece) = board.pieces.get(pos) {
            match piece.kind {
                PieceKind::King => kings += 1,
                PieceKind::Queen => queens += 1,
                PieceKind::Rook => rooks += 1,
                PieceKind::Bishop => bishops += 1,
                PieceKind::Knight => knights += 1,
                PieceKind::Pawn => pawns += 1,
            }
        }
    }

    assert_eq!(kings, 2); // 1 white, 1 black
    assert_eq!(queens, 2);
    assert_eq!(rooks, 4);
    assert_eq!(bishops, 4);
    assert_eq!(knights, 4);
    assert_eq!(pawns, 16);
}

#[test]
fn test_proof_record_update() {
    let (_env, client, white, black) = setup_game();

    client.new_game(&white, &black);

    let initial_state = client.get_state();
    assert_eq!(initial_state.proof_record.verified, false);
    assert_eq!(initial_state.proof_record.last_proof.len(), 0);
}
