//! Record world checkpoints and fork mid-game for variant replay.

use crate::game::SorobanGame;
use crate::simple_world::SimpleWorld;
use alloc::vec::Vec;

use super::fixture::WorldFixture;
use super::harness::GameHarness;
use super::scenario::TurnIndex;

/// Frozen world state at a specific turn.
#[derive(Clone, Debug)]
pub struct ReplayCheckpoint {
    pub turn: u32,
    pub world: SimpleWorld,
}

/// Append-only log of world snapshots for replay and variant testing.
pub struct ReplayLog {
    checkpoints: Vec<ReplayCheckpoint>,
}

impl ReplayLog {
    pub fn new() -> Self {
        Self {
            checkpoints: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.checkpoints.len()
    }

    pub fn is_empty(&self) -> bool {
        self.checkpoints.is_empty()
    }

    pub fn checkpoints(&self) -> &[ReplayCheckpoint] {
        &self.checkpoints
    }

    /// Snapshot the contract world after a turn and append it to the log.
    pub fn record<G: SorobanGame>(&mut self, turn: TurnIndex, harness: &GameHarness) {
        let fixture = WorldFixture::read_from_contract::<G>(harness);
        self.checkpoints.push(ReplayCheckpoint {
            turn: turn.0,
            world: fixture.world().clone(),
        });
    }

    pub fn checkpoint_at(&self, turn: u32) -> Option<&ReplayCheckpoint> {
        self.checkpoints.iter().find(|entry| entry.turn == turn)
    }

    pub fn world_at(&self, turn: u32) -> Option<&SimpleWorld> {
        self.checkpoint_at(turn).map(|entry| &entry.world)
    }

    /// Restore contract storage to the world captured at `turn`.
    pub fn fork_from<G: SorobanGame>(&self, harness: &GameHarness, turn: u32) -> WorldFixture {
        let world = self
            .world_at(turn)
            .cloned()
            .unwrap_or_else(|| panic!("replay log has no checkpoint at turn {}", turn));
        let fixture = WorldFixture::with_world(world);
        fixture.inject::<G>(harness);
        fixture
    }

    /// Assert two checkpoints share the same entity count and world version.
    pub fn assert_same_at(&self, turn_a: u32, turn_b: u32) {
        let a = self
            .world_at(turn_a)
            .unwrap_or_else(|| panic!("missing checkpoint at turn {}", turn_a));
        let b = self
            .world_at(turn_b)
            .unwrap_or_else(|| panic!("missing checkpoint at turn {}", turn_b));
        assert_eq!(entity_count(a), entity_count(b));
        assert_eq!(a.version(), b.version());
    }

    /// Assert entity count or version changed between two turns.
    pub fn assert_differs_at(&self, turn_a: u32, turn_b: u32) {
        let a = self
            .world_at(turn_a)
            .unwrap_or_else(|| panic!("missing checkpoint at turn {}", turn_a));
        let b = self
            .world_at(turn_b)
            .unwrap_or_else(|| panic!("missing checkpoint at turn {}", turn_b));
        assert!(
            entity_count(a) != entity_count(b) || a.version() != b.version(),
            "expected world to differ between turns {} and {}",
            turn_a,
            turn_b
        );
    }
}

fn entity_count(world: &SimpleWorld) -> u32 {
    world.next_entity_id().saturating_sub(1)
}

impl Default for ReplayLog {
    fn default() -> Self {
        Self::new()
    }
}
