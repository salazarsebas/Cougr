#![no_std]

//! # Pac-Man On-Chain Game
//!
//! A practical example demonstrating how to use `cougr-core` to implement
//! on-chain game logic on the Stellar blockchain via Soroban.
//!
//! This contract implements a complete Pac-Man game with:
//! - A 10x10 maze with walls, pellets, and power pellets
//! - Pac-Man movement and direction control
//! - Ghost AI with chase and frightened modes
//! - Score tracking and lives system
//! - Win/lose conditions
//!
//! ## Usage
//!
//! 1. Deploy the contract to Stellar Testnet
//! 2. Call `init_game` to start a new game
//! 3. Call `change_direction` to control Pac-Man
//! 4. Call `update_tick` to advance the game state
//! 5. Query `get_score`, `get_lives`, etc. to check game status

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Env, Vec};

// Import cougr-core for ECS patterns and utilities
// The cougr-core package provides Entity-Component-System patterns optimized
// for Soroban smart contracts, simplifying on-chain game development.
use cougr_core::component::{ComponentTrait, Position as CorePosition};
use cougr_core::event::{CollisionEvent, Event, EventTrait};

// =============================================================================
// Constants
// =============================================================================

/// Width of the game maze
const MAZE_WIDTH: u32 = 10;

/// Height of the game maze
const MAZE_HEIGHT: u32 = 10;

/// Points awarded for eating a regular pellet
const PELLET_POINTS: u32 = 10;

/// Points awarded for eating a power pellet
const POWER_PELLET_POINTS: u32 = 50;

/// Points awarded for eating a ghost in frightened mode
const GHOST_POINTS: u32 = 200;

/// Duration of power mode in ticks
const POWER_MODE_DURATION: u32 = 10;

/// Initial number of lives
const INITIAL_LIVES: u32 = 3;

// =============================================================================
// Storage Keys
// =============================================================================

/// Keys for persistent contract storage
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// The main game state
    GameState,
    /// Flag indicating if game has been initialized
    Initialized,
}

// =============================================================================
// Error Types
// =============================================================================

/// Contract errors for the Pac-Man game
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum GameError {
    /// Game has already been initialized
    AlreadyInitialized = 1,
    /// Game has not been initialized yet
    NotInitialized = 2,
    /// Game is already over
    GameOver = 3,
    /// Invalid direction provided
    InvalidDirection = 4,
    /// Invalid position on the maze
    InvalidPosition = 5,
}

// =============================================================================
// Game Types
// =============================================================================

/// Direction of movement for Pac-Man and ghosts
///
/// Using an enum ensures type safety and clear intent when handling
/// movement logic. The values are chosen to allow easy coordinate updates.
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
///
/// Ghosts alternate between chasing Pac-Man and fleeing when
/// Pac-Man eats a power pellet.
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum GhostMode {
    /// Ghost actively chases Pac-Man
    Chase = 0,
    /// Ghost flees from Pac-Man (after power pellet)
    Frightened = 1,
}

/// Type of cell in the maze grid
///
/// The maze is represented as a flat array where each cell can be
/// one of these types. This allows efficient storage on-chain.
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum CellType {
    /// Empty space - can be traversed
    Empty = 0,
    /// Wall - blocks movement
    Wall = 1,
    /// Regular pellet - awards points when eaten
    Pellet = 2,
    /// Power pellet - activates frightened mode for ghosts
    PowerPellet = 3,
}

/// Position in the 2D maze grid
///
/// Coordinates use i32 to allow for easier boundary calculations
/// and potential negative positions during movement math.
///
/// This extends cougr_core::component::Position with maze-specific
/// helper methods for index conversion.
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    /// Create a new position
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Convert position to array index for maze storage
    ///
    /// The maze is stored as a flat array in row-major order.
    pub fn to_index(&self) -> u32 {
        (self.y as u32) * MAZE_WIDTH + (self.x as u32)
    }

    /// Create position from array index
    pub fn from_index(index: u32) -> Self {
        Self {
            x: (index % MAZE_WIDTH) as i32,
            y: (index / MAZE_WIDTH) as i32,
        }
    }

    /// Convert to cougr_core Position for ECS integration
    pub fn to_core_position(&self) -> CorePosition {
        CorePosition::new(self.x, self.y)
    }

    /// Create from cougr_core Position
    pub fn from_core_position(core_pos: &CorePosition) -> Self {
        Self {
            x: core_pos.x,
            y: core_pos.y,
        }
    }
}

