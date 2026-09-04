//! ECS components and storage keys for {{crate_name}}.
//!
//! The gameplay state a session protects is deliberately tiny - a tap counter - //! so the interesting part stays visible: who is allowed to act, for how long,
//! and how many times.

use cougr_core::impl_component;
use soroban_sdk::{contracttype, Address, Env, Symbol};

/// How many times a player has acted through their session.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Score {
    pub taps: u32,
}

impl_component!(Score, "score", Sparse, { taps: u32 });

/// Instance-storage key mapping a player to their ECS entity.
pub fn player_key(env: &Env, player: &Address) -> (Symbol, Address) {
    (Symbol::new(env, "player"), player.clone())
}
