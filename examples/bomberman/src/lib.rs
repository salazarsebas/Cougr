#![no_std]
use cougr_core::component::ComponentTrait;
use cougr_core::*;
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Bytes, Env, Symbol, Vec};

// Game constants
const GRID_WIDTH: usize = 15;
const GRID_HEIGHT: usize = 13;
const INITIAL_LIVES: u32 = 3;
const BOMB_TIMER: u32 = 3;
const EXPLOSION_DURATION: u32 = 1;

// Component definitions for Bomberman game
#[contracttype]
#[derive(Clone)]
pub struct PlayerComponent {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub lives: u32,
    pub bomb_capacity: u32,
    pub score: u32,
}

impl PlayerComponent {
    pub fn new(id: u32, x: i32, y: i32) -> Self {
        Self {
            id,
            x,
            y,
            lives: INITIAL_LIVES,
            bomb_capacity: 1,
            score: 0,
        }
    }
}

impl ComponentTrait for PlayerComponent {
    fn component_type() -> Symbol {
        symbol_short!("player")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.id.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.x.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.y.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.lives.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.bomb_capacity.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.score.to_be_bytes()));
        bytes
    }

    #[allow(unused_variables)]
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 24 {
            return None;
        }
        let id = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let x = i32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);
        let y = i32::from_be_bytes([
            data.get(8).unwrap(),
            data.get(9).unwrap(),
            data.get(10).unwrap(),
            data.get(11).unwrap(),
        ]);
        let lives = u32::from_be_bytes([
            data.get(12).unwrap(),
            data.get(13).unwrap(),
            data.get(14).unwrap(),
            data.get(15).unwrap(),
        ]);
        let bomb_capacity = u32::from_be_bytes([
            data.get(16).unwrap(),
            data.get(17).unwrap(),
            data.get(18).unwrap(),
            data.get(19).unwrap(),
        ]);
        let score = u32::from_be_bytes([
            data.get(20).unwrap(),
            data.get(21).unwrap(),
            data.get(22).unwrap(),
            data.get(23).unwrap(),
        ]);
        Some(Self {
            id,
            x,
            y,
            lives,
            bomb_capacity,
            score,
        })
    }
}

#[contracttype]
#[derive(Clone)]
pub struct BombComponent {
    pub x: i32,
    pub y: i32,
    pub timer: u32,
    pub power: u32,
    pub owner_id: u32,
}

impl BombComponent {
    pub fn new(x: i32, y: i32, owner_id: u32) -> Self {
        Self {
            x,
            y,
            timer: BOMB_TIMER,
            power: 1,
            owner_id,
        }
    }
}

impl ComponentTrait for BombComponent {
    fn component_type() -> Symbol {
        symbol_short!("bomb")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.x.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.y.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.timer.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.power.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.owner_id.to_be_bytes()));
        bytes
    }

    #[allow(unused_variables)]
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 20 {
            return None;
        }
        let x = i32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let y = i32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);
        let timer = u32::from_be_bytes([
            data.get(8).unwrap(),
            data.get(9).unwrap(),
            data.get(10).unwrap(),
            data.get(11).unwrap(),
        ]);
        let power = u32::from_be_bytes([
            data.get(12).unwrap(),
            data.get(13).unwrap(),
            data.get(14).unwrap(),
            data.get(15).unwrap(),
        ]);
        let owner_id = u32::from_be_bytes([
            data.get(16).unwrap(),
            data.get(17).unwrap(),
            data.get(18).unwrap(),
            data.get(19).unwrap(),
        ]);
        Some(Self {
            x,
            y,
            timer,
            power,
            owner_id,
        })
    }
}

#[contracttype]
#[derive(Clone)]
pub struct ExplosionComponent {
    pub x: i32,
    pub y: i32,
    pub timer: u32,
}

impl ExplosionComponent {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            timer: EXPLOSION_DURATION,
        }
    }
}

impl ComponentTrait for ExplosionComponent {
    fn component_type() -> Symbol {
        symbol_short!("explosion")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.x.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.y.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.timer.to_be_bytes()));
        bytes
    }

    #[allow(unused_variables)]
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 12 {
            return None;
        }
        let x = i32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let y = i32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);
        let timer = u32::from_be_bytes([
            data.get(8).unwrap(),
            data.get(9).unwrap(),
            data.get(10).unwrap(),
            data.get(11).unwrap(),
        ]);
        Some(Self { x, y, timer })
    }
}

