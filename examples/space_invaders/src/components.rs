//! Transitional ECS component surface for Space Invaders.
//!
//! The historical implementation keeps the concrete component definitions in
//! `game_state.rs`. This module re-exports them under the standard arcade
//! example layout while preserving the existing public API.

pub use crate::game_state::{
    Bullet, Direction, EntityPosition, GameState, Health, Invader, InvaderType, Ship, Velocity,
};
