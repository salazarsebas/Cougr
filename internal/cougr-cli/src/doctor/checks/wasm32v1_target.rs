//! `wasm32v1-none` compilation target check.
//!
//! Runs `rustup target list --installed` and confirms an entry exactly
//! matching the [`DOCTOR_TARGET`] constant. The fix command is the canonical
//! rustup `target add` invocation. We deliberately do NOT use
//! `rustc --print target-list` here because that lists every target rustc
//! knows about, which would always pass even if rustup hadn't actually
//! fetched the std libs for the target locally.

use crate::doctor::{runner::CommandRunner, CheckResult, DOCTOR_TARGET};

pub fn check(runner: &dyn CommandRunner) -> CheckResult {
    let out = runner.run("rustup", &["target", "list", "--installed"]);
    if out.status == -1 {
        return CheckResult::fail(
            "wasm32v1-none target",
            "rustup not found on PATH",
            "Install rustup from https://rustup.rs, then: rustup target add wasm32v1-none",
        );
    }
    if out.status != 0 {
        return CheckResult::fail(
            "wasm32v1-none target",
            format!("`rustup target list --installed` exited with status {}", out.status),
            "Re-run `rustup update stable` then `rustup target add wasm32v1-none`",
        );
    }
    let mut found = false;
    let mut found_line = String::new();
    for line in out.stdout.lines() {
        if line.trim() == DOCTOR_TARGET {
            found = true;
            found_line = line.trim().to_string();
            break;
        }
    }
    if found {
        CheckResult::ok("wasm32v1-none target", format!("installed ({found_line})"))
    } else {
        CheckResult::fail(
            "wasm32v1-none target",
            "not installed",
            format!("rustup target add {DOCTOR_TARGET}"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::runner::{CommandOutput, MockRunner};

    #[test]
    fn pass_when_target_present() {
        let runner = MockRunner::new().with_response(
            "rustup",
            &["target", "list", "--installed"],
            CommandOutput::success("wasm32v1-none\nx86_64-unknown-linux-gnu\n"),
        );
        let result = check(&runner);
        assert!(result.passed, "{:?}", result);
    }

    #[test]
    fn fail_when_target_missing() {
        let runner = MockRunner::new().with_response(
            "rustup",
            &["target", "list", "--installed"],
            CommandOutput::success("x86_64-unknown-linux-gnu\n"),
        );
        let result = check(&runner);
        assert!(!result.passed);
        assert_eq!(result.fix.as_deref(), Some("rustup target add wasm32v1-none"));
    }

    #[test]
    fn fail_when_rustup_missing() {
        let runner = MockRunner::new().with_response(
            "rustup",
            &["target", "list", "--installed"],
            CommandOutput::missing(),
        );
        let result = check(&runner);
        assert!(!result.passed, "{result:?}");
        assert!(result.detail.contains("rustup not found"), "{result:?}");
        assert!(
            result
                .fix
                .as_deref()
                .unwrap()
                .contains("rustup target add wasm32v1-none"),
            "{result:?}"
        );
    }

    #[test]
    fn exact_match_only_does_not_pass_wasm32_unknown() {
        // A naive substring search would pass `wasm32-unknown-unknown` as
        // "wasm32v1-none"; confirm we require an exact line match.
        let runner = MockRunner::new().with_response(
            "rustup",
            &["target", "list", "--installed"],
            CommandOutput::success("wasm32-unknown-unknown\n"),
        );
        let result = check(&runner);
        assert!(!result.passed);
    }
}
