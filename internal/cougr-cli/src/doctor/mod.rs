//! `cougr doctor` implementation.
//!
//! The doctor is structured as a sequence of independent [`CheckResult`]s
//! returned by the [`checks`] module, then rendered into a single
//! [`Report`] that owns the formatting and exit-code semantics. Splitting
//! "compute the result" from "render the result" is what lets us test
//! check logic independently of binary IO (the check functions take a
//! [`runner::CommandRunner`] so tests inject canned outputs) and still
//! have one place to update the user-facing formatting.
//!
//! [`runner::CommandRunner`]: runner::CommandRunner

use std::path::PathBuf;

pub mod checks;
pub mod runner;

/// Canonical Soroban/WASM target. Mirrors the value used by the workspace's
/// build scripts and by the SDK. Kept as a constant so a test failure message
/// can reference it without re-typing the string.
pub const DOCTOR_TARGET: &str = "wasm32v1-none";

/// Minimum Rust version used when no `rust-version` field is found in a
/// manifest reachable from the current directory. The doctor prefers a value
/// parsed from a workspace `Cargo.toml` whenever it can find one.
pub const DOCTOR_FALLBACK_RUST_MIN_VERSION: &str = "1.70.0";

/// Minimum Stellar CLI version that the doctor considers acceptable. Picked
/// deliberately a few releases back so users aren't forced onto the bleeding
/// edge just to scaffold a project.
pub const DOCTOR_DEFAULT_STELLAR_MIN_VERSION: &str = "21.0.0";

/// Configuration consumed by [`run`].
#[derive(Debug, Clone)]
pub struct DoctorConfig {
    /// Optional path to a workspace `Cargo.toml` whose `rust-version` field
    /// defines the minimum Rust toolchain. `None` triggers the auto-discovery
    /// path ("walk up directories looking for `Cargo.toml`").
    pub rust_manifest: Option<PathBuf>,

    /// Minimum `stellar` CLI version. Always set; default comes from
    /// [`DOCTOR_DEFAULT_STELLAR_MIN_VERSION`].
    pub stellar_min_version: String,
}

impl Default for DoctorConfig {
    fn default() -> Self {
        Self {
            rust_manifest: None,
            stellar_min_version: DOCTOR_DEFAULT_STELLAR_MIN_VERSION.to_string(),
        }
    }
}

/// Outcome of a single check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    /// Stable identifier used for indexing and for nicer test failure output.
    /// Examples: `"rust toolchain"`, `"wasm32v1-none target"`.
    pub name: &'static str,

    /// Whether the check passed. Drives the green/red marker in [`print_report`].
    pub passed: bool,

    /// Short, single-line outcome. Should NOT contain the fix command;
    /// that's in [`CheckResult::fix`] so we don't duplicate it across renders.
    pub detail: String,

    /// Actionable fix command (or install link) shown when the check failed.
    /// `None` for passing checks. The first line of the message should be a
    /// shell command the user can copy-paste, when one exists.
    pub fix: Option<String>,
}

/// Aggregated doctor outcome.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub checks: Vec<CheckResult>,
}

impl Report {
    pub fn total_count(&self) -> usize {
        self.checks.len()
    }

    pub fn passed_count(&self) -> usize {
        self.checks.iter().filter(|c| c.passed).count()
    }

    pub fn failed_count(&self) -> usize {
        self.checks.iter().filter(|c| !c.passed).count()
    }

    pub fn all_passed(&self) -> bool {
        self.failed_count() == 0 && !self.checks.is_empty()
    }

    /// Build a passing check.
    pub fn ok(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            checks: vec![CheckResult::ok(name, detail)],
        }
    }
}

impl CheckResult {
    pub fn ok(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            passed: true,
            detail: detail.into(),
            fix: None,
        }
    }

    pub fn fail(
        name: &'static str,
        detail: impl Into<String>,
        fix: impl Into<String>,
    ) -> Self {
        Self {
            name,
            passed: false,
            detail: detail.into(),
            fix: Some(fix.into()),
        }
    }
}

