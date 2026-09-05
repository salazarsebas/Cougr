//! Debug tooling for world inspection, metrics, and snapshots.
//!
//! Behind the `debug` feature flag to avoid bloating WASM builds.
//!
//! # Modules
//! - `introspect` - Entity and world inspection utilities
//! - `metrics` - Storage statistics and component counts
//! - `snapshot` - State snapshots and diffing

pub mod introspect;
pub mod metrics;
pub mod snapshot;

pub use introspect::{inspect_entity, inspect_world, list_entities, EntitySummary, WorldSummary};
pub use metrics::{collect_metrics, unique_component_types, StorageMetrics};
pub use snapshot::{diff_snapshots, take_snapshot, WorldDiff, WorldSnapshot};
