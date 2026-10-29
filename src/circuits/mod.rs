//! Pre-built ZK game circuit builders (`hidden_cards`, `fog_of_war`, etc.).
//!
//! Each builder returns a [`GameCircuitSpec`] with a frozen public-input layout and
//! the pipeline test verification key embedded from `internal/cougr-core-circuits`.
//! Production deploys replace the VK via [`GameCircuitSpec::with_verification_key`].
//!
//! Maturity: Experimental.

mod embedded;
mod fair_dice;
mod fog_of_war;
mod generated;
#[cfg(any(test, feature = "testutils"))]
mod generated_fixtures;
mod hidden_cards;
mod sealed_bid;
mod spec;

#[cfg(any(test, feature = "testutils"))]
pub mod test_fixtures {
    pub use super::embedded::{pipeline_proof, pipeline_public_bytes32, pipeline_public_inputs};
}

pub mod prelude {
    pub use super::{
        fair_dice, fog_of_war, hidden_cards, sealed_bid, CircuitId, CircuitParams, GameCircuitSpec,
        PublicInputLayout, PublicInputSlot,
    };
}

pub use fair_dice::fair_dice;
pub use fog_of_war::fog_of_war;
pub use hidden_cards::hidden_cards;
pub use sealed_bid::sealed_bid;
pub use spec::{CircuitId, CircuitParams, GameCircuitSpec, PublicInputLayout, PublicInputSlot};

/// Circuits layer version marker.
pub const MODULE_VERSION: &str = "0.1.0-circuits";
