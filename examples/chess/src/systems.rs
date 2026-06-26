use soroban_sdk::{Bytes, BytesN, Env, Map};

use crate::components::{Color, GameState, GameStatus, Piece, PieceKind};

pub(crate) fn init_board(env: &Env) -> Map<u32, Piece> {
    let mut board = Map::new(env);

    // White pieces (bottom)
    board.set(
        0,
        Piece {
            kind: PieceKind::Rook,
            color: Color::White,
        },
    );
    board.set(
        1,
        Piece {
            kind: PieceKind::Knight,
            color: Color::White,
        },
    );
    board.set(
        2,
        Piece {
            kind: PieceKind::Bishop,
            color: Color::White,
        },
    );
    board.set(
        3,
        Piece {
            kind: PieceKind::Queen,
            color: Color::White,
        },
    );
    board.set(
        4,
        Piece {
            kind: PieceKind::King,
            color: Color::White,
        },
    );
    board.set(
        5,
        Piece {
            kind: PieceKind::Bishop,
            color: Color::White,
        },
    );
    board.set(
        6,
        Piece {
            kind: PieceKind::Knight,
            color: Color::White,
        },
    );
    board.set(
        7,
        Piece {
            kind: PieceKind::Rook,
            color: Color::White,
        },
    );
    for i in 8..16 {
        board.set(
            i,
            Piece {
                kind: PieceKind::Pawn,
                color: Color::White,
            },
        );
    }

    // Black pieces (top)
    board.set(
        56,
        Piece {
            kind: PieceKind::Rook,
            color: Color::Black,
        },
    );
    board.set(
        57,
        Piece {
            kind: PieceKind::Knight,
            color: Color::Black,
        },
    );
    board.set(
        58,
        Piece {
            kind: PieceKind::Bishop,
            color: Color::Black,
        },
    );
    board.set(
        59,
        Piece {
            kind: PieceKind::Queen,
            color: Color::Black,
        },
    );
    board.set(
        60,
        Piece {
            kind: PieceKind::King,
            color: Color::Black,
        },
    );
    board.set(
        61,
        Piece {
            kind: PieceKind::Bishop,
            color: Color::Black,
        },
    );
    board.set(
        62,
        Piece {
            kind: PieceKind::Knight,
            color: Color::Black,
        },
    );
    board.set(
        63,
        Piece {
            kind: PieceKind::Rook,
            color: Color::Black,
        },
    );
    for i in 48..56 {
        board.set(
            i,
            Piece {
                kind: PieceKind::Pawn,
                color: Color::Black,
            },
        );
    }

    board
}

pub(crate) fn compute_state_hash(env: &Env, board: &Map<u32, Piece>) -> BytesN<32> {
    let mut data = Bytes::new(env);
    for pos in 0..64u32 {
        if let Some(piece) = board.get(pos) {
            data.append(&Bytes::from_array(env, &pos.to_be_bytes()));
            let kind_byte = match piece.kind {
                PieceKind::King => 1u8,
                PieceKind::Queen => 2u8,
                PieceKind::Rook => 3u8,
                PieceKind::Bishop => 4u8,
                PieceKind::Knight => 5u8,
                PieceKind::Pawn => 6u8,
            };
            let color_byte = match piece.color {
                Color::White => 0u8,
                Color::Black => 1u8,
            };
            data.append(&Bytes::from_array(env, &[kind_byte, color_byte]));
        }
    }
    env.crypto().sha256(&data).into()
}

pub(crate) fn apply_move(game: &mut GameState, from: u32, to: u32) {
    if let Some(piece) = game.board.pieces.get(from) {
        game.board.pieces.set(to, piece);
        game.board.pieces.remove(from);
    }
}

pub(crate) fn check_endgame(game: &mut GameState) {
    let mut white_king_exists = false;
    let mut black_king_exists = false;

    for pos in 0..64u32 {
        if let Some(piece) = game.board.pieces.get(pos) {
            if piece.kind == PieceKind::King {
                match piece.color {
                    Color::White => white_king_exists = true,
                    Color::Black => black_king_exists = true,
                }
            }
        }
    }

    if !white_king_exists || !black_king_exists {
        game.turn.status = GameStatus::Checkmate;
    }
}