/// Run every check and return a [`Report`]. Does NOT print anything.
///
/// [`runner::SystemRunner`] is the production implementation of
/// [`runner::CommandRunner`]; tests inject a mock runner that returns canned
/// stdout/stderr. This split is what allows
/// `tests/integration_doctor.rs`-style tests to live next to the binary
/// rather than being gated behind a hidden `--self-test` flag.
pub fn run(config: &DoctorConfig, runner: &dyn runner::CommandRunner) -> Report {
    let rust_min =
        resolve_min_rust_version(config.rust_manifest.as_deref());

    Report {
        checks: vec![
            checks::rust_toolchain::check(runner, &rust_min),
            checks::wasm32v1_target::check(runner),
            checks::cargo::check(runner),
            checks::stellar_cli::check(runner, &config.stellar_min_version),
        ],
    }
}

/// Render the report on stdout (or stderr when `warnings_only` is `true`).
///
/// `warnings_only = true` is what `cougr new` uses after a failed pre-flight
/// so the diagnostics are visible without polluting the success message of
/// the scaffold itself.
pub fn print_report(report: &Report, warnings_only: bool) {
    let total = report.total_count();
    // Pad to the longest check name so the alignment survives name changes.
    let label_width = report
        .checks
        .iter()
        .map(|c| c.name.chars().count())
        .max()
        .unwrap_or(0)
        .max(20);
    for (idx, check) in report.checks.iter().enumerate() {
        let marker = if check.passed { "PASS" } else { "FAIL" };
        let line = format!(
            "[{}/{}] {:<label_width$}  {marker:<4}  {}",
            idx + 1,
            total,
            check.name,
            check.detail,
        );
        if warnings_only {
            eprintln!("{line}");
        } else {
            println!("{line}");
        }
        if let Some(fix) = &check.fix {
            // Indent the fix-arrow to start at the same column as `detail`
            // in the line above. Format string is
            // "[{}/{}] {:<label_width$}  {marker:<4}  {}" so the offset is
            // "[X/X] ".len() + label_width + "  ".len() + marker_width + "  ".len().
            let marker_width = 4;
            let indent_cols =
                "[X/X] ".len() + label_width + 2 + marker_width + 2;
            let indent = " ".repeat(indent_cols);
            for sub in fix.lines() {
                let indented = format!("{indent}-> {sub}");
                if warnings_only {
                    eprintln!("{indented}");
                } else {
                    println!("{indented}");
                }
            }
        }
    }
    if warnings_only {
        eprintln!(
            "{}/{} checks passing; refer to warnings above.",
            report.passed_count(),
            total,
        );
    } else {
        let summary = format!(
            "{}/{} checks passed.",
            report.passed_count(),
            total,
        );
        println!("{summary}");
        if !report.all_passed() {
            eprintln!("error: toolchain checks failed; see messages above.");
        }
    }
}

/// Look for a `rust-version` field in the workspace `Cargo.toml`. If the
/// user passed `--rust-manifest PATH`, only that file is consulted;
/// otherwise we walk up from the current directory and STOP at the first
/// manifest that declares a `[workspace]`, matching the issue's "Root
/// Cargo.toml … as source of truth" requirement. Examples in this repository
/// do not declare their own `rust-version` field, so a naive nearest-match
/// would either find an unrelated field or fall back to the constant.
fn resolve_min_rust_version(override: Option<&std::path::Path>) -> String {
    let candidates: Vec<PathBuf> = if let Some(path) = override {
        vec![path.to_path_buf()]
    } else {
        discover_workspace_root()
    };

    for candidate in candidates {
        if let Ok(text) = std::fs::read_to_string(&candidate) {
            if let Some(version) = parse_rust_version_field(&text) {
                return version;
            }
            // A workspace manifest without its own `rust-version` field is
            // still the correct stop point -- we have the source of truth.
            if override.is_some() {
                break;
            }
        }
    }

    DOCTOR_FALLBACK_RUST_MIN_VERSION.to_string()
}