/// PacMan Component
/// Separates player-specific state for ECS decomposition.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PacMan {
    pub position: Position,
    pub direction: Direction,
    pub start_position: Position,
}

impl PacMan {
    pub fn new(pos: Position) -> Self {
        Self {
            position: pos,
            direction: Direction::Right,
            start_position: pos,
        }
    }
}

/// Maze Component
/// Encapsulates the world grid and collectible tracking.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Maze {
    pub grid: Vec<CellType>,
    pub pellets_remaining: u32,
}

/// GameStats Component
/// Acts as a resource for global game metrics.
#[contracttype]
#[derive(Clone, Debug)]
pub struct GameStats {
    pub score: u32,
    pub lives: u32,
    pub power_mode_timer: u32,
}

impl GameStats {
    pub fn new() -> Self {
        Self {
            score: 0,
            lives: INITIAL_LIVES,
            power_mode_timer: 0,
        }
    }
}

/// GameStatus Component
/// Tracks the progression and lifecycle of the match.
#[contracttype]
#[derive(Clone, Debug)]
pub struct GameStatus {
    pub game_over: bool,
    pub won: bool,
}

impl GameStatus {
    pub fn new() -> Self {
        Self { game_over: false, won: false }
    }
}

/// Ghost entity with position and behavior state
///
/// Each ghost maintains its own position, direction, and mode.
/// The start_position is used to respawn the ghost when eaten.
/// Uses cougr_core entity patterns with a unique entity_id.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Ghost {
    /// Unique entity ID for this ghost (used with cougr_core CollisionEvent)
    pub entity_id: u64,
    pub position: Position,
    pub direction: Direction,
    pub mode: GhostMode,
    pub frightened_timer: u32,
    pub start_position: Position,
}

/// Entity ID for Pac-Man (constant for collision events)
const PACMAN_ENTITY_ID: u64 = 0;

/// Starting entity ID for ghosts
const GHOST_ENTITY_ID_START: u64 = 1;

impl Ghost {
    /// Create a new ghost at the given position with a unique entity ID
    pub fn new(entity_id: u64, x: i32, y: i32) -> Self {
        let pos = Position::new(x, y);
        Self {
            entity_id,
            position: pos,
            direction: Direction::Up,
            mode: GhostMode::Chase,
            frightened_timer: 0,
            start_position: pos,
        }
    }

    /// Reset ghost to starting position in chase mode
    pub fn respawn(&mut self) {
        self.position = self.start_position;
        self.mode = GhostMode::Chase;
        self.frightened_timer = 0;
    }

    /// Create a CollisionEvent between this ghost and Pac-Man
    /// Uses cougr_core's CollisionEvent for standardized event handling
    pub fn create_collision_event(&self) -> CollisionEvent {
        CollisionEvent::new(PACMAN_ENTITY_ID, self.entity_id, symbol_short!("ghost"))
    }
}

/// Complete game state stored on-chain
///
/// This struct contains all data needed to represent the current
/// state of a Pac-Man game. It is stored in persistent storage
/// and updated with each game action.
///
/// Uses cougr_core patterns for entity management and event tracking.
#[contracttype]
#[derive(Clone, Debug)]
pub struct GameState {
    pub pacman: PacMan,
    pub ghosts: Vec<Ghost>,
    pub maze: Maze,
    pub stats: GameStats,
    pub status: GameStatus,
    pub last_collision_events: Vec<Event>,
}

// =============================================================================
// Contract Definition
// =============================================================================

