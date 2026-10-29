use crate::types::{Direction, GhostMode, MAZE_WIDTH, PACMAN_ENTITY_ID};
use cougr_core::component::Position as CorePosition;
use cougr_core::event::CollisionEvent;
use soroban_sdk::{contracttype, symbol_short};

/// Position in the 2D maze grid
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn to_index(&self) -> u32 {
        (self.y as u32) * MAZE_WIDTH + (self.x as u32)
    }

    pub fn from_index(index: u32) -> Self {
        Self {
            x: (index % MAZE_WIDTH) as i32,
            y: (index / MAZE_WIDTH) as i32,
        }
    }

    pub fn to_core_position(&self) -> CorePosition {
        CorePosition::new(self.x, self.y)
    }

    pub fn from_core_position(core_pos: &CorePosition) -> Self {
        Self {
            x: core_pos.x,
            y: core_pos.y,
        }
    }
}

/// Ghost entity with position and behavior state
#[contracttype]
#[derive(Clone, Debug)]
pub struct Ghost {
    pub entity_id: u64,
    pub position: Position,
    pub direction: Direction,
    pub mode: GhostMode,
    pub frightened_timer: u32,
    pub start_position: Position,
}

impl Ghost {
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

    pub fn respawn(&mut self) {
        self.position = self.start_position;
        self.mode = GhostMode::Chase;
        self.frightened_timer = 0;
    }

    pub fn create_collision_event(&self) -> CollisionEvent {
        CollisionEvent::new(PACMAN_ENTITY_ID, self.entity_id, symbol_short!("ghost"))
    }
}