/// Walk up from `current_dir` looking for a `Cargo.toml` that declares a
/// `[workspace]`. Returns all candidates so a future iteration can prefer
/// "first manifest with `[workspace]`" but still discoverable when called
/// by recursive cousin tests.
fn discover_workspace_root() -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut here = std::env::current_dir().ok();
    while let Some(dir) = here {
        let candidate = dir.join("Cargo.toml");
        if candidate.is_file() {
            let is_workspace = std::fs::read_to_string(&candidate)
                .map(|text| text.contains("[workspace]"))
                .unwrap_or(false);
            found.push(candidate.clone());
            if is_workspace {
                break;
            }
        }
        here = dir.parent().map(|p| p.to_path_buf());
    }
    found
}

/// Minimal `rust-version = "X.Y.Z"` extractor. The Cargo manifest format has
/// many other `"field"`-style entries; we only want the one named `rust-version`.
fn parse_rust_version_field(manifest: &str) -> Option<String> {
    for line in manifest.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("rust-version") {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix('=') {
                let rest = rest.trim();
                if rest.starts_with('"') && rest.ends_with('"') && rest.len() >= 2 {
                    return Some(rest[1..rest.len() - 1].to_string());
                }
            }
        }
    }
    None
}

/// Parse a dotted version ("1.70.0") into `[u32; 3]`. Pre-release or build
/// suffixes (`-nightly`, `+abc`) are REJECTED — a `rustc 1.85.0-nightly`
/// toolchain is not a stable release and must not silently satisfy a stable
/// minimum. Trailing whitespace is tolerated.
pub fn parse_dotted_version(s: &str) -> Option<[u32; 3]> {
    let head = s.trim().trim_start_matches('v');
    if head.is_empty() || !head.chars().next()?.is_ascii_digit() {
        return None;
    }
    // Reject anything that isn't `digits[.digits[.digits]]` exactly. This
    // discards `nightly`, `+buildmeta`, etc.
    if !head.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
        return None;
    }
    let mut parts = head.split('.');
    let a = parts.next()?.parse::<u32>().ok()?;
    if a == 0 && head == "0" {
        return None;
    }
    let b = match parts.next() {
        Some(s) if !s.is_empty() => s.parse::<u32>().ok()?,
        _ => 0,
    };
    let c = match parts.next() {
        Some(s) if !s.is_empty() => s.parse::<u32>().ok()?,
        _ => 0,
    };
    if parts.next().is_some() {
        return None;
    }
    Some([a, b, c])
}

/// `actual >= required` against two dotted three-tuples.
pub fn version_at_least(actual: [u32; 3], required: [u32; 3]) -> bool {
    actual[0] > required[0]
        || (actual[0] == required[0] && actual[1] > required[1])
        || (actual[0] == required[0] && actual[1] == required[1] && actual[2] >= required[2])
}

/// Parse `rustc 1.85.0 (abc 2025-02-14)` and similar shape variants. We return
/// the first token that parses as a dotted version after splitting on
/// whitespace.
pub fn parse_rustc_version_line(line: &str) -> Option<[u32; 3]> {
    for token in line.split_whitespace().skip(1) {
        if let Some(v) = parse_dotted_version(token) {
            return Some(v);
        }
    }
    None
}

