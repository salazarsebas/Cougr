//! Toolchain diagnostics for `cougr doctor`.
//!
//! Verifies that the local environment has everything needed to build
//! and test a Cougr project: Rust toolchain, wasm32v1-none target,
//! stellar CLI, and cargo. Each check produces a pass/fail result with
//! an actionable fix command on failure.
//!
//! Per the product spec (docs/strategy/06-product-strategy.md) and
//! issue #247, `cougr doctor` diagnoses and instructs — it does NOT
//! modify the user's system.

use anyhow::Result;
use std::process::{exit, Command};

// ---------------------------------------------------------------------------
// Minimum versions (sourced from root Cargo.toml and ecosystem docs)
// ---------------------------------------------------------------------------

/// Minimum Rust version from root `Cargo.toml` (`rust-version`).
const MIN_RUST_VERSION: &str = "1.70.0";

/// Minimum Stellar CLI version known to work with cougr-core 1.1.0.
/// See: https://developers.stellar.org/docs/tools/cli/stellar-cli
const MIN_STELLAR_CLI_VERSION: &str = "21.0.0";

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run all toolchain diagnostic checks.
///
/// Prints pass/fail results and exits non-zero if any check fails.
/// Returns `Ok(())` when all checks pass so callers can chain.
pub fn run() -> Result<()> {
    let checks = [
        check_rust_version(),
        check_wasm_target(),
        check_stellar_cli(),
        check_cargo(),
    ];

    println!("=== Cougr Doctor — Toolchain Diagnostics ===\n");

    let mut passed = 0u32;
    let total = checks.len() as u32;

    for check in &checks {
        let icon = if check.pass { "✓" } else { "✗" };
        println!("  {}  {}", icon, check.label);
        if check.pass {
            println!("      {}", check.detail);
            passed += 1;
        } else {
            println!("      ✗  {}", check.detail);
            if let Some(ref fix) = check.fix {
                println!("      →  Fix: {}", fix);
            }
        }
        println!();
    }

    println!("=== {}/{} checks passed ===", passed, total);

    if passed < total {
        exit(1);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Check result type
// ---------------------------------------------------------------------------

struct CheckResult {
    label: String,
    pass: bool,
    detail: String,
    fix: Option<String>,
}

impl CheckResult {
    fn ok(label: &str, detail: &str) -> Self {
        CheckResult {
            label: label.to_string(),
            pass: true,
            detail: detail.to_string(),
            fix: None,
        }
    }

    fn fail(label: &str, detail: &str, fix: &str) -> Self {
        CheckResult {
            label: label.to_string(),
            pass: false,
            detail: detail.to_string(),
            fix: Some(fix.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Check 1: Rust toolchain version
// ---------------------------------------------------------------------------

fn check_rust_version() -> CheckResult {
    let output = match Command::new("rustc").arg("--version").output() {
        Ok(o) => o,
        Err(e) => {
            return CheckResult::fail(
                "Rust toolchain",
                &format!("rustc not found: {}", e),
                "Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
            );
        }
    };

    if !output.status.success() {
        return CheckResult::fail(
            "Rust toolchain",
            "rustc --version returned an error",
            "Reinstall Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let installed = parse_rustc_version(&stdout);

    match installed {
        Some(ref version) => {
            if version_cmp(version, MIN_RUST_VERSION) >= 0 {
                CheckResult::ok(
                    "Rust toolchain",
                    &format!("rustc {} (≥ {})", version, MIN_RUST_VERSION),
                )
            } else {
                CheckResult::fail(
                    "Rust toolchain",
                    &format!(
                        "rustc {} is older than minimum {}",
                        version, MIN_RUST_VERSION
                    ),
                    &format!("rustup update stable"),
                )
            }
        }
        None => CheckResult::fail(
            "Rust toolchain",
            &format!("could not parse rustc version from: {}", stdout.trim()),
            "Check your Rust installation: rustup update stable",
        ),
    }
}

/// Parse version from `rustc --version` output, e.g. `rustc 1.70.0 (90c541806 2023-05-31)`.
fn parse_rustc_version(output: &str) -> Option<String> {
    // Expected format: "rustc X.Y.Z (...)"
    let parts: Vec<&str> = output.split_whitespace().collect();
    if parts.len() >= 2 {
        let version = parts[1];
        // Validate it looks like a semver
        if version.chars().filter(|c| *c == '.').count() >= 2
            && version.chars().all(|c| c.is_ascii_digit() || c == '.')
        {
            return Some(version.to_string());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Check 2: wasm32v1-none target
// ---------------------------------------------------------------------------

fn check_wasm_target() -> CheckResult {
    let output = match Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return CheckResult::fail(
                "wasm32v1-none target",
                &format!("rustup not found: {}", e),
                "Install rustup: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
            );
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.contains("wasm32v1-none") {
        CheckResult::ok(
            "wasm32v1-none target",
            "wasm32v1-none target installed",
        )
    } else {
        CheckResult::fail(
            "wasm32v1-none target",
            "wasm32v1-none target not installed",
            "rustup target add wasm32v1-none",
        )
    }
}

// ---------------------------------------------------------------------------
// Check 3: stellar CLI
// ---------------------------------------------------------------------------

fn check_stellar_cli() -> CheckResult {
    // Try `stellar --version` first (newer CLI), fall back to `stellar version`
    let output = Command::new("stellar").arg("--version").output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => {
            // Fallback: try `stellar version` (older CLI)
            match Command::new("stellar").arg("version").output() {
                Ok(o) if o.status.success() => o,
                Ok(_) => {
                    return CheckResult::fail(
                        "stellar CLI",
                        "stellar CLI returned an error",
                        "Install stellar CLI: see https://developers.stellar.org/docs/tools/cli/stellar-cli",
                    );
                }
                Err(e) => {
                    return CheckResult::fail(
                        "stellar CLI",
                        &format!("stellar CLI not found: {}", e),
                        "Install stellar CLI: see https://developers.stellar.org/docs/tools/cli/stellar-cli",
                    );
                }
            }
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{} {}", stdout.trim(), stderr.trim());

    match parse_stellar_version(&combined) {
        Some(ref version) => {
            if version_cmp(version, MIN_STELLAR_CLI_VERSION) >= 0 {
                CheckResult::ok(
                    "stellar CLI",
                    &format!("stellar {} (≥ {})", version, MIN_STELLAR_CLI_VERSION),
                )
            } else {
                CheckResult::fail(
                    "stellar CLI",
                    &format!(
                        "stellar {} is older than minimum {}",
                        version, MIN_STELLAR_CLI_VERSION
                    ),
                    "Upgrade stellar CLI: see https://developers.stellar.org/docs/tools/cli/stellar-cli#install",
                )
            }
        }
        None => {
            // Could not parse version, but the binary runs — pass with a note
            CheckResult::ok(
                "stellar CLI",
                &format!(
                    "stellar CLI found (could not parse version from: {})",
                    combined.trim()
                ),
            )
        }
    }
}

/// Parse version from stellar CLI output.
/// Handles formats like `stellar 21.0.0 (abcdef)` or `stellar 21.0.0`.
fn parse_stellar_version(output: &str) -> Option<String> {
    // Look for "stellar X.Y.Z" pattern
    for word in output.split_whitespace() {
        let cleaned = word.trim_matches(|c: char| c == '(' || c == ')' || c == ',');
        if cleaned.chars().filter(|c| *c == '.').count() >= 2
            && cleaned.chars().all(|c| c.is_ascii_digit() || c == '.')
            && cleaned.len() >= 5
        {
            return Some(cleaned.to_string());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Check 4: cargo (sanity check)
// ---------------------------------------------------------------------------

fn check_cargo() -> CheckResult {
    let output = match Command::new("cargo").arg("--version").output() {
        Ok(o) => o,
        Err(e) => {
            return CheckResult::fail(
                "cargo",
                &format!("cargo not found: {}", e),
                "Install Rust (includes cargo): curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
            );
        }
    };

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        CheckResult::ok("cargo", &format!("{}", stdout.trim()))
    } else {
        CheckResult::fail(
            "cargo",
            "cargo --version returned an error",
            "Reinstall Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
        )
    }
}

// ---------------------------------------------------------------------------
// Simple semver comparison (avoids adding a dependency)
// ---------------------------------------------------------------------------

/// Compare two semver strings.
/// Returns `Ordering::Greater` if `a > b`, `Equal` if same, `Less` if `a < b`.
fn version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |v: &str| -> Vec<u64> {
        v.split('.')
            .filter_map(|s| s.parse::<u64>().ok())
            .collect()
    };

    let va = parse(a);
    let vb = parse(b);

    for i in 0..va.len().max(vb.len()) {
        let na = va.get(i).copied().unwrap_or(0);
        let nb = vb.get(i).copied().unwrap_or(0);
        match na.cmp(&nb) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rustc_version_valid() {
        assert_eq!(
            parse_rustc_version("rustc 1.70.0 (90c541806 2023-05-31)"),
            Some("1.70.0".to_string())
        );
        assert_eq!(
            parse_rustc_version("rustc 1.82.0-nightly (a9bb6ca05 2025-01-15)"),
            Some("1.82.0".to_string())
        );
    }

    #[test]
    fn test_parse_rustc_version_invalid() {
        assert_eq!(parse_rustc_version(""), None);
        assert_eq!(parse_rustc_version("not rustc output"), None);
    }

    #[test]
    fn test_parse_stellar_version_valid() {
        assert_eq!(
            parse_stellar_version("stellar 21.0.0 (abcdef)"),
            Some("21.0.0".to_string())
        );
        assert_eq!(
            parse_stellar_version("stellar 22.1.3"),
            Some("22.1.3".to_string())
        );
    }

    #[test]
    fn test_parse_stellar_version_invalid() {
        assert_eq!(parse_stellar_version(""), None);
        assert_eq!(parse_stellar_version("unknown output"), None);
    }

    #[test]
    fn test_version_cmp() {
        assert_eq!(version_cmp("1.70.0", "1.70.0"), std::cmp::Ordering::Equal);
        assert_eq!(version_cmp("1.71.0", "1.70.0"), std::cmp::Ordering::Greater);
        assert_eq!(version_cmp("1.70.0", "1.71.0"), std::cmp::Ordering::Less);
        assert_eq!(version_cmp("1.70.1", "1.70.0"), std::cmp::Ordering::Greater);
        assert_eq!(version_cmp("2.0.0", "1.99.0"), std::cmp::Ordering::Greater);
        assert_eq!(version_cmp("21.0.0", "20.5.0"), std::cmp::Ordering::Greater);
    }
}