/// Pac-Man game contract
///
/// This contract demonstrates how to build on-chain game logic using
/// cougr-core and Soroban. It handles persistent game state, player
/// input, and game mechanics entirely on the Stellar blockchain.
#[contract]
pub struct PacManContract;

#[contractimpl]
impl PacManContract {
    // =========================================================================
    // Initialization
    // =========================================================================

    /// Initialize a new Pac-Man game
    ///
    /// Creates the maze, places Pac-Man and ghosts at starting positions,
    /// and sets up initial game state. Can only be called once per contract
    /// instance.
    ///
    /// # Returns
    /// The initial game state
    ///
    /// # Panics
    /// Panics if the game has already been initialized
    pub fn init_game(env: Env) -> GameState {
        // Check if already initialized
        if env.storage().instance().has(&DataKey::Initialized) {
            panic!("Game already initialized");
        }

        let grid = Self::create_maze(&env);
        let mut pellet_count: u32 = 0;
        for i in 0..grid.len() {
            let cell = grid.get(i).unwrap();
            if cell == CellType::Pellet || cell == CellType::PowerPellet {
                pellet_count += 1;
            }
        }

        let maze = Maze { grid, pellets_remaining: pellet_count };

        let mut ghosts: Vec<Ghost> = Vec::new(&env);
        ghosts.push_back(Ghost::new(GHOST_ENTITY_ID_START, 4, 4));
        ghosts.push_back(Ghost::new(GHOST_ENTITY_ID_START + 1, 5, 4));
        ghosts.push_back(Ghost::new(GHOST_ENTITY_ID_START + 2, 4, 5));
        ghosts.push_back(Ghost::new(GHOST_ENTITY_ID_START + 3, 5, 5));

        let pacman = PacMan::new(Position::new(1, 1));
        let collision_events: Vec<Event> = Vec::new(&env);

        let state = GameState {
            pacman,
            ghosts,
            maze,
            stats: GameStats::new(),
            status: GameStatus::new(),
            last_collision_events: collision_events,
        };

        // Store game state
        env.storage().instance().set(&DataKey::GameState, &state);
        env.storage().instance().set(&DataKey::Initialized, &true);

        // Extend TTL to prevent archival
        env.storage().instance().extend_ttl(50000, 100000);

        state
    }

    // =========================================================================
    // Game Actions
    // =========================================================================

    /// Change Pac-Man's direction
    ///
    /// Updates the direction Pac-Man will move on the next tick.
    /// The direction change takes effect immediately.
    ///
    /// # Arguments
    /// * `direction` - The new direction for Pac-Man
    ///
    /// # Panics
    /// Panics if the game is over or not initialized
    pub fn change_direction(env: Env, direction: Direction) {
        let mut state = Self::get_state(&env);

        if state.status.game_over {
            panic!("Game is over");
        }

        state.pacman.direction = direction;
        env.storage().instance().set(&DataKey::GameState, &state);
    }

    /// Advance the game by one tick
    ///
    /// This is the main game loop function. It:
    /// 1. Moves Pac-Man in the current direction
    /// 2. Checks for pellet collection
    /// 3. Moves all ghosts according to their AI
    /// 4. Checks for collisions between Pac-Man and ghosts
    /// 5. Updates timers and game state
    ///
    /// # Returns
    /// The updated game state after the tick
    ///
    /// # Panics
    /// Panics if the game is over or not initialized
    pub fn update_tick(env: Env) -> GameState {
        let mut state = Self::get_state(&env);

        if state.status.game_over {
            panic!("Game is over");
        }

        // Execute Systems
        systems::player_movement_system(&mut state.pacman, &state.maze);
        systems::collectible_system(&mut state.pacman, &mut state.maze, &mut state.stats, &mut state.ghosts);
        systems::ghost_movement_system(&state.pacman, &mut state.ghosts, &state.maze);
        systems::collision_system(&env, &mut state.pacman, &mut state.ghosts, &mut state.stats, &mut state.status, &mut state.last_collision_events);

        // Progress and State Management Systems
        systems::game_progress_system::handle_power_timer(&mut state.stats, &mut state.ghosts);
        systems::game_progress_system::check_status(&mut state.maze, &mut state.status);

        // Save updated state
        env.storage().instance().set(&DataKey::GameState, &state);

        state
    }

