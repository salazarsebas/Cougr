//! Tower Defense game components using cougr-core's ComponentTrait
//!
//! This module defines all the ECS components needed for a tower defense game
//! on the Stellar blockchain via Soroban.

pub use cougr_core::component::{ComponentStorage, ComponentTrait};
use soroban_sdk::{contracttype, symbol_short, Bytes, Env, Symbol};

// ============================================================================
// Map constants
// ============================================================================

/// Map size (10x10 grid)
pub const MAP_WIDTH: u32 = 10;
pub const MAP_HEIGHT: u32 = 10;

/// Path length (number of waypoints)
pub const PATH_LENGTH: usize = 8;

/// Predefined path from spawn to base
pub const PATH: [(u32, u32); PATH_LENGTH] = [
    (0, 5), // Spawn point
    (2, 5),
    (2, 2),
    (5, 2),
    (5, 7),
    (8, 7),
    (8, 5),
    (9, 5), // Base
];

// ============================================================================
// Game Status
// ============================================================================

/// Game status enum
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum GameStatus {
    Active = 0,
    Won = 1,
    Lost = 2,
}

impl GameStatus {
    pub fn to_u8(self) -> u8 {
        match self {
            GameStatus::Active => 0,
            GameStatus::Won => 1,
            GameStatus::Lost => 2,
        }
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(GameStatus::Active),
            1 => Some(GameStatus::Won),
            2 => Some(GameStatus::Lost),
            _ => None,
        }
    }
}

// ============================================================================
// Tower Kind
// ============================================================================

/// Tower types available in the game
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum TowerKind {
    Basic = 0,  // Low damage, medium range, fast cooldown
    Sniper = 1, // High damage, long range, slow cooldown
    Splash = 2, // Medium damage, short range, damages area
}

impl TowerKind {
    pub fn to_u8(self) -> u8 {
        match self {
            TowerKind::Basic => 0,
            TowerKind::Sniper => 1,
            TowerKind::Splash => 2,
        }
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(TowerKind::Basic),
            1 => Some(TowerKind::Sniper),
            2 => Some(TowerKind::Splash),
            _ => None,
        }
    }

    /// Get tower stats based on kind
    pub fn stats(self) -> (u32, u32, u32) {
        // Returns (range, damage, cooldown)
        match self {
            TowerKind::Basic => (2, 10, 1),
            TowerKind::Sniper => (4, 25, 3),
            TowerKind::Splash => (1, 15, 2),
        }
    }
}

// ============================================================================
// Position Component
// ============================================================================

/// Position component for entities on the grid
#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Position {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    /// Check if position is within map bounds
    pub fn is_valid(&self) -> bool {
        self.x < MAP_WIDTH && self.y < MAP_HEIGHT
    }

    /// Calculate Manhattan distance to another position
    pub fn distance_to(&self, other: &Position) -> u32 {
        let dx = self.x.abs_diff(other.x);
        let dy = self.y.abs_diff(other.y);
        dx + dy
    }
}

impl ComponentTrait for Position {
    fn component_type() -> Symbol {
        symbol_short!("position")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut data = [0u8; 8];
        data[0..4].copy_from_slice(&self.x.to_be_bytes());
        data[4..8].copy_from_slice(&self.y.to_be_bytes());
        Bytes::from_array(env, &data)
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }
        let mut buf = [0u8; 8];
        data.copy_into_slice(&mut buf);
        let x = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let y = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        Some(Self { x, y })
    }

    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table
    }
}

// ============================================================================
// Enemy Component
// ============================================================================

/// Enemy component - represents enemies moving along a path
#[derive(Clone, Debug, PartialEq)]
pub struct EnemyComponent {
    pub hp: u32,
    pub max_hp: u32,
    pub speed: u32,      // Steps per tick
    pub path_index: u32, // Current position on the path
}

impl EnemyComponent {
    pub fn new(hp: u32, speed: u32) -> Self {
        Self {
            hp,
            max_hp: hp,
            speed,
            path_index: 0,
        }
    }

    /// Create a basic enemy for a given wave
    pub fn for_wave(wave: u32) -> Self {
        let base_hp = 50 + (wave * 10);
        let speed = 1;
        Self::new(base_hp, speed)
    }

    /// Check if the enemy is alive
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    /// Check if the enemy has reached the base
    pub fn reached_base(&self) -> bool {
        self.path_index >= (PATH_LENGTH as u32 - 1)
    }

