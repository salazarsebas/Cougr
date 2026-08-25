//! `cougr doctor` — diagnose the local environment for Cougr development.
//!
//! This command is not yet implemented. A non-zero exit code is returned so
//! scripts and CI pipelines can detect that no work was performed.

use std::process::ExitCode;

/// Entry-point called by the CLI dispatcher.
pub fn run() -> ExitCode {
    eprintln!("error: `cougr doctor` is not yet implemented");
    ExitCode::FAILURE
}
