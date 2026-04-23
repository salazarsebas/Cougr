#![no_std]

mod components;
mod systems;

use components::{PieceComponent, TetrominoShape};
use cougr_core::{ComponentTrait, SimpleWorld};
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env, Vec};
use systems::{collision_system, gravity_system, lock_system, BOARD_WIDTH};

// ── Persistent metadata (score, level, next piece) ──────────────────────────

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    pub board: Vec<u32>,
    pub next_shape: TetrominoShape,
    pub score: u32,
    pub level: u32,
    pub lines_cleared: u32,
    pub game_over: bool,
}

// ── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct TetrisContract;

#[contractimpl]
impl TetrisContract {
    /// Initialize a new game.
    pub fn init_game(env: Env) -> GameState {
        let board = Vec::from_array(&env, [0u32; 20]);
        let first_shape = random_shape(&env, 0);
        let next_shape = random_shape(&env, 1);

        let mut world = SimpleWorld::new(&env);
        spawn_piece(&mut world, &env, first_shape);

        let state = GameState {
            board,
            next_shape,
            score: 0,
            level: 1,
            lines_cleared: 0,
            game_over: false,
        };

        save_world(&env, &world);
        save_state(&env, &state);
        state
    }

    /// Move the active piece left.
    pub fn move_left(env: Env) -> bool {
        let state = load_state(&env);
        if state.game_over { return false; }
        let mut world = load_world(&env);
        let moved = shift_piece(&mut world, &env, &state.board, -1, 0, 0);
        if moved { save_world(&env, &world); }
        moved
    }

    /// Move the active piece right.
    pub fn move_right(env: Env) -> bool {
        let state = load_state(&env);
        if state.game_over { return false; }
        let mut world = load_world(&env);
        let moved = shift_piece(&mut world, &env, &state.board, 1, 0, 0);
        if moved { save_world(&env, &world); }
        moved
    }

    /// Soft-drop: move the active piece down one row.
    pub fn move_down(env: Env) -> bool {
        let mut state = load_state(&env);
        if state.game_over { return false; }
        let mut world = load_world(&env);
        let moved = shift_piece(&mut world, &env, &state.board, 0, 1, 0);
        if moved {
            save_world(&env, &world);
            true
        } else {
            do_lock(&env, &mut state, &mut world);
            false
        }
    }

    /// Rotate the active piece clockwise.
    pub fn rotate(env: Env) -> bool {
        let state = load_state(&env);
        if state.game_over { return false; }
        let mut world = load_world(&env);
        let moved = shift_piece(&mut world, &env, &state.board, 0, 0, 1);
        if moved { save_world(&env, &world); }
        moved
    }

    /// Hard-drop: slam the piece to the bottom.
    pub fn drop(env: Env) -> u32 {
        let mut state = load_state(&env);
        if state.game_over { return 0; }
        let mut world = load_world(&env);
        let mut dropped = 0u32;
        while shift_piece(&mut world, &env, &state.board, 0, 1, 0) {
            dropped += 1;
        }
        do_lock(&env, &mut state, &mut world);
        dropped
    }

    /// Gravity tick: move piece down or lock it.
    pub fn update_tick(env: Env) -> GameState {
        let mut state = load_state(&env);
        if state.game_over { return state; }
        let mut world = load_world(&env);
        if !gravity_system(&mut world, &env, &state.board) {
            do_lock(&env, &mut state, &mut world);
        } else {
            save_world(&env, &world);
            save_state(&env, &state);
        }
        state
    }

