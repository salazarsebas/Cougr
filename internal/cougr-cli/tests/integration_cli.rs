//! Integration tests for `cousr_cli::run` end-to-end argument parsing.

use cougr_cli::run;

#[test]
fn missing_subcommand_produces_usage_error() {
    // clap requires a subcommand -- invoking `cougr` with no args or only the
    // program name produces a Usage error rather than running anything.
    let err = run(["cougr"]).unwrap_err();
    match err {
        cougr_cli::CliError::Usage(_) => {}
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn unknown_subcommand_produces_usage_error() {
    let err = run(["cougr", "wiggle"]).unwrap_err();
    match err {
        cougr_cli::CliError::Usage(_) => {}
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn doctor_help_flag_does_not_fail() {
    let result = run(["cougr", "doctor", "--help"]);
    // `--help` is intercepted by clap and prints to stdout. The driver
    // returns `ExitCode::SUCCESS`-like Ok(()) regardless of the actual
    // doctor outcome because clap short-circuits before the doctor runs.
    assert!(result.is_ok(), "{:?}", result);
}
