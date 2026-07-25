//! Library surface of `cougr-cli`.
//!
//! The binary [`crate::main`] is intentionally tiny; the bulk of the work lives
//! here so unit tests and future embedders can drive the same code path without
//! spawning a subprocess.
//!
//! # Public modules
//!
//! - [`doctor`] — toolchain diagnostics, the implementation behind `cougr doctor`
//!   and the non-fatal pre-flight invoked by [`new::scaffold`].
//! - [`new`] — minimal `cougr new <name>` scaffolding, intentionally small
//!   because the broader template story (per `docs/strategy/06-product-strategy.md`)
//!   lives in a separate epic.
//!
//! [`crate::main`]: mod@main

use std::ffi::OsString;

use clap::Parser;

pub mod doctor;
pub mod new;

mod cli;

/// Error variants produced by [`run`]. The binary converts each variant into an
/// exit code; tests treat them as ordinary `Result` values.
///
/// ## Contract for embedders
///
/// Every variant's existence has a corresponding recipe in `main.rs`, so a
/// caller wiring its own exit-code map can rely on the same convention:
/// `DoctorFailed` -> 1, `Scaffold` -> 1, `Usage` -> 2, `DoctorWarningsPrinted`
/// -> wrapped `code` (always 0 in this build; encoded explicitly to keep the
/// variant non-exhaustive-proof-friendly for future per-warning exit codes).
///
/// `DoctorWarningsPrinted` is intentionally an `Err` even though the
/// scaffold itself succeeded: it is the binary's signal that some side-
/// effecting diagnostic was emitted and lets the caller route its own
/// "warnings happened" branch in lockstep with the binary.
#[derive(Debug, PartialEq, Eq)]
pub enum CliError {
    /// `cougr doctor` was invoked standalone and at least one check failed.
    DoctorFailed,
    /// `cougr new` ran doctor in non-fatal mode, printed warnings, and
    /// still scaffolded. The wrapped `code` is the exit code that callers
    /// asked for (typically 0 -- the scaffold succeeded, only the
    /// pre-flight was unhealthy).
    DoctorWarningsPrinted(u8),
    /// `cougr new` failed to scaffold the project for some non-doctor reason
    /// (e.g. invalid name, IO error).
    Scaffold(String),
    /// The user supplied a subcommand with an argument clap refused to
    /// parse, or no subcommand at all. Maps to exit 2.
    Usage(String),
}

/// Top-level driver. Parses arguments with clap and dispatches to the matching
/// subcommand. The dispatch signature matches the binary's expected surface:
/// the binary passes `std::env::args_os()`; tests pass synthetic argv vectors.
pub fn run<I, T>(args: I) -> Result<(), CliError>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let argv: Vec<OsString> = args.into_iter().map(Into::into).collect();
    let parsed = cli::Cli::try_parse_from(&argv)
        .map_err(|e| CliError::Usage(e.to_string()))?;

    match parsed.command {
        cli::Command::Doctor(doctor_args) => cli::dispatch_doctor(doctor_args),
        cli::Command::New(new_args) => cli::dispatch_new(new_args),
    }
}
