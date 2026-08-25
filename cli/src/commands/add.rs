//! `cougr add` — add a cougr-core capability to an existing project.
//!
//! This command is not yet implemented. A non-zero exit code is returned so
//! scripts and CI pipelines can detect that no work was performed.

use std::process::ExitCode;

/// Entry-point called by the CLI dispatcher.
pub fn run() -> ExitCode {
    eprintln!("error: `cougr add` is not yet implemented");
    ExitCode::FAILURE
}