    /// Manually eat a pellet at the current position
    ///
    /// This function is provided for explicit pellet eating. Note that
    /// `update_tick` already handles pellet collection automatically.
    ///
    /// # Returns
    /// The points earned (0 if no pellet at current position)
    pub fn eat_pellet(env: Env) -> u32 {
        let mut state = Self::get_state(&env);

        if state.status.game_over {
            return 0;
        }

        let idx = state.pacman.position.to_index();
        let cell = state.maze.grid.get(idx).unwrap();

        let points = match cell {
            CellType::Pellet => {
                state.maze.grid.set(idx, CellType::Empty);
                state.stats.score += PELLET_POINTS;
                state.maze.pellets_remaining -= 1;
                PELLET_POINTS
            }
            CellType::PowerPellet => {
                state.maze.grid.set(idx, CellType::Empty);
                state.stats.score += POWER_PELLET_POINTS;
                state.maze.pellets_remaining -= 1;
                systems::collectible_system::activate_power_mode(&mut state.stats, &mut state.ghosts);
                POWER_PELLET_POINTS
            }
            _ => 0,
        };

        if points > 0 {
            env.storage().instance().set(&DataKey::GameState, &state);
        }

        points
    }

    // =========================================================================
    // Query Functions
    // =========================================================================

    /// Get the current score
    pub fn get_score(env: Env) -> u32 {
        Self::get_state(&env).stats.score
    }

    /// Get the remaining lives
    pub fn get_lives(env: Env) -> u32 {
        Self::get_state(&env).stats.lives
    }

    /// Get Pac-Man's current position
    pub fn get_pacman_position(env: Env) -> Position {
        Self::get_state(&env).pacman.position
    }

    /// Get the current maze state
    pub fn get_maze(env: Env) -> Vec<CellType> {
        Self::get_state(&env).maze.grid
    }

    /// Get the complete game state
    pub fn get_game_state(env: Env) -> GameState {
        Self::get_state(&env)
    }

    /// Check if the game is over and whether the player won
    ///
    /// # Returns
    /// A tuple of (game_over, won)
    pub fn check_game_over(env: Env) -> (bool, bool) {
        let state = Self::get_state(&env).status;
        (state.game_over, state.won)
    }

    /// Get the last collision events
    ///
    /// Returns collision events from the most recent tick, using
    /// cougr_core's Event system for standardized event handling.
    pub fn get_collision_events(env: Env) -> Vec<Event> {
        Self::get_state(&env).last_collision_events
    }

    /// Get Pac-Man's position as a cougr_core Position component
    ///
    /// Demonstrates integration with cougr_core's component system,
    /// returning a serialized Position using ComponentTrait.
    pub fn get_pacman_core_position(env: Env) -> CorePosition {
        let state = Self::get_state(&env);
        state.pacman.position.to_core_position()
    }

    /// Serialize Pac-Man's position using cougr_core ComponentTrait
    ///
    /// This demonstrates how to use cougr_core's serialization patterns
    /// for component data, enabling ECS-style data handling.
    pub fn get_serialized_pacman_position(env: Env) -> soroban_sdk::Bytes {
        let state = Self::get_state(&env);
        let core_pos = state.pacman.position.to_core_position();
        // Use cougr_core's ComponentTrait for serialization
        core_pos.serialize(&env)
    }

    // =========================================================================
    // Internal Helper Functions
    // =========================================================================

    /// Get the current game state from storage
    fn get_state(env: &Env) -> GameState {
        env.storage()
            .instance()
            .get(&DataKey::GameState)
            .expect("Game not initialized")
    }