// Grid cell types
#[contracttype]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum CellType {
    Empty = 0,
    Wall = 1,
    Destructible = 2,
    PowerUp = 3,
}

#[contracttype]
#[derive(Clone)]
pub struct GridComponent {
    pub cells: Vec<CellType>,
}

impl GridComponent {
    #[allow(clippy::if_same_then_else)]
    pub fn new(env: &Env) -> Self {
        let mut cells = Vec::new(env);

        for _ in 0..(GRID_WIDTH * GRID_HEIGHT) {
            cells.push_back(CellType::Empty);
        }

        // Initialize walls around the perimeter
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let index = y * GRID_WIDTH + x;
                if x == 0 || x == GRID_WIDTH - 1 || y == 0 || y == GRID_HEIGHT - 1 {
                    cells.set(index as u32, CellType::Wall);
                } else if x % 2 == 0 && y % 2 == 0 {
                    cells.set(index as u32, CellType::Wall);
                }
            }
        }

        // Add some destructible blocks and power-ups
        for x in 1..GRID_WIDTH - 1 {
            for y in 1..GRID_HEIGHT - 1 {
                let index = y * GRID_WIDTH + x;
                if (x + y) % 3 == 0 && cells.get(index as u32).unwrap() == CellType::Empty {
                    cells.set(index as u32, CellType::Destructible);
                } else if (x + y) % 7 == 0 && cells.get(index as u32).unwrap() == CellType::Empty {
                    cells.set(index as u32, CellType::PowerUp);
                }
            }
        }

        Self { cells }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> CellType {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            self.cells
                .get((y * GRID_WIDTH + x) as u32)
                .unwrap_or(CellType::Wall)
        } else {
            CellType::Wall
        }
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell_type: CellType) {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            self.cells.set((y * GRID_WIDTH + x) as u32, cell_type);
        }
    }

    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= GRID_WIDTH as i32 || y >= GRID_HEIGHT as i32 {
            return false;
        }
        matches!(
            self.get_cell(x as usize, y as usize),
            CellType::Empty | CellType::PowerUp
        )
    }
}

impl ComponentTrait for GridComponent {
    fn component_type() -> Symbol {
        symbol_short!("grid")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        for cell in self.cells.iter() {
            bytes.append(&Bytes::from_array(env, &[cell as u8]));
        }
        bytes
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != (GRID_WIDTH * GRID_HEIGHT) as u32 {
            return None;
        }
        let mut cells = Vec::new(env);
        for i in 0..GRID_WIDTH * GRID_HEIGHT {
            let cell = match data.get(i as u32).unwrap() {
                0 => CellType::Empty,
                1 => CellType::Wall,
                2 => CellType::Destructible,
                3 => CellType::PowerUp,
                _ => return None,
            };
            cells.push_back(cell);
        }
        Some(Self { cells })
    }
}

#[contracttype]
#[derive(Clone)]
pub struct GameStateComponent {
    pub current_tick: u32,
    pub game_over: bool,
    pub winner_id: Option<u32>,
}

impl GameStateComponent {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            current_tick: 0,
            game_over: false,
            winner_id: None,
        }
    }
}

impl ComponentTrait for GameStateComponent {
    fn component_type() -> Symbol {
        symbol_short!("gstate")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.current_tick.to_be_bytes()));
        bytes.append(&Bytes::from_array(
            env,
            &[if self.game_over { 1 } else { 0 }],
        ));
        match self.winner_id {
            Some(id) => {
                bytes.append(&Bytes::from_array(env, &[1]));
                bytes.append(&Bytes::from_array(env, &id.to_be_bytes()));
            }
            None => {
                bytes.append(&Bytes::from_array(env, &[0]));
            }
        }
        bytes
    }

    #[allow(unused_variables)]
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 9 {
            return None;
        }
        let current_tick = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let game_over = data.get(4).unwrap() != 0;
        let has_winner = data.get(5).unwrap() != 0;
        let winner_id = if has_winner && data.len() >= 13 {
            Some(u32::from_be_bytes([
                data.get(6).unwrap(),
                data.get(7).unwrap(),
                data.get(8).unwrap(),
                data.get(9).unwrap(),
            ]))
        } else {
            None
        };
        Some(Self {
            current_tick,
            game_over,
            winner_id,
        })
    }
}

