//! Stellar CLI check.
//!
//! Runs `stellar --version`, parses the first numeric token after
//! `stellar`, and compares against a known-good minimum. The fix message
//! points at the official install page rather than a single shell command
//! because the installation method depends on the user's OS and packager
//! choices (Homebrew, apt, Windows installer, etc.).

use crate::doctor::{
    parse_stellar_version_line, version_at_least, runner::CommandRunner, CheckResult,
};

pub fn check(runner: &dyn CommandRunner, min_version: &str) -> CheckResult {
    let out = runner.run("stellar", &["--version"]);
    if out.status == -1 {
        return CheckResult::fail(
            "stellar CLI",
            "stellar CLI not found on PATH",
            "Install from https://developers.stellar.org/docs/tools/cli/install (minimum version: ".to_string() + min_version + ")",
        );
    }
    if out.status != 0 {
        return CheckResult::fail(
            "stellar CLI",
            format!("`stellar --version` exited with status {}", out.status),
            "Reinstall from https://developers.stellar.org/docs/tools/cli/install",
        );
    }
    let min = match crate::doctor::parse_dotted_version(min_version) {
        Some(v) => v,
        None => {
            return CheckResult::fail(
                "stellar CLI",
                format!("could not parse minimum version `{min_version}`"),
                format!("pass --stellar-min with a valid semver like \"{min_version}\""),
            )
        }
    };
    let actual = match parse_stellar_version_line(&out.stdout) {
        Some(v) => v,
        None => {
            return CheckResult::fail(
                "stellar CLI",
                format!("could not parse stellar output: {:?}", out.stdout),
                "Upgrade: https://developers.stellar.org/docs/tools/cli/install",
            )
        }
    };
    if version_at_least(actual, min) {
        CheckResult::ok(
            "stellar CLI",
            format!(
                "stellar {}.{}.{} (meets {}.{}.{})",
                actual[0], actual[1], actual[2], min[0], min[1], min[2]
            ),
        )
    } else {
        CheckResult::fail(
            "stellar CLI",
            format!(
                "stellar {}.{}.{} is below the required {}.{}.{}",
                actual[0], actual[1], actual[2], min[0], min[1], min[2]
            ),
            "Upgrade from https://developers.stellar.org/docs/tools/cli/install",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::runner::{CommandOutput, MockRunner};

    #[test]
    fn pass_when_stellar_meets_minimum() {
        let runner = MockRunner::new().with_response(
            "stellar",
            &["--version"],
            CommandOutput::success("stellar 23.1.0 (release)\n"),
        );
        let result = check(&runner, "21.0.0");
        assert!(result.passed, "{:?}", result);
        assert!(result.detail.contains("meets"));
    }

    #[test]
    fn fail_when_stellar_too_old() {
        let runner = MockRunner::new().with_response(
            "stellar",
            &["--version"],
            CommandOutput::success("stellar 20.5.0 (release)\n"),
        );
        let result = check(&runner, "21.0.0");
        assert!(!result.passed);
        assert!(result.detail.contains("below"));
        assert!(
            result.fix.as_deref().unwrap().contains("developers.stellar.org"),
            "{:?}",
            result
        );
    }

    #[test]
    fn fail_when_stellar_missing() {
        let runner = MockRunner::new().with_response(
            "stellar",
            &["--version"],
            CommandOutput::missing(),
        );
        let result = check(&runner, "21.0.0");
        assert!(!result.passed, "{result:?}");
        assert!(result.detail.contains("not found on PATH"), "{result:?}");
        let fix = result.fix.as_deref().unwrap();
        assert!(fix.contains("developers.stellar.org"), "{fix}");
        assert!(fix.contains("21.0.0"), "{fix}");
    }

    #[test]
    fn fail_when_output_unparseable() {
        let runner = MockRunner::new().with_response(
            "stellar",
            &["--version"],
            CommandOutput::success("stellar (unknown)\n"),
        );
        let result = check(&runner, "21.0.0");
        assert!(!result.passed);
        assert!(result.detail.contains("could not parse"));
    }
}