    /// Create the initial maze layout
    ///
    /// The maze is a 10x10 grid with walls forming a navigable pattern.
    /// Power pellets are placed in the corners.
    ///
    /// Layout:
    /// ##########
    /// #P......P#
    /// #.##.##..#
    /// #.#...#..#
    /// #...#....#
    /// #.#.#.##.#
    /// #.#......#
    /// #.##.###.#
    /// #P......P#
    /// ##########
    fn create_maze(env: &Env) -> Vec<CellType> {
        let mut maze: Vec<CellType> = Vec::new(env);

        // Define maze layout as a string for clarity
        // # = Wall, . = Pellet, P = Power Pellet, ' ' = Empty
        let layout: [&str; 10] = [
            "##########",
            "#P......P#",
            "#.##.##..#",
            "#.#...#..#",
            "#...#....#",
            "#.#.#.##.#",
            "#.#......#",
            "#.##.###.#",
            "#P......P#",
            "##########",
        ];

        for row in layout.iter() {
            for ch in row.chars() {
                let cell = match ch {
                    '#' => CellType::Wall,
                    '.' => CellType::Pellet,
                    'P' => CellType::PowerPellet,
                    _ => CellType::Empty,
                };
                maze.push_back(cell);
            }
        }

        maze
    }
}

/// ECS Systems for Pac-Man
/// Logic is decomposed into individual systems according to the Cougr pattern.
mod systems {
    use super::*;

    /// PlayerMovementSystem: Handles Pac-Man movement and maze boundary wrapping.
    pub fn player_movement_system(pacman: &mut PacMan, maze: &Maze) {
        let mut new_pos = pacman.position;

        match pacman.direction {
            Direction::Up => new_pos.y -= 1,
            Direction::Down => new_pos.y += 1,
            Direction::Left => new_pos.x -= 1,
            Direction::Right => new_pos.x += 1,
        }

        new_pos = wrap_position(new_pos);

        let idx = new_pos.to_index();
        if maze.grid.get(idx).unwrap() != CellType::Wall {
            pacman.position = new_pos;
        }
    }

    /// GhostMovementSystem: Logic for ghost AI, alternating between chase and frightened.
    pub fn ghost_movement_system(pacman: &PacMan, ghosts: &mut Vec<Ghost>, maze: &Maze) {
        for i in 0..ghosts.len() {
            let mut ghost = ghosts.get(i).unwrap();
            update_ghost(&mut ghost, pacman.position, maze);
            ghosts.set(i, ghost);
        }
    }

    fn wrap_position(mut new_pos: Position) -> Position {
        if new_pos.x < 0 {
            new_pos.x = (MAZE_WIDTH - 1) as i32;
        } else if new_pos.x >= MAZE_WIDTH as i32 {
            new_pos.x = 0;
        }
        if new_pos.y < 0 {
            new_pos.y = (MAZE_HEIGHT - 1) as i32;
        } else if new_pos.y >= MAZE_HEIGHT as i32 {
            new_pos.y = 0;
        }
        new_pos
    }

    fn update_ghost(ghost: &mut Ghost, pacman_pos: Position, maze: &Maze) {
        if ghost.frightened_timer > 0 {
            ghost.frightened_timer -= 1;
            if ghost.frightened_timer == 0 {
                ghost.mode = GhostMode::Chase;
            }
        }

        ghost.direction = calculate_ghost_direction(ghost, pacman_pos, maze);

        let mut new_pos = ghost.position;
        apply_direction(&mut new_pos, ghost.direction);
        new_pos = wrap_position(new_pos);

        if maze.grid.get(new_pos.to_index()).unwrap() != CellType::Wall {
            ghost.position = new_pos;
        }
    }

    fn apply_direction(pos: &mut Position, dir: Direction) {
        match dir {
            Direction::Up => pos.y -= 1,
            Direction::Down => pos.y += 1,
            Direction::Left => pos.x -= 1,
            Direction::Right => pos.x += 1,
        }
    }

