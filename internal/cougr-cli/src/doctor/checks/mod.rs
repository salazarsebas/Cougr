//! Toolchain check functions.
//!
//! Each submodule owns exactly one [`crate::doctor::CheckResult`] factory.
//! Tests for each check live alongside the check itself; tests that need a
//! fixture running the full doctor live in [`crate::doctor::tests`] and
//! `internal/cougr-cli/tests/`.

pub mod cargo;
pub mod rust_toolchain;
pub mod stellar_cli;
pub mod wasm32v1_target;
