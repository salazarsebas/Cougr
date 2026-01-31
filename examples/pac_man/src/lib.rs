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
    /// Current position of Pac-Man
    pub pacman_pos: Position,
    /// Current direction Pac-Man is facing
    pub pacman_dir: Direction,
    /// Starting position for respawns
    pub pacman_start: Position,
    /// Array of ghost entities (each with unique entity_id for cougr_core)
    pub ghosts: Vec<Ghost>,
    /// Flat array representing the maze (row-major order)
    pub maze: Vec<CellType>,
    /// Current score
    pub score: u32,
    /// Remaining lives
    pub lives: u32,
    /// Whether the game has ended
    pub game_over: bool,
    /// Whether the player won (all pellets collected)
    pub won: bool,
    /// Remaining ticks of power mode
    pub power_mode_timer: u32,
    /// Number of pellets remaining to collect
    pub pellets_remaining: u32,
    /// Last collision events (cougr_core Event system integration)
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

        // Create the maze layout
        let maze = Self::create_maze(&env);

        // Count pellets in the maze
        let mut pellet_count: u32 = 0;
        for i in 0..maze.len() {
            let cell = maze.get(i).unwrap();
            if cell == CellType::Pellet || cell == CellType::PowerPellet {
                pellet_count += 1;
            }
        }

        // Create ghosts at their starting positions
        let mut ghosts: Vec<Ghost> = Vec::new(&env);
        // Create ghosts with unique entity IDs for cougr_core collision tracking
        ghosts.push_back(Ghost::new(GHOST_ENTITY_ID_START, 4, 4)); // Ghost 1 - center area
        ghosts.push_back(Ghost::new(GHOST_ENTITY_ID_START + 1, 5, 4)); // Ghost 2 - center area
        ghosts.push_back(Ghost::new(GHOST_ENTITY_ID_START + 2, 4, 5)); // Ghost 3 - center area
        ghosts.push_back(Ghost::new(GHOST_ENTITY_ID_START + 3, 5, 5)); // Ghost 4 - center area

        // Pac-Man starting position
        let pacman_start = Position::new(1, 1);

        // Create initial game state
        // Initialize collision events vector using cougr_core Event system
        let collision_events: Vec<Event> = Vec::new(&env);

        let state = GameState {
            pacman_pos: pacman_start,
            pacman_dir: Direction::Right,
            pacman_start,
            ghosts,
            maze,
            score: 0,
            lives: INITIAL_LIVES,
            game_over: false,
            won: false,
            power_mode_timer: 0,
            pellets_remaining: pellet_count,
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

        if state.game_over {
            panic!("Game is over");
        }

        state.pacman_dir = direction;
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

        if state.game_over {
            panic!("Game is over");
        }

        // Move Pac-Man
        Self::move_pacman(&env, &mut state);

        // Check for pellet collection at new position
        Self::check_pellet_collection(&env, &mut state);

        // Move ghosts
        Self::move_ghosts(&env, &mut state);

        // Check for ghost collisions
        Self::check_ghost_collisions(&env, &mut state);

        // Decrement power mode timer
        if state.power_mode_timer > 0 {
            state.power_mode_timer -= 1;

            // End frightened mode when timer expires
            if state.power_mode_timer == 0 {
                Self::end_frightened_mode(&env, &mut state);
            }
        }

        // Check win condition
        if state.pellets_remaining == 0 {
            state.game_over = true;
            state.won = true;
        }

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

        if state.game_over {
            return 0;
        }

        let idx = state.pacman_pos.to_index();
        let cell = state.maze.get(idx).unwrap();

        let points = match cell {
            CellType::Pellet => {
                state.maze.set(idx, CellType::Empty);
                state.score += PELLET_POINTS;
                state.pellets_remaining -= 1;
                PELLET_POINTS
            }
            CellType::PowerPellet => {
                state.maze.set(idx, CellType::Empty);
                state.score += POWER_PELLET_POINTS;
                state.pellets_remaining -= 1;
                Self::activate_power_mode(&env, &mut state);
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
        Self::get_state(&env).score
    }

    /// Get the remaining lives
    pub fn get_lives(env: Env) -> u32 {
        Self::get_state(&env).lives
    }

    /// Get Pac-Man's current position
    pub fn get_pacman_position(env: Env) -> Position {
        Self::get_state(&env).pacman_pos
    }

    /// Get the current maze state
    pub fn get_maze(env: Env) -> Vec<CellType> {
        Self::get_state(&env).maze
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
        let state = Self::get_state(&env);
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
        state.pacman_pos.to_core_position()
    }

    /// Serialize Pac-Man's position using cougr_core ComponentTrait
    ///
    /// This demonstrates how to use cougr_core's serialization patterns
    /// for component data, enabling ECS-style data handling.
    pub fn get_serialized_pacman_position(env: Env) -> soroban_sdk::Bytes {
        let state = Self::get_state(&env);
        let core_pos = state.pacman_pos.to_core_position();
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

    /// Move Pac-Man in the current direction
    fn move_pacman(_env: &Env, state: &mut GameState) {
        let mut new_pos = state.pacman_pos;

        // Calculate new position based on direction
        match state.pacman_dir {
            Direction::Up => new_pos.y -= 1,
            Direction::Down => new_pos.y += 1,
            Direction::Left => new_pos.x -= 1,
            Direction::Right => new_pos.x += 1,
        }

        // Handle maze wrapping (tunnel behavior)
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

        // Check for wall collision
        let idx = new_pos.to_index();
        let cell = state.maze.get(idx).unwrap();

        if cell != CellType::Wall {
            state.pacman_pos = new_pos;
        }
        // If wall, Pac-Man stays in place
    }

    /// Check and handle pellet collection at Pac-Man's position
    fn check_pellet_collection(_env: &Env, state: &mut GameState) {
        let idx = state.pacman_pos.to_index();
        let cell = state.maze.get(idx).unwrap();

        match cell {
            CellType::Pellet => {
                state.maze.set(idx, CellType::Empty);
                state.score += PELLET_POINTS;
                state.pellets_remaining -= 1;
            }
            CellType::PowerPellet => {
                state.maze.set(idx, CellType::Empty);
                state.score += POWER_PELLET_POINTS;
                state.pellets_remaining -= 1;
                // Activate power mode inline
                state.power_mode_timer = POWER_MODE_DURATION;
                // Set all ghosts to frightened mode
                for i in 0..state.ghosts.len() {
                    let mut ghost = state.ghosts.get(i).unwrap();
                    ghost.mode = GhostMode::Frightened;
                    ghost.frightened_timer = POWER_MODE_DURATION;
                    state.ghosts.set(i, ghost);
                }
            }
            _ => {}
        }
    }

    /// Move all ghosts according to their AI behavior
    fn move_ghosts(_env: &Env, state: &mut GameState) {
        let pacman_pos = state.pacman_pos;

        for i in 0..state.ghosts.len() {
            let mut ghost = state.ghosts.get(i).unwrap();

            // Update frightened timer
            if ghost.frightened_timer > 0 {
                ghost.frightened_timer -= 1;
                if ghost.frightened_timer == 0 {
                    ghost.mode = GhostMode::Chase;
                }
            }

            // Calculate best direction based on mode
            let new_dir = Self::calculate_ghost_direction(state, &ghost, pacman_pos);
            ghost.direction = new_dir;

            // Calculate new position
            let mut new_pos = ghost.position;
            match ghost.direction {
                Direction::Up => new_pos.y -= 1,
                Direction::Down => new_pos.y += 1,
                Direction::Left => new_pos.x -= 1,
                Direction::Right => new_pos.x += 1,
            }

            // Handle wrapping
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

            // Check for wall collision
            let idx = new_pos.to_index();
            let cell = state.maze.get(idx).unwrap();

            if cell != CellType::Wall {
                ghost.position = new_pos;
            }

            state.ghosts.set(i, ghost);
        }
    }

    /// Calculate the best direction for a ghost to move
    ///
    /// In Chase mode, ghosts move toward Pac-Man.
    /// In Frightened mode, ghosts move away from Pac-Man.
    fn calculate_ghost_direction(
        state: &GameState,
        ghost: &Ghost,
        pacman_pos: Position,
    ) -> Direction {
        // Get all possible directions
        let directions = [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];
        let mut best_dir = ghost.direction;
        let mut best_score: i32 = i32::MIN;

        for dir in directions.iter() {
            // Calculate new position for this direction
            let mut test_pos = ghost.position;
            match dir {
                Direction::Up => test_pos.y -= 1,
                Direction::Down => test_pos.y += 1,
                Direction::Left => test_pos.x -= 1,
                Direction::Right => test_pos.x += 1,
            }

            // Handle wrapping
            if test_pos.x < 0 {
                test_pos.x = (MAZE_WIDTH - 1) as i32;
            } else if test_pos.x >= MAZE_WIDTH as i32 {
                test_pos.x = 0;
            }
            if test_pos.y < 0 {
                test_pos.y = (MAZE_HEIGHT - 1) as i32;
            } else if test_pos.y >= MAZE_HEIGHT as i32 {
                test_pos.y = 0;
            }

            // Check if this position is a wall
            let idx = test_pos.to_index();
            let cell = state.maze.get(idx).unwrap();
            if cell == CellType::Wall {
                continue;
            }

            // Calculate Manhattan distance to Pac-Man
            let new_dx = pacman_pos.x - test_pos.x;
            let new_dy = pacman_pos.y - test_pos.y;
            let distance = new_dx.abs() + new_dy.abs();

            // Score based on mode
            let score = match ghost.mode {
                GhostMode::Chase => -distance,     // Minimize distance
                GhostMode::Frightened => distance, // Maximize distance
            };

            if score > best_score {
                best_score = score;
                best_dir = *dir;
            }
        }

        best_dir
    }

    /// Check for collisions between Pac-Man and ghosts
    /// Check for collisions between Pac-Man and ghosts
    ///
    /// Uses cougr_core's CollisionEvent to track collision events,
    /// enabling standardized event-driven game logic.
    fn check_ghost_collisions(env: &Env, state: &mut GameState) {
        let pacman_pos = state.pacman_pos;

        // Clear previous collision events
        state.last_collision_events = Vec::new(env);

        for i in 0..state.ghosts.len() {
            let mut ghost = state.ghosts.get(i).unwrap();

            if ghost.position == pacman_pos {
                // Create collision event using cougr_core's CollisionEvent
                let collision_event = ghost.create_collision_event();

                // Serialize collision event using cougr_core's EventTrait
                let event_data = collision_event.serialize(env);
                let event = Event::new(CollisionEvent::event_type(), event_data);
                state.last_collision_events.push_back(event);

                match ghost.mode {
                    GhostMode::Chase => {
                        // Pac-Man loses a life
                        state.lives -= 1;

                        if state.lives == 0 {
                            state.game_over = true;
                            state.won = false;
                        } else {
                            // Respawn Pac-Man at start
                            state.pacman_pos = state.pacman_start;
                            state.pacman_dir = Direction::Right;
                        }
                    }
                    GhostMode::Frightened => {
                        // Pac-Man eats the ghost
                        state.score += GHOST_POINTS;
                        ghost.respawn();
                        state.ghosts.set(i, ghost);
                    }
                }
            }
        }
    }

    /// Activate power mode (called when eating a power pellet)
    fn activate_power_mode(_env: &Env, state: &mut GameState) {
        state.power_mode_timer = POWER_MODE_DURATION;

        // Set all ghosts to frightened mode
        for i in 0..state.ghosts.len() {
            let mut ghost = state.ghosts.get(i).unwrap();
            ghost.mode = GhostMode::Frightened;
            ghost.frightened_timer = POWER_MODE_DURATION;
            state.ghosts.set(i, ghost);
        }
    }

    /// End frightened mode for all ghosts
    fn end_frightened_mode(_env: &Env, state: &mut GameState) {
        for i in 0..state.ghosts.len() {
            let mut ghost = state.ghosts.get(i).unwrap();
            ghost.mode = GhostMode::Chase;
            ghost.frightened_timer = 0;
            state.ghosts.set(i, ghost);
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod test;