/// Parse `stellar 23.1.0 (release)` and similar shapes.
pub fn parse_stellar_version_line(line: &str) -> Option<[u32; 3]> {
    // `stellar --version` historically produces `stellar X.Y.Z (...)`. We
    // accept either "stellar X.Y.Z" or "cli X.Y.Z" defensively.
    for token in line.split_whitespace().skip(1) {
        if let Some(v) = parse_dotted_version(token) {
            return Some(v);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rust_version_field_finds_quoted_value() {
        let manifest = "\
            [package]\n\
            name = \"x\"\n\
            version = \"0.1.0\"\n\
            rust-version = \"1.70.0\"\n\
            edition = \"2021\"\n";
        assert_eq!(parse_rust_version_field(manifest), Some("1.70.0".into()));
    }

    #[test]
    fn parse_rust_version_field_ignores_other_keys() {
        let manifest = "version = \"1.70.0\"";
        assert_eq!(parse_rust_version_field(manifest), None);
    }

    #[test]
    fn parse_dotted_version_basic() {
        assert_eq!(parse_dotted_version("1.70.0"), Some([1, 70, 0]));
        assert_eq!(parse_dotted_version("v23.1.0"), Some([23, 1, 0]));
        assert_eq!(parse_dotted_version("0.0"), Some([0, 0, 0]));
        assert_eq!(parse_dotted_version("1"), Some([1, 0, 0]));
    }

    #[test]
    fn parse_dotted_version_rejects_non_numeric() {
        assert_eq!(parse_dotted_version("abc"), None);
    }

    #[test]
    fn parse_dotted_version_rejects_pre_release() {
        // A nightly toolchain cannot satisfy a stable minimum. Pre-release
        // info must surface as a parse failure rather than silently stripping.
        assert_eq!(parse_dotted_version("1.85.0-nightly"), None);
        assert_eq!(parse_dotted_version("1.85.0+abc"), None);
        assert_eq!(parse_dotted_version("1.85.0-beta.1"), None);
        assert_eq!(parse_dotted_version("1.85.0 "), Some([1, 85, 0]));
    }

    #[test]
    fn parse_dotted_version_rejects_internal_whitespace() {
        // The strict parser must not silently tolerate whitespace anywhere
        // except the leading/trailing trim.
        assert_eq!(parse_dotted_version("1.85 .0"), None);
        assert_eq!(parse_dotted_version("1 .85.0"), None);
        assert_eq!(parse_dotted_version("1\t85.0"), None);
    }

    #[test]
    fn parse_dotted_version_accepts_surrounding_whitespace() {
        assert_eq!(parse_dotted_version("   1.85.0   "), Some([1, 85, 0]));
        assert_eq!(parse_dotted_version("\t1.85.0\t"), Some([1, 85, 0]));
    }

    #[test]
    fn parse_dotted_version_rejects_too_many_components() {
        assert_eq!(parse_dotted_version("1.2.3.4"), None);
    }

    #[test]
    fn parse_dotted_version_rejects_empty_segments() {
        assert_eq!(parse_dotted_version("1..0"), None);
        assert_eq!(parse_dotted_version(".1.0"), None);
    }

    #[test]
    fn version_at_least_compares_correctly() {
        assert!(version_at_least([1, 70, 0], [1, 70, 0]));
        assert!(version_at_least([1, 71, 0], [1, 70, 0]));
        assert!(version_at_least([2, 0, 0], [1, 70, 0]));
        assert!(!version_at_least([1, 69, 99], [1, 70, 0]));
    }

    #[test]
    fn rustc_line_parser_finds_first_numeric_token() {
        assert_eq!(
            parse_rustc_version_line("rustc 1.85.0 (abc 2025)"),
            Some([1, 85, 0])
        );
    }

    #[test]
    fn stellar_line_parser_finds_version_token() {
        assert_eq!(
            parse_stellar_version_line("stellar 23.1.0 (release)"),
            Some([23, 1, 0])
        );
    }

    #[test]
    fn report_counts_passed_and_failed() {
        let report = Report {
            checks: vec![
                CheckResult::ok("a", ""),
                CheckResult::fail("b", "missing", "do thing"),
                CheckResult::ok("c", ""),
            ],
        };
        assert_eq!(report.total_count(), 3);
        assert_eq!(report.passed_count(), 2);
        assert_eq!(report.failed_count(), 1);
        assert!(!report.all_passed());
    }
}
