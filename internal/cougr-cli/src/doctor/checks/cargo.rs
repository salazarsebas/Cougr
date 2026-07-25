//! Cargo sanity check.
//!
//! This is a "is cargo on PATH and can answer `--version`" probe rather than
//! a "does cargo match a minimum version" probe; cargo is shipped by rustup
//! and tracks the Rust toolchain, so a separate version floor would just
//! duplicate the rustc check above.

use crate::doctor::{runner::CommandRunner, CheckResult};

pub fn check(runner: &dyn CommandRunner) -> CheckResult {
    let out = runner.run("cargo", &["--version"]);
    if out.status == -1 {
        return CheckResult::fail(
            "cargo",
            "cargo not found on PATH",
            "Install Rust (which includes cargo) from https://rustup.rs",
        );
    }
    if out.status != 0 {
        return CheckResult::fail(
            "cargo",
            format!("cargo --version exited with status {}", out.status),
            "Re-run `rustup update stable`; cargo ships with rustup",
        );
    }
    let trimmed = out.stdout.trim();
    if trimmed.is_empty() {
        return CheckResult::fail(
            "cargo",
            "cargo --version returned no output",
            "Re-run `rustup update stable`; this usually means the toolchain is half-installed",
        );
    }
    CheckResult::ok("cargo", trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::runner::{CommandOutput, MockRunner};

    #[test]
    fn pass_when_cargo_reports_version() {
        let runner = MockRunner::new().with_response(
            "cargo",
            &["--version"],
            CommandOutput::success("cargo 1.85.0 (abc 2025)\n"),
        );
        let result = check(&runner);
        assert!(result.passed, "{:?}", result);
        assert!(result.detail.contains("1.85.0"));
    }

    #[test]
    fn fail_when_cargo_missing() {
        // Explicit `CommandOutput::missing()` rather than relying on
        // MockRunner's default. Either path exercises the same `status == -1`
        // branch, but registering `missing()` documents the test's intent.
        let runner = MockRunner::new().with_response(
            "cargo",
            &["--version"],
            CommandOutput::missing(),
        );
        let result = check(&runner);
        assert!(!result.passed, "{result:?}");
        assert!(result.detail.contains("not found"), "{result:?}");
        assert!(
            result.fix.as_deref().unwrap().contains("rustup.rs"),
            "{result:?}"
        );
    }

    #[test]
    fn fail_when_cargo_returns_empty() {
        let runner = MockRunner::new().with_response(
            "cargo",
            &["--version"],
            CommandOutput::success(""),
        );
        let result = check(&runner);
        assert!(!result.passed);
        assert!(result.detail.contains("no output"));
    }
}