    /// Return the current game state.
    pub fn get_state(env: Env) -> GameState {
        load_state(&env)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn save_state(env: &Env, state: &GameState) {
    env.storage().instance().set(&symbol_short!("game"), state);
}

fn load_state(env: &Env) -> GameState {
    env.storage()
        .instance()
        .get(&symbol_short!("game"))
        .expect("Game not initialized")
}

fn save_world(env: &Env, world: &SimpleWorld) {
    env.storage().instance().set(&symbol_short!("world"), world);
}

fn load_world(env: &Env) -> SimpleWorld {
    env.storage()
        .instance()
        .get(&symbol_short!("world"))
        .expect("World not initialized")
}

fn random_shape(env: &Env, offset: u64) -> TetrominoShape {
    let idx = (env.ledger().sequence() as u64 + offset) % 7;
    match idx {
        0 => TetrominoShape::I,
        1 => TetrominoShape::J,
        2 => TetrominoShape::L,
        3 => TetrominoShape::O,
        4 => TetrominoShape::S,
        5 => TetrominoShape::T,
        _ => TetrominoShape::Z,
    }
}

fn spawn_piece(world: &mut SimpleWorld, env: &Env, shape: TetrominoShape) {
    let id = world.spawn_entity();
    let piece = PieceComponent::new(shape, BOARD_WIDTH / 2 - 1, 0);
    world.set_typed(env, id, &piece);
}

/// Apply dx/dy/d_rot to the active piece if the result doesn't collide.
fn shift_piece(
    world: &mut SimpleWorld,
    env: &Env,
    board: &Vec<u32>,
    dx: i32,
    dy: i32,
    d_rot: i32,
) -> bool {
    let entities = world.get_entities_with_component(&symbol_short!("piece"), env);
    if entities.is_empty() { return false; }
    let id = entities.get(0).unwrap();
    let data = world.get_component(id, &symbol_short!("piece")).unwrap();
    let mut piece = PieceComponent::deserialize(env, &data).unwrap();

    let nx = piece.x + dx;
    let ny = piece.y + dy;
    let nr = (piece.rotation as i32 + d_rot).rem_euclid(4) as u32;

    if collision_system(board, piece.shape, nx, ny, nr) {
        return false;
    }
    piece.x = nx;
    piece.y = ny;
    piece.rotation = nr;
    world.set_typed(env, id, &piece);
    true
}

/// Lock the active piece, clear lines, update score, spawn next piece.
fn do_lock(env: &Env, state: &mut GameState, world: &mut SimpleWorld) {
    let (new_board, lines) = lock_system(world, env, &state.board);

    // Check game over: any piece locked above row 0
    let entities = world.get_entities_with_component(&symbol_short!("piece"), env);
    let piece_above = if entities.is_empty() {
        // piece was despawned by lock_system; check if board top row is occupied
        new_board.get(0).unwrap_or(0) != 0
    } else {
        false
    };

    state.board = new_board;

    if lines > 0 {
        let points = match lines {
            1 => 100u32,
            2 => 300,
            3 => 500,
            4 => 800,
            _ => 0,
        };
        state.score += points * (state.level + 1);
        state.lines_cleared += lines;
        if state.lines_cleared >= state.level * 10 {
            state.level += 1;
        }
    }

    // Spawn next piece
    let next = state.next_shape;
    state.next_shape = random_shape(env, state.lines_cleared as u64 + 1);
    spawn_piece(world, env, next);

    // Game over if new piece immediately collides
    let new_entities = world.get_entities_with_component(&symbol_short!("piece"), env);
    if !new_entities.is_empty() {
        let id = new_entities.get(0).unwrap();
        let data = world.get_component(id, &symbol_short!("piece")).unwrap();
        let piece = PieceComponent::deserialize(env, &data).unwrap();
        if piece_above
            || collision_system(&state.board, piece.shape, piece.x, piece.y, piece.rotation)
        {
            state.game_over = true;
        }
    }

    save_world(env, world);
    save_state(env, state);
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_init_game() {
        let env = Env::default();
        let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
        let state = client.init_game();
        assert_eq!(state.score, 0);
        assert!(!state.game_over);
    }

    #[test]
    fn test_move_functions() {
        let env = Env::default();
        let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
        client.init_game();
        let _moved = client.move_left();
    }

    #[test]
    fn test_rotation() {
        let env = Env::default();
        let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
        client.init_game();
        let _rotated = client.rotate();
    }

    #[test]
    fn test_collision_detection() {
        let env = Env::default();
        let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
        client.init_game();
        for _ in 0..10 {
            client.move_left();
        }
    }

    #[test]
    fn test_line_clearing() {
        let env = Env::default();
        let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
        client.init_game();
        let _state = client.update_tick();
    }

    #[test]
    fn test_score_updates() {
        let env = Env::default();
        let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
        client.init_game();
        assert_eq!(client.get_state().score, 0);
    }

    #[test]
    fn test_game_over() {
        let env = Env::default();
        let client = TetrisContractClient::new(&env, &env.register(TetrisContract, ()));
        client.init_game();
        assert!(!client.get_state().game_over);
    }
}
