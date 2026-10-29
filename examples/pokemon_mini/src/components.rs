//! Pokémon Mini game components using cougr-core's ComponentTrait
//!
//! This module demonstrates how to create custom game components for a
//! Pokémon-style mini game on the Stellar blockchain via Soroban.

pub use cougr_core::component::{ComponentStorage, ComponentTrait};
use soroban_sdk::{contracttype, symbol_short, Bytes, Env, Symbol};

// ============================================================================
// Map constants
// ============================================================================

/// Map size (8x8 grid as specified in the issue)
pub const MAP_WIDTH: i32 = 8;
pub const MAP_HEIGHT: i32 = 8;

/// Encounter trigger constant (deterministic formula: (x + y + move_count) % K == 0)
pub const ENCOUNTER_MODULO: u32 = 5;

// ============================================================================
// Tile Types
// ============================================================================

/// Tile types for the game map
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum TileType {
    Grass = 0,
    Wall = 1,
    Water = 2,
    TallGrass = 3,
    Spawn = 4,
}

impl TileType {
    pub fn to_u8(self) -> u8 {
        match self {
            TileType::Grass => 0,
            TileType::Wall => 1,
            TileType::Water => 2,
            TileType::TallGrass => 3,
            TileType::Spawn => 4,
        }
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(TileType::Grass),
            1 => Some(TileType::Wall),
            2 => Some(TileType::Water),
            3 => Some(TileType::TallGrass),
            4 => Some(TileType::Spawn),
            _ => None,
        }
    }

    /// Check if a tile blocks movement
    pub fn is_blocked(&self) -> bool {
        matches!(self, TileType::Wall | TileType::Water)
    }

    /// Check if a tile can trigger encounters
    pub fn can_trigger_encounter(&self) -> bool {
        matches!(self, TileType::TallGrass)
    }
}

// ============================================================================
// Direction
// ============================================================================

/// Direction enum for player movement
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

impl Direction {
    pub fn to_u8(self) -> u8 {
        match self {
            Direction::Up => 0,
            Direction::Down => 1,
            Direction::Left => 2,
            Direction::Right => 3,
        }
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Direction::Up),
            1 => Some(Direction::Down),
            2 => Some(Direction::Left),
            3 => Some(Direction::Right),
            _ => None,
        }
    }

    /// Get the delta movement for this direction
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

// ============================================================================
// Position Component
// ============================================================================

