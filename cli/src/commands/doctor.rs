//! `cougr doctor` - validate the local development environment.
//!
//! Checks that the toolchain, compilation targets, and external tools required
//! to build and deploy a Cougr project are all present and meet the minimum
//! versions specified in the workspace `Cargo.toml`.
//!
//! Each check reports **pass** or **fail** and, on failure, prints the exact
//! command needed to fix the problem. The command exits 0 only when every
//! check passes.
//!
//! This function is also called non-fatally from `cougr new` to surface
//! environment problems before the developer discovers them at build time.

use std::fmt;
use std::process::Command;

// ── Minimum versions ─────────────────────────────────────────────────────────
//
// The Rust minimum comes from the root workspace `Cargo.toml` (`rust-version =
// "1.70.0"`).  The stellar-cli minimum tracks the first release of the
// current stable v21 series.

/// Minimum Rust version required by the workspace (from root `Cargo.toml`).
const MIN_RUST: (u64, u64, u64) = (1, 70, 0);

/// Minimum `stellar` CLI version that ships the v21 API surface.
const MIN_STELLAR: (u64, u64, u64) = (21, 0, 0);

/// WASM compilation target required for Soroban contracts.
const WASM_TARGET: &str = "wasm32v1-none";

// ── Result type ──────────────────────────────────────────────────────────────

#[derive(Debug)]
enum CheckStatus {
    Pass(String),
    Fail { detail: String, fix: String },
    Warn { detail: String, fix: String },
}

struct CheckResult {
    name: &'static str,
    status: CheckStatus,
}

impl CheckResult {
    fn pass(name: &'static str, detail: impl Into<String>) -> Self {
        CheckResult {
            name,
            status: CheckStatus::Pass(detail.into()),
        }
    }

    fn fail(name: &'static str, detail: impl Into<String>, fix: impl Into<String>) -> Self {
        CheckResult {
            name,
            status: CheckStatus::Fail {
                detail: detail.into(),
                fix: fix.into(),
            },
        }
    }

    fn warn(name: &'static str, detail: impl Into<String>, fix: impl Into<String>) -> Self {
        CheckResult {
            name,
            status: CheckStatus::Warn {
                detail: detail.into(),
                fix: fix.into(),
            },
        }
    }

    fn is_pass(&self) -> bool {
        matches!(self.status, CheckStatus::Pass(_))
    }

    fn is_fail(&self) -> bool {
        matches!(self.status, CheckStatus::Fail { .. })
    }
}

// ── Version parsing ───────────────────────────────────────────────────────────

/// Parse a semver-like `"MAJOR.MINOR.PATCH"` string into a tuple.
/// Returns `None` if parsing fails.
fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let s = s.trim();
    let parts: Vec<&str> = s.splitn(4, '.').collect();
    if parts.len() < 3 {
        return None;
    }
    let major = parts[0].parse().ok()?;
    // Strip any pre-release suffix from the patch segment (e.g. "0-beta.1")
    let patch_str = parts[2].split('-').next().unwrap_or("0");
    let minor = parts[1].parse().ok()?;
    let patch = patch_str.trim().parse().ok()?;
    Some((major, minor, patch))
}

fn version_ok(actual: (u64, u64, u64), minimum: (u64, u64, u64)) -> bool {
    actual >= minimum
}

fn version_string(v: (u64, u64, u64)) -> String {
    format!("{}.{}.{}", v.0, v.1, v.2)
}

// ── Individual checks ─────────────────────────────────────────────────────────

/// Check 1: `cargo` is on PATH (sanity check).
fn check_cargo() -> CheckResult {
    match Command::new("cargo").arg("--version").output() {
        Ok(out) if out.status.success() => {
            let raw = String::from_utf8_lossy(&out.stdout);
            let version = raw
                .split_whitespace()
                .nth(1)
                .unwrap_or("unknown")
                .to_string();
            CheckResult::pass("cargo", format!("cargo {version}"))
        }
        _ => CheckResult::fail(
            "cargo",
            "cargo not found on PATH",
            "Install Rust via rustup: https://rustup.rs",
        ),
    }
}

