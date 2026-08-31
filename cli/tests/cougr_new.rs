//! End-to-end tests that drive the compiled `cougr` binary.
//!
//! The fast tests assert on the generated tree and the CLI's exit codes. The
//! `#[ignore]`d `generated_projects_pass_cargo_test` compiles all four templates
//! against the published `cougr-core` — it is the real definition-of-done check,
//! run in CI and locally with:
//!
//! ```bash
//! cargo test -p cougr-cli -- --ignored --nocapture
//! ```

use std::path::Path;
use std::process::{Command, Output};

const TEMPLATES: [&str; 4] = ["starter", "turn-based", "hidden-info", "session-auth"];

fn cougr(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cougr"))
        .args(args)
        .output()
        .expect("run the cougr binary")
}

fn generate(name: &str, template: &str, parent: &Path) -> Output {
    cougr(&[
        "new",
        name,
        "--template",
        template,
        "--path",
        parent.to_str().expect("utf-8 temp path"),
    ])
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[test]
fn every_template_generates_a_complete_project() {
    for template in TEMPLATES {
        let dir = tempfile::tempdir().unwrap();
        let output = generate("demo", template, dir.path());
        assert!(
            output.status.success(),
            "`--template {template}` failed: {}",
            stderr(&output)
        );

        let project = dir.path().join("demo");
        for expected in [
            "Cargo.toml",
            "README.md",
            ".gitignore",
            "src/lib.rs",
            "src/components.rs",
            "src/systems.rs",
            "src/test.rs",
        ] {
            assert!(
                project.join(expected).is_file(),
                "`{template}` did not generate `{expected}`"
            );
        }
    }
}

#[test]
fn starter_is_the_default_template() {
    let dir = tempfile::tempdir().unwrap();
    let output = cougr(&[
        "new",
        "demo",
        "--path",
        dir.path().to_str().expect("utf-8 temp path"),
    ]);
    assert!(output.status.success(), "{}", stderr(&output));

    let readme = std::fs::read_to_string(dir.path().join("demo/README.md")).unwrap();
    assert!(readme.contains("--template starter"));
}

#[test]
fn success_output_names_the_next_commands() {
    let dir = tempfile::tempdir().unwrap();
    let output = generate("demo", "starter", dir.path());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("cd demo"));
    assert!(stdout.contains("cargo test"));
    assert!(stdout.contains("stellar contract build"));
}

#[test]
fn an_unknown_template_is_a_usage_error() {
    let dir = tempfile::tempdir().unwrap();
    let output = generate("demo", "roguelike", dir.path());

    assert!(!output.status.success());
    let message = stderr(&output);
    assert!(message.contains("roguelike"), "{message}");
    assert!(
        message.contains("starter"),
        "expected the valid values: {message}"
    );
    assert!(!dir.path().join("demo").exists());
}

#[test]
fn an_invalid_name_is_reported_without_a_panic() {
    let dir = tempfile::tempdir().unwrap();
    let output = generate("2fast", "starter", dir.path());

    assert!(!output.status.success());
    let message = stderr(&output);
    assert!(message.starts_with("error:"), "{message}");
    assert!(!message.contains("panicked"), "{message}");
    assert!(message.contains("help:"), "expected a hint: {message}");
}

#[test]
fn an_existing_directory_is_never_overwritten() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("demo");
    std::fs::create_dir(&project).unwrap();
    std::fs::write(project.join("keep.txt"), "mine").unwrap();

    let output = generate("demo", "starter", dir.path());

    assert!(!output.status.success());
    assert!(stderr(&output).contains("already exists"));
    assert_eq!(
        std::fs::read_to_string(project.join("keep.txt")).unwrap(),
        "mine"
    );
    assert!(!project.join("Cargo.toml").exists());
}

#[test]
fn add_list_shows_the_v1_pieces() {
    let output = cougr(&["add", "--list"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let listing = String::from_utf8_lossy(&output.stdout);
    for piece in ["session-auth", "hidden-hand", "standards/pausable"] {
        assert!(listing.contains(piece), "missing {piece}: {listing}");
    }
}

#[test]
fn add_wires_a_piece_and_refuses_to_overwrite_it() {
    let dir = tempfile::tempdir().unwrap();
    let generated = generate("demo", "starter", dir.path());
    assert!(generated.status.success(), "{}", stderr(&generated));
    let project = dir.path().join("demo");

    let added = Command::new(env!("CARGO_BIN_EXE_cougr"))
        .arg("add")
        .arg("session-auth")
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(added.status.success(), "{}", stderr(&added));
    assert!(project.join("src/session_auth.rs").is_file());
    assert!(std::fs::read_to_string(project.join("src/lib.rs"))
        .unwrap()
        .contains("pub mod session_auth;"));

    let second = Command::new(env!("CARGO_BIN_EXE_cougr"))
        .args(["add", "session-auth"])
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(!second.status.success());
    assert!(stderr(&second).contains("src/session_auth.rs"));
}

/// The definition-of-done check: every template builds and its tests pass
/// against the published `cougr-core`.
///
/// Ignored by default because it downloads and compiles the Soroban SDK.
#[test]
#[ignore = "compiles four Soroban projects; run with --ignored"]
fn generated_projects_pass_cargo_test() {
    for template in TEMPLATES {
        let dir = tempfile::tempdir().unwrap();
        let output = generate("demo", template, dir.path());
        assert!(output.status.success(), "{}", stderr(&output));

        let status = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
            .arg("test")
            .current_dir(dir.path().join("demo"))
            .status()
            .expect("run cargo test in the generated project");

        assert!(status.success(), "`{template}` failed `cargo test`");
    }
}
