//! Lightweight world assertions for tests (no `std`).

use crate::simple_world::SimpleWorld;

/// Assert simple world metrics without pulling in the `debug` feature.
pub struct SnapshotAssert;

impl SnapshotAssert {
    pub fn entity_count(world: &SimpleWorld) -> u32 {
        world.next_entity_id().saturating_sub(1)
    }

    pub fn assert_entity_count(world: &SimpleWorld, expected: u32) {
        assert_eq!(Self::entity_count(world), expected, "entity count mismatch");
    }

    pub fn assert_version_increased(before: &SimpleWorld, after: &SimpleWorld) {
        assert!(
            after.version() > before.version(),
            "expected world version to increase"
        );
    }

    pub fn assert_version_unchanged(before: &SimpleWorld, after: &SimpleWorld) {
        assert_eq!(before.version(), after.version(), "world version changed");
    }
}

#[cfg(feature = "debug")]
impl SnapshotAssert {
    /// Diff two worlds using the `debug` snapshot helpers.
    pub fn diff_entity_delta(
        env: &soroban_sdk::Env,
        before: &SimpleWorld,
        after: &SimpleWorld,
    ) -> u32 {
        let before_snap = crate::debug::take_snapshot(before, env);
        let after_snap = crate::debug::take_snapshot(after, env);
        let diff = crate::debug::diff_snapshots(&before_snap, &after_snap, env);
        (diff.added_entities.len() as u32)
            .saturating_add(diff.removed_entities.len() as u32)
            .saturating_add(diff.added_components.len() as u32)
            .saturating_add(diff.removed_components.len() as u32)
            .saturating_add(diff.modified_components.len() as u32)
    }
}
