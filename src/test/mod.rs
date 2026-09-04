//! On-chain game testing sandbox (harness, scenarios, fixtures, replay).
//!
//! Available with the `testutils` feature. Uses `no_std` + `alloc` - not `std`.

mod fixture;
mod harness;
mod mock_session;
mod replay;
mod scenario;
mod snapshot;

pub use fixture::WorldFixture;
pub use harness::{GameHarness, PlayerSlot};
pub use mock_session::MockSession;
pub use replay::{ReplayCheckpoint, ReplayLog};
pub use scenario::{Scenario, TurnIndex};
pub use snapshot::SnapshotAssert;

/// Bootstrap marker - bumped when sandbox API changes.
pub const MODULE_VERSION: &str = "0.1.0-sandbox";