/// Position component - represents a point on the grid
#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Check if position is within map bounds
    pub fn is_valid(&self) -> bool {
        self.x >= 0 && self.x < MAP_WIDTH && self.y >= 0 && self.y < MAP_HEIGHT
    }

    /// Apply a direction to get a new position
    pub fn apply_direction(&self, direction: Direction) -> Self {
        let (dx, dy) = direction.delta();
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

impl ComponentTrait for Position {
    fn component_type() -> Symbol {
        symbol_short!("position")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        let x_bytes = Bytes::from_array(env, &self.x.to_be_bytes());
        let y_bytes = Bytes::from_array(env, &self.y.to_be_bytes());
        bytes.append(&x_bytes);
        bytes.append(&y_bytes);
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 8 {
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
        Some(Self { x, y })
    }

    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table
    }
}

// ============================================================================
// Creature Component
// ============================================================================

/// Creature - represents a Pokémon-like creature with basic stats
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Creature {
    pub species_id: u32,
    pub level: u32,
    pub hp: u32,
    pub max_hp: u32,
    pub atk: u32,
    pub def: u32,
}

impl Creature {
    /// Create a new creature with base stats
    pub fn new(species_id: u32, level: u32, max_hp: u32, atk: u32, def: u32) -> Self {
        Self {
            species_id,
            level,
            hp: max_hp,
            max_hp,
            atk,
            def,
        }
    }

    /// Create a starter creature
    pub fn starter() -> Self {
        Self::new(1, 5, 20, 8, 5)
    }

    /// Create a wild creature based on player's move count (deterministic)
    pub fn wild_from_seed(seed: u32) -> Self {
        // Deterministic creature generation
        let species_id = (seed % 3) + 1; // Species 1-3
        let level = (seed % 5) + 3; // Level 3-7
        let max_hp = 10 + (level * 2);
        let atk = 4 + level;
        let def = 3 + (level / 2);
        Self::new(species_id, level, max_hp, atk, def)
    }

    /// Take damage and return true if still alive
    pub fn take_damage(&mut self, damage: u32) -> bool {
        if damage >= self.hp {
            self.hp = 0;
            false
        } else {
            self.hp -= damage;
            true
        }
    }

    /// Check if creature is fainted
    pub fn is_fainted(&self) -> bool {
        self.hp == 0
    }

    /// Heal the creature fully
    pub fn heal_full(&mut self) {
        self.hp = self.max_hp;
    }
}

impl ComponentTrait for Creature {
    fn component_type() -> Symbol {
        symbol_short!("creature")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.species_id.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.level.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.hp.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.max_hp.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.atk.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.def.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 24 {
            return None;
        }
        let species_id = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let level = u32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);
        let hp = u32::from_be_bytes([
            data.get(8).unwrap(),
            data.get(9).unwrap(),
            data.get(10).unwrap(),
            data.get(11).unwrap(),
        ]);
        let max_hp = u32::from_be_bytes([
            data.get(12).unwrap(),
            data.get(13).unwrap(),
            data.get(14).unwrap(),
            data.get(15).unwrap(),
        ]);
        let atk = u32::from_be_bytes([
            data.get(16).unwrap(),
            data.get(17).unwrap(),
            data.get(18).unwrap(),
            data.get(19).unwrap(),
        ]);
        let def = u32::from_be_bytes([
            data.get(20).unwrap(),
            data.get(21).unwrap(),
            data.get(22).unwrap(),
            data.get(23).unwrap(),
        ]);
        Some(Self {
            species_id,
            level,
            hp,
            max_hp,
            atk,
            def,
        })
    }

    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table
    }
}

// ============================================================================
// Battle Action
// ============================================================================

/// Actions available during battle
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum BattleAction {
    Attack = 0,
    Defend = 1,
    Run = 2,
}

impl BattleAction {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(BattleAction::Attack),
            1 => Some(BattleAction::Defend),
            2 => Some(BattleAction::Run),
            _ => None,
        }
    }
}

// ============================================================================
// Battle Phase
// ============================================================================

/// Battle phase states
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum BattlePhase {
    WaitingPlayerAction = 0,
    Resolved = 1,
    Finished = 2,
}

// ============================================================================
// Battle Result
// ============================================================================

/// Battle result outcomes
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum BattleResult {
    None = 0,
    Win = 1,
    Lose = 2,
    Escaped = 3,
}

// ============================================================================
// Battle State
// ============================================================================

/// Complete battle state
#[contracttype]
#[derive(Clone, Debug)]
pub struct BattleState {
    pub battle_id: u32,
    pub player_creature: Creature,
    pub enemy_creature: Creature,
    pub turn: u32,
    pub phase: BattlePhase,
    pub result: BattleResult,
    pub player_defending: bool,
}

impl BattleState {
    /// Create a new battle
    pub fn new(battle_id: u32, player_creature: Creature, enemy_creature: Creature) -> Self {
        Self {
            battle_id,
            player_creature,
            enemy_creature,
            turn: 1,
            phase: BattlePhase::WaitingPlayerAction,
            result: BattleResult::None,
            player_defending: false,
        }
    }

    /// Calculate damage using deterministic formula
    pub fn calculate_damage(attacker_atk: u32, defender_def: u32) -> u32 {
        if attacker_atk > defender_def {
            attacker_atk - defender_def
        } else {
            1 // Minimum damage
        }
    }

    /// Check if battle is over
    pub fn is_finished(&self) -> bool {
        self.phase == BattlePhase::Finished
    }
}