/// Check 2: `rustup` / Rust toolchain present and meets `MIN_RUST`.
fn check_rust() -> CheckResult {
    let output = match Command::new("rustc").arg("--version").output() {
        Ok(o) => o,
        Err(_) => {
            return CheckResult::fail(
                "rust",
                "rustc not found on PATH",
                "Install Rust via rustup: https://rustup.rs",
            );
        }
    };

    if !output.status.success() {
        return CheckResult::fail(
            "rust",
            "rustc exited with an error",
            "Install Rust via rustup: https://rustup.rs",
        );
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    // rustc --version → "rustc 1.82.0 (f6e511eec 2024-10-15)"
    let version_str = raw.split_whitespace().nth(1).unwrap_or("0.0.0");

    match parse_version(version_str) {
        None => CheckResult::warn(
            "rust",
            format!("could not parse rustc version from: {raw}"),
            format!(
                "rustup update  # ensure Rust >= {}",
                version_string(MIN_RUST)
            ),
        ),
        Some(actual) if !version_ok(actual, MIN_RUST) => CheckResult::fail(
            "rust",
            format!(
                "rustc {} is below the minimum {}",
                version_string(actual),
                version_string(MIN_RUST)
            ),
            format!(
                "rustup update  # upgrades to the latest stable (>= {})",
                version_string(MIN_RUST)
            ),
        ),
        Some(actual) => CheckResult::pass("rust", format!("rustc {}", version_string(actual))),
    }
}

/// Check 3: `wasm32v1-none` target is installed.
fn check_wasm_target() -> CheckResult {
    let output = match Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
    {
        Ok(o) => o,
        Err(_) => {
            // rustup not found; the target cannot be verified.
            return CheckResult::warn(
                "wasm32v1-none",
                "rustup not found - cannot verify wasm32v1-none target",
                format!("Install rustup (https://rustup.rs) then: rustup target add {WASM_TARGET}"),
            );
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.lines().any(|l| l.trim() == WASM_TARGET) {
        CheckResult::pass("wasm32v1-none", format!("{WASM_TARGET} target installed"))
    } else {
        CheckResult::fail(
            "wasm32v1-none",
            format!("{WASM_TARGET} target is not installed"),
            format!("rustup target add {WASM_TARGET}"),
        )
    }
}

/// Check 4: `stellar` CLI is on PATH and meets `MIN_STELLAR`.
fn check_stellar() -> CheckResult {
    let output = match Command::new("stellar").arg("--version").output() {
        Ok(o) => o,
        Err(_) => {
            return CheckResult::fail(
                "stellar-cli",
                "stellar CLI not found on PATH",
                "cargo install --locked stellar-cli  \
                 # see https://developers.stellar.org/docs/tools/cli/stellar-cli",
            );
        }
    };

    if !output.status.success() {
        return CheckResult::fail(
            "stellar-cli",
            "stellar --version exited with an error",
            "cargo install --locked stellar-cli  \
             # see https://developers.stellar.org/docs/tools/cli/stellar-cli",
        );
    }

    // `stellar --version` may write to stderr on some releases
    let raw_stdout = String::from_utf8_lossy(&output.stdout);
    let raw_stderr = String::from_utf8_lossy(&output.stderr);
    let raw = if raw_stdout.trim().is_empty() {
        raw_stderr
    } else {
        raw_stdout
    };

    // Output is typically "stellar 21.3.0" or "stellar-cli 21.3.0"
    // Find the first token that looks like a version number.
    let version_str = raw
        .split_whitespace()
        .find(|t| t.starts_with(|c: char| c.is_ascii_digit()))
        .unwrap_or("0.0.0");

    match parse_version(version_str) {
        None => CheckResult::warn(
            "stellar-cli",
            format!("could not parse stellar version from: {}", raw.trim()),
            format!(
                "cargo install --locked stellar-cli  \
                 # ensure stellar-cli >= {}",
                version_string(MIN_STELLAR)
            ),
        ),
        Some(actual) if !version_ok(actual, MIN_STELLAR) => CheckResult::fail(
            "stellar-cli",
            format!(
                "stellar CLI {} is below the minimum {}",
                version_string(actual),
                version_string(MIN_STELLAR)
            ),
            format!(
                "cargo install --locked stellar-cli  \
                 # upgrades to the latest stable (>= {}); \
                 see https://developers.stellar.org/docs/tools/cli/stellar-cli",
                version_string(MIN_STELLAR)
            ),
        ),
        Some(actual) => CheckResult::pass(
            "stellar-cli",
            format!("stellar CLI {}", version_string(actual)),
        ),
    }
}

// ── Display helpers ───────────────────────────────────────────────────────────

const PASS: &str = "✔";
const FAIL: &str = "✘";
const WARN: &str = "⚠";

fn print_result(r: &CheckResult) {
    match &r.status {
        CheckStatus::Pass(detail) => {
            println!("  {PASS} {:<20} {detail}", r.name);
        }
        CheckStatus::Fail { detail, fix } => {
            println!("  {FAIL} {:<20} {detail}", r.name);
            println!("    fix: {fix}");
        }
        CheckStatus::Warn { detail, fix } => {
            println!("  {WARN} {:<20} {detail}", r.name);
            println!("    fix: {fix}");
        }
    }
}

// ── Public entry points ───────────────────────────────────────────────────────

/// Run all environment checks and print a report to stdout.
///
/// Returns `Ok(())` when every check passes, or an error summary when one or
/// more checks fail.  Warnings do not cause a non-zero exit.
pub fn run() -> Result<(), DoctorError> {
    println!("cougr doctor - environment check");
    println!();

    let results = [
        check_cargo(),
        check_rust(),
        check_wasm_target(),
        check_stellar(),
    ];

    for r in &results {
        print_result(r);
    }

    let total = results.len();
    let passed = results.iter().filter(|r| r.is_pass()).count();
    let failed = results.iter().filter(|r| r.is_fail()).count();

    println!();

    if failed == 0 {
        println!("{passed}/{total} checks passed");
        Ok(())
    } else {
        println!("{passed}/{total} checks passed - {failed} check(s) failed");
        Err(DoctorError(failed))
    }
}

/// Run all checks silently; print a one-line warning only when something fails.
///
/// This is called from `cougr new` as a non-fatal advisory step.  It never
/// returns an error: the caller should continue regardless of the outcome.
pub fn run_as_warning() {
    let results = [
        check_cargo(),
        check_rust(),
        check_wasm_target(),
        check_stellar(),
    ];

    let total = results.len();
    let passed = results.iter().filter(|r| r.is_pass()).count();

    if passed < total {
        let failed = total - passed;
        eprintln!(
            "warning: {failed} environment check(s) failed. \
             Run `cougr doctor` for details and fix commands."
        );
    }
}

// ── Error type ────────────────────────────────────────────────────────────────

/// Returned by [`run`] when one or more checks fail.  Wraps the failure count.
#[derive(Debug)]
pub struct DoctorError(pub usize);

impl fmt::Display for DoctorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} environment check(s) failed", self.0)
    }
}

