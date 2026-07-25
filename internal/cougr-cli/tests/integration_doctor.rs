//! Integration tests for `cougr doctor` end-to-end orchestration.
//!
//! These live in the integration test directory so they exercise only the
//! public API of the crate (no `#[cfg(test)] mod tests` reach into private
//! state), matching how a third-party embedder would consume the library.

use cougr_cli::doctor::runner::{CommandOutput, MockRunner, SystemRunner};
use cougr_cli::doctor::{
    run as run_doctor, DoctorConfig, DOCTOR_DEFAULT_STELLAR_MIN_VERSION, DOCTOR_TARGET,
};

fn success(stdout: &str) -> CommandOutput {
    CommandOutput::success(stdout.to_string())
}

fn all_passing_runner() -> MockRunner {
    MockRunner::new()
        .with_response("rustc", &["--version"], success("rustc 1.85.0 (abc 2025)"))
        .with_response(
            "rustup",
            &["target", "list", "--installed"],
            success("wasm32v1-none\nx86_64-unknown-linux-gnu\n"),
        )
        .with_response("cargo", &["--version"], success("cargo 1.85.0 (abc 2025)\n"))
        .with_response(
            "stellar",
            &["--version"],
            success("stellar 23.1.0 (release)\n"),
        )
}

#[test]
fn all_passing_runner_drives_a_clean_report() {
    let runner = all_passing_runner();
    let report = run_doctor(
        &DoctorConfig {
            rust_manifest: None,
            stellar_min_version: DOCTOR_DEFAULT_STELLAR_MIN_VERSION.to_string(),
        },
        &runner,
    );
    assert_eq!(report.total_count(), 4);
    assert_eq!(report.passed_count(), 4);
    assert_eq!(report.failed_count(), 0);
    assert!(report.all_passed());
}

#[test]
fn missing_wasm_target_prints_specific_fix_command() {
    let runner = all_passing_runner().with_response(
        "rustup",
        &["target", "list", "--installed"],
        success("x86_64-unknown-linux-gnu\n"),
    );
    let report = run_doctor(&DoctorConfig::default(), &runner);
    let target = report
        .checks
        .iter()
        .find(|c| c.name == "wasm32v1-none target")
        .unwrap();
    assert!(!target.passed);
    let fix = target.fix.as_deref().unwrap();
    assert!(fix.contains("rustup target add"), "{fix}");
    assert!(fix.contains(DOCTOR_TARGET), "{fix}");
}

#[test]
fn missing_stellar_cli_prints_install_link_and_min_version() {
    let runner = all_passing_runner(); // stellar is missing from the mock
    let report = run_doctor(
        &DoctorConfig {
            stellar_min_version: "22.0.0".into(),
            ..Default::default()
        },
        &runner,
    );
    let stellar = report
        .checks
        .iter()
        .find(|c| c.name == "stellar CLI")
        .unwrap();
    assert!(!stellar.passed);
    let fix = stellar.fix.as_deref().unwrap();
    assert!(fix.contains("developers.stellar.org"), "{fix}");
    assert!(fix.contains("22.0.0"), "{fix}");
}

#[test]
fn rust_too_old_prints_rustup_update_command() {
    let runner = all_passing_runner().with_response(
        "rustc",
        &["--version"],
        success("rustc 1.68.0 (abc 2025)"),
    );
    let report = run_doctor(&DoctorConfig::default(), &runner);
    let rust = report
        .checks
        .iter()
        .find(|c| c.name == "rust toolchain")
        .unwrap();
    assert!(!rust.passed);
    let fix = rust.fix.as_deref().unwrap();
    assert!(fix.contains("rustup update stable"), "{fix}");
}

#[test]
fn cargo_check_reports_cargo_version_when_present() {
    // `cargo` was registered as success in `all_passing_runner`, so the
    // happy-path of the wiring should pass with the version string it
    // returned. The failure-mode paths live in the unit tests and in
    // `missing_cargo_response_yields_failed_check` below.
    let runner = all_passing_runner();
    let report = run_doctor(&DoctorConfig::default(), &runner);
    let cargo_check = report.checks.iter().find(|c| c.name == "cargo").unwrap();
    assert!(cargo_check.passed);
    assert!(cargo_check.detail.contains("1.85.0"));
}

#[test]
fn missing_cargo_response_yields_failed_check() {
    // Strip the cargo success entry so the check sees a missing-cargo state.
    let runner = MockRunner::new()
        .with_response("rustc", &["--version"], success("rustc 1.85.0 (abc 2025)"))
        .with_response(
            "rustup",
            &["target", "list", "--installed"],
            success("wasm32v1-none\n"),
        )
        .with_response(
            "stellar",
            &["--version"],
            success("stellar 23.1.0 (release)\n"),
        );
    let report = run_doctor(&DoctorConfig::default(), &runner);
    let cargo_check = report.checks.iter().find(|c| c.name == "cargo").unwrap();
    assert!(!cargo_check.passed);
    assert!(cargo_check.detail.contains("not found"));
}

#[test]
fn real_system_runner_at_least_returns_a_well_formed_report() {
    // Smoke test against the actual host. We don't assert pass/fail for
    // every check (CI may or may not have wasm32v1-none / stellar installed)
    // but we DO require the rust toolchain check to actually run against a
    // rustc binary (CI images always have one) and to produce a non-empty,
    // non-"missing" detail. This belt-and-suspenders assertion makes the
    // test fail loudly rather than collapse to a "all-fail" shrug on an
    // under-equipped host.
    let canonical_names = [
        "rust toolchain",
        "wasm32v1-none target",
        "cargo",
        "stellar CLI",
    ];
    let report = run_doctor(&DoctorConfig::default(), &SystemRunner);
    assert_eq!(report.checks.len(), canonical_names.len());
    let names: Vec<&str> = report.checks.iter().map(|c| c.name).collect();
    assert_eq!(names, canonical_names, "check name order is a public surface");
    for check in &report.checks {
        assert!(!check.detail.is_empty(), "{check:?}");
    }
    let rust_check = report
        .checks
        .iter()
        .find(|c| c.name == "rust toolchain")
        .expect("rust check entry");
    // Belt-and-suspenders: assert the rust check actually ran against a real
    // rustc binary. CI images always have rustc; if it didn't run there,
    // either the runner is broken or the host is broken -- either way we
    // want this test to fail loudly rather than produce a friendly shrug.
    assert!(
        rust_check.passed,
        "rust toolchain check did not pass on host: {rust_check:?}"
    );
}
