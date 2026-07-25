//! [`CommandRunner`] abstraction and its production + test impls.
//!
//! Tests inject a [`MockRunner`] that returns canned [`CommandOutput`]s for
//! fixed `(program, args)` pairs. `cougr doctor` itself only ever calls into
//! this trait, never `std::process::Command::new` directly, so the test suite
//! is fully deterministic and can run in any environment regardless of which
//! toolchains are actually installed on the machine.

use std::collections::HashMap;

/// One captured or canned subprocess invocation outcome.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandOutput {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

impl CommandOutput {
    pub fn success(stdout: impl Into<String>) -> Self {
        Self {
            status: 0,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    pub fn failure(status: i32, stderr: impl Into<String>) -> Self {
        Self {
            status,
            stdout: String::new(),
            stderr: stderr.into(),
        }
    }

    pub fn missing() -> Self {
        Self {
            status: -1,
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}

/// Trait every doctor check uses to spawn a subprocess. Production code uses
/// [`SystemRunner`]; tests use [`MockRunner`].
pub trait CommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> CommandOutput;
}

/// Production implementation: spawns a real subprocess and captures its
/// stdout/stderr/exit status. A `status` of `-1` means the program could not
/// be started at all (e.g. not on `PATH`).
pub struct SystemRunner;

impl CommandRunner for SystemRunner {
    fn run(&self, program: &str, args: &[&str]) -> CommandOutput {
        match std::process::Command::new(program).args(args).output() {
            Ok(out) => CommandOutput {
                status: out.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            },
            Err(_) => CommandOutput::missing(),
        }
    }
}

/// Test helper that returns canned outputs keyed by `(program, args)`. The
/// lookup uses an internal hash; build keys with [`key_for`] when you need to
/// peek at them from outside.
#[derive(Default, Debug, Clone)]
pub struct MockRunner {
    responses: HashMap<String, CommandOutput>,
}

impl MockRunner {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }

    pub fn with_response(mut self, program: &str, args: &[&str], output: CommandOutput) -> Self {
        self.responses.insert(key_for(program, args), output);
        self
    }
}

impl CommandRunner for MockRunner {
    fn run(&self, program: &str, args: &[&str]) -> CommandOutput {
        self.responses
            .get(&key_for(program, args))
            .cloned()
            .unwrap_or_else(CommandOutput::missing)
    }
}

fn key_for(program: &str, args: &[&str]) -> String {
    let mut s = String::with_capacity(
        program.len() + args.iter().map(|a| a.len() + 1).sum::<usize>(),
    );
    s.push_str(program);
    for a in args {
        s.push('\0');
        s.push_str(a);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_runner_returns_canned_output() {
        let runner = MockRunner::new().with_response(
            "rustc",
            &["--version"],
            CommandOutput::success("rustc 1.85.0 (abc 2025)"),
        );
        let out = runner.run("rustc", &["--version"]);
        assert_eq!(out.status, 0);
        assert!(out.stdout.contains("1.85.0"));
    }

    #[test]
    fn mock_runner_missing_returns_status_minus_one_for_unmocked_call() {
        // MockRunner mirrors SystemRunner's "binary missing" semantics so
        // check code that treats status == -1 as "not found on PATH" works
        // identically under either runner.
        let runner = MockRunner::new();
        let out = runner.run("nonexistent-binary", &["--version"]);
        assert_eq!(out.status, -1);
        assert!(out.stdout.is_empty());
        assert!(out.stderr.is_empty());
    }

    #[test]
    fn keys_differentiate_args() {
        let a = key_for("rustup", &["target", "list", "--installed"]);
        let b = key_for("rustup", &["target", "list"]);
        assert_ne!(a, b);
    }
}
