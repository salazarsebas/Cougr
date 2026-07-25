//! `cougr` binary entry point.
//!
//! All real logic lives in [`cougr_cli::run`]. The binary is a thin adapter so the
//! crate is also exercisable as a library from tests and from future embedders
//! (e.g. a desktop front-end wrapping the doctor checks).

use std::process::ExitCode;

use cougr_cli::{run, CliError};

fn main() -> ExitCode {
    match run(std::env::args_os()) {
        Ok(()) => ExitCode::from(0),
        Err(CliError::DoctorFailed) => ExitCode::from(1),
        Err(CliError::DoctorWarningsPrinted(code)) => ExitCode::from(code),
        Err(CliError::Scaffold(message)) => {
            eprintln!("error: {message}");
            ExitCode::from(1)
        }
        Err(CliError::Usage(message)) => {
            eprintln!("error: {message}");
            ExitCode::from(2)
        }
    }
}
