//! Rust toolchain check.
//!
//! Runs `rustc --version`, parses the version out of the first numeric token
//! after `rustc`, and compares against the minimum resolved from the
//! workspace `Cargo.toml`. On failure, prints `rustup update stable` as the
//! fix; on missing binary, prints the rustup install URL because rustup is the
//! only supported install path that produces a single managed toolchain.

use crate::doctor::{
    parse_rustc_version_line, version_at_least, CheckResult, CommandRunner,
};

pub fn check(runner: &dyn CommandRunner, min_version: &str) -> CheckResult {
    let min = match crate::doctor::parse_dotted_version(min_version) {
        Some(v) => v,
        None => {
            return CheckResult::fail(
                "rust toolchain",
                format!("could not parse minimum version `{min_version}`"),
                format!("set `rust-version` to a valid semver like \"1.70.0\" in Cargo.toml"),
            )
        }
    };

    let out = runner.run("rustc", &["--version"]);
    if out.status == -1 {
        return CheckResult::fail(
            "rust toolchain",
            "rustc not found on PATH",
            "Install rustup from https://rustup.rs and run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
        );
    }
    if out.status != 0 {
        return CheckResult::fail(
            "rust toolchain",
            format!("rustc --version exited with status {}", out.status),
            "Re-run `rustup update stable` to repair the toolchain",
        );
    }
    let actual = match parse_rustc_version_line(&out.stdout) {
        Some(v) => v,
        None => {
            return CheckResult::fail(
                "rust toolchain",
                format!("could not parse rustc output: {:?}", out.stdout),
                "Re-run `rustup update stable`",
            )
        }
    };
    if version_at_least(actual, min) {
        let detail = format!("rustc {}.{}.{} (meets {}.{}.{})", actual[0], actual[1], actual[2], min[0], min[1], min[2]);
        CheckResult::ok("rust toolchain", detail)
    } else {
        CheckResult::fail(
            "rust toolchain",
            format!(
                "rustc {}.{}.{} is below the required {}.{}.{}",
                actual[0], actual[1], actual[2], min[0], min[1], min[2]
            ),
            "rustup update stable",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::runner::{CommandOutput, MockRunner};

    #[test]
    fn pass_when_rustc_meets_minimum() {
        let runner = MockRunner::new().with_response(
            "rustc",
            &["--version"],
            CommandOutput::success("rustc 1.85.0 (abc 2025)"),
        );
        let result = check(&runner, "1.70.0");
        assert!(result.passed, "{:?}", result);
        assert!(result.detail.contains("meets"));
    }

    #[test]
    fn fail_when_rustc_below_minimum() {
        let runner = MockRunner::new().with_response(
            "rustc",
            &["--version"],
            CommandOutput::success("rustc 1.69.0 (abc 2025)"),
        );
        let result = check(&runner, "1.70.0");
        assert!(!result.passed);
        assert_eq!(result.fix.as_deref(), Some("rustup update stable"));
    }

    #[test]
    fn fail_when_rustc_not_installed() {
        // Register an explicit "missing binary" response so the check routes
        // through the genuine "not found on PATH" branch rather than the
        // parse-failure fallback. MockRunner returns status=-1 (missing())
        // for any call without a registered response, so this also covers
        // the case where callers forget to register.
        let runner = MockRunner::new().with_response(
            "rustc",
            &["--version"],
            crate::doctor::runner::CommandOutput::missing(),
        );
        let result = check(&runner, "1.70.0");
        assert!(!result.passed, "{result:?}");
        assert!(result.detail.contains("not found"), "{result:?}");
        assert_eq!(
            result.fix.as_deref(),
            Some(
                "Install rustup from https://rustup.rs and run: \
                 curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
            )
        );
    }

    #[test]
    fn fail_when_rustc_output_unparseable() {
        let runner = MockRunner::new().with_response(
            "rustc",
            &["--version"],
            CommandOutput::success("rustc (unknown)"),
        );
        let result = check(&runner, "1.70.0");
        assert!(!result.passed);
        assert!(result.detail.contains("could not parse"));
    }

    #[test]
    fn fail_when_rustc_is_nightly() {
        // A nightly or beta toolchain cannot satisfy a stable minimum. The
        // check must surface this rather than passing because parse_dotted_version
        // strips the pre-release label.
        let runner = MockRunner::new().with_response(
            "rustc",
            &["--version"],
            CommandOutput::success("rustc 1.85.0-nightly"),
        );
        let result = check(&runner, "1.70.0");
        assert!(!result.passed, "{result:?}");
        assert!(result.detail.contains("could not parse"), "{result:?}");
    }

    #[test]
    fn fail_when_min_version_is_garbage() {
        let runner = MockRunner::new().with_response(
            "rustc",
            &["--version"],
            CommandOutput::success("rustc 1.85.0"),
        );
        let result = check(&runner, "not-a-version");
        assert!(!result.passed);
        assert!(result.detail.contains("could not parse"));
    }
}