// ============================================================================
// Player Marker Component
// ============================================================================

/// Direction component wrapper for storage
#[derive(Clone, Debug)]
pub struct DirectionComponent {
    pub direction: Direction,
}

impl DirectionComponent {
    pub fn new(direction: Direction) -> Self {
        Self { direction }
    }
}

impl ComponentTrait for DirectionComponent {
    fn component_type() -> Symbol {
        symbol_short!("facing")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        Bytes::from_array(env, &[self.direction.to_u8()])
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 1 {
            return None;
        }
        let direction = Direction::from_u8(data.get(0).unwrap())?;
        Some(Self { direction })
    }

    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_serialization() {
        let env = Env::default();
        let pos = Position::new(5, 7);

        let serialized = pos.serialize(&env);
        let deserialized = Position::deserialize(&env, &serialized).unwrap();

        assert_eq!(pos.x, deserialized.x);
        assert_eq!(pos.y, deserialized.y);
    }

    #[test]
    fn test_position_valid() {
        assert!(Position::new(0, 0).is_valid());
        assert!(Position::new(7, 7).is_valid());
        assert!(!Position::new(-1, 0).is_valid());
        assert!(!Position::new(8, 0).is_valid());
        assert!(!Position::new(0, 8).is_valid());
    }

    #[test]
    fn test_position_apply_direction() {
        let pos = Position::new(4, 4);
        assert_eq!(pos.apply_direction(Direction::Up), Position::new(4, 3));
        assert_eq!(pos.apply_direction(Direction::Down), Position::new(4, 5));
        assert_eq!(pos.apply_direction(Direction::Left), Position::new(3, 4));
        assert_eq!(pos.apply_direction(Direction::Right), Position::new(5, 4));
    }

    #[test]
    fn test_tile_type_blocked() {
        assert!(!TileType::Grass.is_blocked());
        assert!(TileType::Wall.is_blocked());
        assert!(TileType::Water.is_blocked());
        assert!(!TileType::TallGrass.is_blocked());
        assert!(!TileType::Spawn.is_blocked());
    }

    #[test]
    fn test_tile_type_encounter() {
        assert!(!TileType::Grass.can_trigger_encounter());
        assert!(!TileType::Wall.can_trigger_encounter());
        assert!(TileType::TallGrass.can_trigger_encounter());
    }

    #[test]
    fn test_creature_serialization() {
        let env = Env::default();
        let creature = Creature::new(1, 5, 20, 8, 5);

        let serialized = creature.serialize(&env);
        let deserialized = Creature::deserialize(&env, &serialized).unwrap();

        assert_eq!(creature.species_id, deserialized.species_id);
        assert_eq!(creature.level, deserialized.level);
        assert_eq!(creature.hp, deserialized.hp);
        assert_eq!(creature.max_hp, deserialized.max_hp);
        assert_eq!(creature.atk, deserialized.atk);
        assert_eq!(creature.def, deserialized.def);
    }

    #[test]
    fn test_creature_damage() {
        let mut creature = Creature::new(1, 5, 20, 8, 5);
        assert!(creature.take_damage(5));
        assert_eq!(creature.hp, 15);
        assert!(!creature.take_damage(20));
        assert_eq!(creature.hp, 0);
        assert!(creature.is_fainted());
    }

    #[test]
    fn test_battle_damage_calculation() {
        // Normal damage
        assert_eq!(BattleState::calculate_damage(10, 5), 5);
        // Low attack
        assert_eq!(BattleState::calculate_damage(3, 5), 1);
        // Equal stats
        assert_eq!(BattleState::calculate_damage(5, 5), 1);
    }

    #[test]
    fn test_direction_delta() {
        assert_eq!(Direction::Up.delta(), (0, -1));
        assert_eq!(Direction::Down.delta(), (0, 1));
        assert_eq!(Direction::Left.delta(), (-1, 0));
        assert_eq!(Direction::Right.delta(), (1, 0));
    }
}