    fn calculate_ghost_direction(ghost: &Ghost, pacman_pos: Position, maze: &Maze) -> Direction {
        let directions = [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];
        let mut best_dir = ghost.direction;
        let mut best_score: i32 = i32::MIN;

        for &dir in directions.iter() {
            let mut test_pos = ghost.position;
            apply_direction(&mut test_pos, dir);
            test_pos = wrap_position(test_pos);

            if maze.grid.get(test_pos.to_index()).unwrap() == CellType::Wall {
                continue;
            }

            let new_dx = pacman_pos.x - test_pos.x;
            let new_dy = pacman_pos.y - test_pos.y;
            let distance = new_dx.abs() + new_dy.abs();

            let score = match ghost.mode {
                GhostMode::Chase => -distance,
                GhostMode::Frightened => distance,
            };

            if score > best_score {
                best_score = score;
                best_dir = dir;
            }
        }
        best_dir
    }

    /// CollectibleSystem: Manages pellet and power pellet consumption.
    pub fn collectible_system(pacman: &mut PacMan, maze: &mut Maze, stats: &mut GameStats, ghosts: &mut Vec<Ghost>) {
        let idx = pacman.position.to_index();
        let cell = maze.grid.get(idx).unwrap();

        match cell {
            CellType::Pellet => {
                maze.grid.set(idx, CellType::Empty);
                stats.score += PELLET_POINTS;
                maze.pellets_remaining -= 1;
            }
            CellType::PowerPellet => {
                maze.grid.set(idx, CellType::Empty);
                stats.score += POWER_PELLET_POINTS;
                maze.pellets_remaining -= 1;
                activate_power_mode(stats, ghosts);
            }
            _ => {}
        }
    }

    pub fn activate_power_mode(stats: &mut GameStats, ghosts: &mut Vec<Ghost>) {
        stats.power_mode_timer = POWER_MODE_DURATION;
        for i in 0..ghosts.len() {
            let mut ghost = ghosts.get(i).unwrap();
            ghost.mode = GhostMode::Frightened;
            ghost.frightened_timer = POWER_MODE_DURATION;
            ghosts.set(i, ghost);
        }
    }

    /// CollisionSystem: Checks for interactions between Pac-Man and Ghosts.
    pub fn collision_system(
        env: &Env,
        pacman: &mut PacMan,
        ghosts: &mut Vec<Ghost>,
        stats: &mut GameStats,
        status: &mut GameStatus,
        events: &mut Vec<Event>,
    ) {
        *events = Vec::new(env);

        for i in 0..ghosts.len() {
            let mut ghost = ghosts.get(i).unwrap();
            if ghost.position == pacman.position {
                let event = Event::new(
                    CollisionEvent::event_type(),
                    ghost.create_collision_event().serialize(env),
                );
                events.push_back(event);

                match ghost.mode {
                    GhostMode::Chase => {
                        stats.lives -= 1;
                        if stats.lives == 0 {
                            status.game_over = true;
                            status.won = false;
                        } else {
                            pacman.position = pacman.start_position;
                            pacman.direction = Direction::Right;
                        }
                    }
                    GhostMode::Frightened => {
                        stats.score += GHOST_POINTS;
                        ghost.respawn();
                        ghosts.set(i, ghost);
                    }
                }
            }
        }
    }

    /// GameProgressSystem: Detects global level transitions.
    pub mod game_progress_system {
        use super::*;

        pub fn check_status(maze: &mut Maze, status: &mut GameStatus) {
            if maze.pellets_remaining == 0 {
                status.game_over = true;
                status.won = true;
            }
        }

        /// Updates power mode timers and reverts ghost behavior when expired.
        pub fn handle_power_timer(stats: &mut GameStats, ghosts: &mut Vec<Ghost>) {
            if stats.power_mode_timer > 0 {
                stats.power_mode_timer -= 1;
                if stats.power_mode_timer == 0 {
                    end_frightened_mode(ghosts);
                }
            }
        }

        pub fn end_frightened_mode(ghosts: &mut Vec<Ghost>) {
            for i in 0..ghosts.len() {
                let mut ghost = ghosts.get(i).unwrap();
                ghost.mode = GhostMode::Chase;
                ghost.frightened_timer = 0;
                ghosts.set(i, ghost);
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod test;
