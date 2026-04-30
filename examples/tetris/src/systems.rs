use crate::components::{PieceComponent, TetrominoShape};
use cougr_core::{ComponentTrait, SimpleWorld};
use soroban_sdk::{symbol_short, Env, Vec};

pub const BOARD_WIDTH: i32 = 10;
pub const BOARD_HEIGHT: i32 = 20;

/// Returns the 4 cell offsets (dx, dy) for a given shape and rotation.
pub fn piece_coords(shape: TetrominoShape, rot: u32) -> [(i32, i32); 4] {
    match shape {
        TetrominoShape::I => match rot {
            0 => [(-1, 0), (0, 0), (1, 0), (2, 0)],
            1 => [(1, -1), (1, 0), (1, 1), (1, 2)],
            2 => [(-1, 1), (0, 1), (1, 1), (2, 1)],
            _ => [(0, -1), (0, 0), (0, 1), (0, 2)],
        },
        TetrominoShape::O => [(0, 0), (1, 0), (0, 1), (1, 1)],
        TetrominoShape::T => match rot {
            0 => [(-1, 0), (0, 0), (1, 0), (0, 1)],
            1 => [(0, -1), (0, 0), (0, 1), (-1, 0)],
            2 => [(-1, 0), (0, 0), (1, 0), (0, -1)],
            _ => [(0, -1), (0, 0), (0, 1), (1, 0)],
        },
        TetrominoShape::J => match rot {
            0 => [(-1, 0), (0, 0), (1, 0), (1, 1)],
            1 => [(0, -1), (0, 0), (0, 1), (-1, 1)],
            2 => [(-1, -1), (-1, 0), (0, 0), (1, 0)],
            _ => [(1, -1), (0, 0), (0, -1), (0, 1)],
        },
        TetrominoShape::L => match rot {
            0 => [(-1, 0), (0, 0), (1, 0), (-1, 1)],
            1 => [(0, -1), (0, 0), (0, 1), (1, 1)],
            2 => [(1, -1), (-1, 0), (0, 0), (1, 0)],
            _ => [(-1, -1), (0, -1), (0, 0), (0, 1)],
        },
        TetrominoShape::S => match rot {
            0 | 2 => [(0, 0), (1, 0), (-1, 1), (0, 1)],
            _ => [(0, -1), (0, 0), (1, 0), (1, 1)],
        },
        TetrominoShape::Z => match rot {
            0 | 2 => [(-1, 0), (0, 0), (0, 1), (1, 1)],
            _ => [(1, -1), (1, 0), (0, 0), (0, 1)],
        },
    }
}

/// Returns true if the given piece position/rotation collides with the board or walls.
pub fn collision_system(board: &[u32], shape: TetrominoShape, x: i32, y: i32, rot: u32) -> bool {
    for (dx, dy) in piece_coords(shape, rot) {
        let ax = x + dx;
        let ay = y + dy;
        if !(0..BOARD_WIDTH).contains(&ax) || ay >= BOARD_HEIGHT {
            return true;
        }
        if ay >= 0 {
            let row = board.get(ay as u32).unwrap_or(0);
            if (row >> ax) & 1 == 1 {
                return true;
            }
        }
    }
    false
}

/// Gravity system: tries to move the active piece down by one row.
/// Returns true if the piece moved, false if it should be locked.
pub fn gravity_system(world: &mut SimpleWorld, env: &Env, board: &Vec<u32>) -> bool {
    let entities = world.get_entities_with_component(&symbol_short!("piece"), env);
    if entities.is_empty() {
        return false;
    }
    let id = entities.get(0).unwrap();
    let data = world.get_component(id, &symbol_short!("piece")).unwrap();
    let mut piece = PieceComponent::deserialize(env, &data).unwrap();

    if collision_system(board, piece.shape, piece.x, piece.y + 1, piece.rotation) {
        return false;
    }
    piece.y += 1;
    world.set_typed(env, id, &piece);
    true
}

/// Lock system: writes the active piece onto the board, clears full lines,
/// and returns (new_board, lines_cleared).
pub fn lock_system(
    world: &mut SimpleWorld,
    env: &Env,
    board: &[u32],  // ✅ Correct
) -> (Vec<u32>, u32) {
    let entities = world.get_entities_with_component(&symbol_short!("piece"), env);
    if entities.is_empty() {
        return (board.clone(), 0);
    }
    let id = entities.get(0).unwrap();
    let data = world.get_component(id, &symbol_short!("piece")).unwrap();
    let piece = PieceComponent::deserialize(env, &data).unwrap();

    let mut new_board = board.clone();
    for (dx, dy) in piece_coords(piece.shape, piece.rotation) {
        let ax = piece.x + dx;
        let ay = piece.y + dy;
        if ay >= 0 && ay < BOARD_HEIGHT {
            let mut row = new_board.get(ay as u32).unwrap_or(0);
            row |= 1 << ax;
            new_board.set(ay as u32, row);
        }
    }

    // Clear full lines
    let mut cleared = Vec::new(env);
    for i in 0..new_board.len() {
        let row = new_board.get(i).unwrap();
        if row != 1023 {
            cleared.push_back(row);
        }
    }
    let lines = new_board.len() - cleared.len();
    let mut final_board = Vec::new(env);
    for _ in 0..lines {
        final_board.push_back(0u32);
    }
    for i in 0..cleared.len() {
        final_board.push_back(cleared.get(i).unwrap());
    }

    world.despawn_entity(id);
    (final_board, lines)
}