impl std::error::Error for DoctorError {}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_version ────────────────────────────────────────────────────────

    #[test]
    fn parse_version_handles_standard_semver() {
        assert_eq!(parse_version("1.82.0"), Some((1, 82, 0)));
        assert_eq!(parse_version("21.3.0"), Some((21, 3, 0)));
        assert_eq!(parse_version("0.0.1"), Some((0, 0, 1)));
    }

    #[test]
    fn parse_version_strips_prerelease_suffix() {
        assert_eq!(parse_version("1.70.0-beta.1"), Some((1, 70, 0)));
        assert_eq!(parse_version("21.0.0-rc.2"), Some((21, 0, 0)));
    }

    #[test]
    fn parse_version_returns_none_for_garbage() {
        assert_eq!(parse_version("not-a-version"), None);
        assert_eq!(parse_version("1.2"), None);
        assert_eq!(parse_version(""), None);
    }

    // ── version_ok ──────────────────────────────────────────────────────────

    #[test]
    fn version_ok_exact_match() {
        assert!(version_ok((1, 70, 0), (1, 70, 0)));
    }

    #[test]
    fn version_ok_newer_is_fine() {
        assert!(version_ok((1, 82, 0), (1, 70, 0)));
        assert!(version_ok((2, 0, 0), (1, 99, 99)));
    }

    #[test]
    fn version_ok_older_fails() {
        assert!(!version_ok((1, 69, 9), (1, 70, 0)));
        assert!(!version_ok((20, 99, 99), (21, 0, 0)));
    }

    // ── CheckResult helpers ──────────────────────────────────────────────────

    #[test]
    fn pass_result_is_pass() {
        let r = CheckResult::pass("test", "ok");
        assert!(r.is_pass());
    }

    #[test]
    fn fail_result_is_not_pass() {
        let r = CheckResult::fail("test", "broken", "fix it");
        assert!(!r.is_pass());
    }

    #[test]
    fn warn_result_is_not_pass() {
        let r = CheckResult::warn("test", "unclear", "try this");
        assert!(!r.is_pass());
        assert!(!r.is_fail());
    }

    #[test]
    fn fail_result_is_fail() {
        let r = CheckResult::fail("test", "broken", "fix it");
        assert!(r.is_fail());
    }

    // ── check_cargo ──────────────────────────────────────────────────────────

    #[test]
    fn cargo_check_passes_in_ci() {
        // cargo is always available during `cargo test`
        let r = check_cargo();
        assert!(r.is_pass(), "cargo should be available during tests");
    }

    // ── check_rust ───────────────────────────────────────────────────────────

    #[test]
    fn rust_check_passes_in_ci() {
        // rustc is always available during `cargo test`
        let r = check_rust();
        assert!(
            r.is_pass(),
            "rustc should be available and >= 1.70.0 during tests"
        );
    }
}