    /// Take damage and return remaining HP
    pub fn take_damage(&mut self, damage: u32) -> u32 {
        if damage >= self.hp {
            self.hp = 0;
        } else {
            self.hp -= damage;
        }
        self.hp
    }
}

impl ComponentTrait for EnemyComponent {
    fn component_type() -> Symbol {
        symbol_short!("enemy")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut data = [0u8; 16];
        data[0..4].copy_from_slice(&self.hp.to_be_bytes());
        data[4..8].copy_from_slice(&self.max_hp.to_be_bytes());
        data[8..12].copy_from_slice(&self.speed.to_be_bytes());
        data[12..16].copy_from_slice(&self.path_index.to_be_bytes());
        Bytes::from_array(env, &data)
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 16 {
            return None;
        }
        let mut buf = [0u8; 16];
        data.copy_into_slice(&mut buf);
        let hp = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let max_hp = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let speed = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
        let path_index = u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]);
        Some(Self {
            hp,
            max_hp,
            speed,
            path_index,
        })
    }

    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table
    }
}

// ============================================================================
// Tower Component
// ============================================================================

/// Tower component - represents static defenders
#[derive(Clone, Debug, PartialEq)]
pub struct TowerComponent {
    pub kind: TowerKind,
    pub range: u32,
    pub damage: u32,
    pub cooldown: u32,
    pub current_cooldown: u32, // Ticks until can fire again
}

impl TowerComponent {
    pub fn new(kind: TowerKind) -> Self {
        let (range, damage, cooldown) = kind.stats();
        Self {
            kind,
            range,
            damage,
            cooldown,
            current_cooldown: 0,
        }
    }

    /// Check if tower can attack this tick
    pub fn can_attack(&self) -> bool {
        self.current_cooldown == 0
    }

    /// Reset cooldown after attack
    pub fn reset_cooldown(&mut self) {
        self.current_cooldown = self.cooldown;
    }

    /// Tick down cooldown
    pub fn tick_cooldown(&mut self) {
        if self.current_cooldown > 0 {
            self.current_cooldown -= 1;
        }
    }
}

impl ComponentTrait for TowerComponent {
    fn component_type() -> Symbol {
        symbol_short!("tower")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut data = [0u8; 17];
        data[0] = self.kind.to_u8();
        data[1..5].copy_from_slice(&self.range.to_be_bytes());
        data[5..9].copy_from_slice(&self.damage.to_be_bytes());
        data[9..13].copy_from_slice(&self.cooldown.to_be_bytes());
        data[13..17].copy_from_slice(&self.current_cooldown.to_be_bytes());
        Bytes::from_array(env, &data)
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 17 {
            return None;
        }
        let mut buf = [0u8; 17];
        data.copy_into_slice(&mut buf);
        let kind = TowerKind::from_u8(buf[0])?;
        let range = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
        let damage = u32::from_be_bytes([buf[5], buf[6], buf[7], buf[8]]);
        let cooldown = u32::from_be_bytes([buf[9], buf[10], buf[11], buf[12]]);
        let current_cooldown = u32::from_be_bytes([buf[13], buf[14], buf[15], buf[16]]);
        Some(Self {
            kind,
            range,
            damage,
            cooldown,
            current_cooldown,
        })
    }

    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table
    }
}

// ============================================================================
// Wave Component
// ============================================================================

/// Wave component - tracks encounter progression
#[derive(Clone, Debug, PartialEq)]
pub struct WaveComponent {
    pub current_wave: u32,
    pub total_waves: u32,
    pub remaining_spawns: u32,
    pub spawn_interval: u32,    // Ticks between spawns
    pub ticks_until_spawn: u32, // Countdown to next spawn
}

impl WaveComponent {
    pub fn new(total_waves: u32, enemies_per_wave: u32) -> Self {
        Self {
            current_wave: 1,
            total_waves,
            remaining_spawns: enemies_per_wave,
            spawn_interval: 3,
            ticks_until_spawn: 0,
        }
    }

    /// Check if all waves are complete
    pub fn all_waves_complete(&self) -> bool {
        self.current_wave > self.total_waves
    }

    /// Check if current wave has more enemies to spawn
    pub fn has_spawns_remaining(&self) -> bool {
        self.remaining_spawns > 0
    }

    /// Check if should spawn this tick
    pub fn should_spawn(&self) -> bool {
        self.has_spawns_remaining() && self.ticks_until_spawn == 0
    }

    /// Record a spawn
    pub fn record_spawn(&mut self) {
        if self.remaining_spawns > 0 {
            self.remaining_spawns -= 1;
            self.ticks_until_spawn = self.spawn_interval;
        }
    }

