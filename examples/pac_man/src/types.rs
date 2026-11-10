pub const MAZE_WIDTH: u32 = 10;
pub const MAZE_HEIGHT: u32 = 10;
pub const PELLET_POINTS: u32 = 10;
pub const POWER_PELLET_POINTS: u32 = 50;
pub const GHOST_POINTS: u32 = 200;
pub const POWER_MODE_DURATION: u32 = 10;
pub const INITIAL_LIVES: u32 = 3;
pub const PACMAN_ENTITY_ID: u64 = 0;
pub const GHOST_ENTITY_ID_START: u64 = 1;

use soroban_sdk::{contracterror, contracttype, Vec};

/// Direction of movement for Pac-Man and ghosts
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

/// Ghost behavior mode
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum GhostMode {
    Chase = 0,
    Frightened = 1,
}

/// Type of cell in the maze grid
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum CellType {
    Empty = 0,
    Wall = 1,
    Pellet = 2,
    PowerPellet = 3,
}

/// Error types for the Pac-Man game
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum GameError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    GameOver = 3,
    InvalidDirection = 4,
    InvalidPosition = 5,
}

use crate::components::{Ghost, Position};
/// Storage keys
use cougr_core::event::Event;

/// Complete game state stored on-chain
#[contracttype]
#[derive(Clone, Debug)]
pub struct GameState {
    pub pacman_pos: Position,
    pub pacman_dir: Direction,
    pub pacman_start: Position,
    pub ghosts: Vec<Ghost>,
    pub maze: Vec<CellType>,
    pub score: u32,
    pub lives: u32,
    pub game_over: bool,
    pub won: bool,
    pub power_mode_timer: u32,
    pub pellets_remaining: u32,
    pub last_collision_events: Vec<Event>,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    GameState,
    Initialized,
}