#[contract]
pub struct BombermanContract;

#[allow(unused_variables)]
#[contractimpl]
impl BombermanContract {
    /// Initialize the game world using cougr-core ECS
    /// This demonstrates how cougr-core simplifies persistent game state management
    /// compared to vanilla Soroban where you'd manually handle storage keys and serialization
    ///
    /// Benefits of using cougr-core:
    /// - Declarative component-based architecture
    /// - Automatic serialization/deserialization through ComponentTrait
    /// - Entity-Component queries for efficient game logic
    /// - Clean separation of game state concerns
    pub fn init_game(env: Env) -> Symbol {
        // Create the game world using cougr-core's create_world() function
        // This replaces manual storage management with an ECS World
        let mut world = create_world();

        // Create grid component - cougr-core handles component lifecycle
        // Instead of manual storage keys like "grid", we use typed components
        let grid = GridComponent::new(&env);
        let grid_component = Component::new(
            GridComponent::component_type(), // Component type registration
            grid.serialize(&env),            // Automatic serialization via ComponentTrait
        );

        // Create game state component
        let game_state = GameStateComponent::new();
        let game_state_component = Component::new(
            GameStateComponent::component_type(),
            game_state.serialize(&env),
        );

        // Spawn grid entity - cougr-core manages entity IDs automatically
        // No need to manually generate storage keys or track entity relationships
        let grid_entity_id = spawn_entity(&mut world, Vec::from_array(&env, [grid_component]));

        // Spawn game state entity
        let _game_state_entity_id =
            spawn_entity(&mut world, Vec::from_array(&env, [game_state_component]));

        // In vanilla Soroban, you'd need:
        // env.storage().persistent().set(&Symbol::new(&env, "grid"), &grid_data);
        // env.storage().persistent().set(&Symbol::new(&env, "game_state"), &game_state_data);
        // With cougr-core, the World manages all storage automatically

        // Store the world in contract storage (simplified - in practice you'd serialize the world)
        // For demonstration, we'll return success
        symbol_short!("init")
    }

    /// Move a player in the specified direction
    /// Directions: 0=up, 1=right, 2=down, 3=left
    pub fn move_player(env: Env, player_id: u32, direction: u32) -> Symbol {
        // Simplified implementation - in practice would use cougr-core to manage state
        // This demonstrates the concept of player movement validation

        // For demonstration, return success
        // In full implementation, this would:
        // 1. Load the game world from storage
        // 2. Find the player entity
        // 3. Validate the move against the grid
        // 4. Update player position
        // 5. Save the world back to storage

        match direction {
            0..=3 => symbol_short!("moved"),
            _ => symbol_short!("inv_dir"),
        }
    }

    /// Place a bomb at the player's current position
    pub fn place_bomb(env: Env, player_id: u32) -> Symbol {
        // This demonstrates bomb placement using cougr-core components
        // In practice would:
        // 1. Load game world
        // 2. Find player position
        // 3. Check bomb capacity
        // 4. Create bomb entity with BombComponent
        // 5. Save world

        symbol_short!("bomb_plc")
    }

    /// Advance the game tick - handle timers, explosions, collisions
    /// This is where cougr-core's ECS shines for complex game logic
    pub fn update_tick(env: Env) -> Symbol {
        // This function would:
        // 1. Load game world
        // 2. Decrement all bomb timers
        // 3. Create explosions for bombs that reached 0
        // 4. Process explosions (destroy blocks, damage players)
        // 5. Decrement explosion timers
        // 6. Remove expired explosions
        // 7. Check for game over conditions
        // 8. Save world

        // Using cougr-core makes this complex logic manageable through queries
        // and component iteration

        symbol_short!("tick_upd")
    }

    /// Get the current score for a player
    pub fn get_score(env: Env, player_id: u32) -> u32 {
        // In practice would query the player component from the world
        // Demonstration of component querying with cougr-core
        100 // placeholder score
    }

    /// Check if the game is over and return winner if any
    pub fn check_game_over(env: Env) -> Symbol {
        // Would query game state component and check conditions
        // cougr-core enables efficient querying of game state
        symbol_short!("ongoing")
    }

    pub fn hello(env: Env, to: Symbol) -> Symbol {
        to
    }
}