    /// Tick spawn timer
    pub fn tick(&mut self) {
        if self.ticks_until_spawn > 0 {
            self.ticks_until_spawn -= 1;
        }
    }

    /// Advance to next wave
    pub fn next_wave(&mut self, enemies_per_wave: u32) {
        self.current_wave += 1;
        self.remaining_spawns = enemies_per_wave;
        self.ticks_until_spawn = 0;
    }
}

impl ComponentTrait for WaveComponent {
    fn component_type() -> Symbol {
        symbol_short!("wave")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut data = [0u8; 20];
        data[0..4].copy_from_slice(&self.current_wave.to_be_bytes());
        data[4..8].copy_from_slice(&self.total_waves.to_be_bytes());
        data[8..12].copy_from_slice(&self.remaining_spawns.to_be_bytes());
        data[12..16].copy_from_slice(&self.spawn_interval.to_be_bytes());
        data[16..20].copy_from_slice(&self.ticks_until_spawn.to_be_bytes());
        Bytes::from_array(env, &data)
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 20 {
            return None;
        }
        let mut buf = [0u8; 20];
        data.copy_into_slice(&mut buf);
        let current_wave = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let total_waves = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let remaining_spawns = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
        let spawn_interval = u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]);
        let ticks_until_spawn = u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]);
        Some(Self {
            current_wave,
            total_waves,
            remaining_spawns,
            spawn_interval,
            ticks_until_spawn,
        })
    }

    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table
    }
}

// ============================================================================
// Base Component
// ============================================================================

/// Base component - tracks survival condition
#[derive(Clone, Debug, PartialEq)]
pub struct BaseComponent {
    pub health: u32,
    pub max_health: u32,
}

impl BaseComponent {
    pub fn new(health: u32) -> Self {
        Self {
            health,
            max_health: health,
        }
    }

    /// Check if base is destroyed
    pub fn is_destroyed(&self) -> bool {
        self.health == 0
    }

    /// Take damage from enemy reaching base
    pub fn take_damage(&mut self, damage: u32) {
        if damage >= self.health {
            self.health = 0;
        } else {
            self.health -= damage;
        }
    }
}

impl ComponentTrait for BaseComponent {
    fn component_type() -> Symbol {
        symbol_short!("base")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut data = [0u8; 8];
        data[0..4].copy_from_slice(&self.health.to_be_bytes());
        data[4..8].copy_from_slice(&self.max_health.to_be_bytes());
        Bytes::from_array(env, &data)
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }
        let mut buf = [0u8; 8];
        data.copy_into_slice(&mut buf);
        let health = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let max_health = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        Some(Self { health, max_health })
    }

    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table
    }
}

// ============================================================================
// Game Status Component
// ============================================================================

/// Game status component - tracks active, won, or lost state
#[derive(Clone, Debug, PartialEq)]
pub struct GameStatusComponent {
    pub status: GameStatus,
    pub tick_count: u32,
    pub enemies_killed: u32,
}

impl GameStatusComponent {
    pub fn new() -> Self {
        Self {
            status: GameStatus::Active,
            tick_count: 0,
            enemies_killed: 0,
        }
    }

    pub fn is_active(&self) -> bool {
        self.status == GameStatus::Active
    }

    pub fn set_won(&mut self) {
        self.status = GameStatus::Won;
    }

    pub fn set_lost(&mut self) {
        self.status = GameStatus::Lost;
    }

    pub fn increment_tick(&mut self) {
        self.tick_count += 1;
    }

    pub fn record_kill(&mut self) {
        self.enemies_killed += 1;
    }
}

impl Default for GameStatusComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentTrait for GameStatusComponent {
    fn component_type() -> Symbol {
        symbol_short!("status")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut data = [0u8; 9];
        data[0] = self.status.to_u8();
        data[1..5].copy_from_slice(&self.tick_count.to_be_bytes());
        data[5..9].copy_from_slice(&self.enemies_killed.to_be_bytes());
        Bytes::from_array(env, &data)
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 9 {
            return None;
        }
        let mut buf = [0u8; 9];
        data.copy_into_slice(&mut buf);
        let status = GameStatus::from_u8(buf[0])?;
        let tick_count = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
        let enemies_killed = u32::from_be_bytes([buf[5], buf[6], buf[7], buf[8]]);
        Some(Self {
            status,
            tick_count,
            enemies_killed,
        })
    }

    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table
    }
}
