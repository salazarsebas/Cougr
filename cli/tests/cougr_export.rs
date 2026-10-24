//! End-to-end coverage for the studio config export path.

use std::fs;
use std::process::{Command, Output};

fn export(config: &str) -> (tempfile::TempDir, Output) {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.json");
    fs::write(&config_path, config).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_cougr"))
        .args(["export", "demo", "--config"])
        .arg(&config_path)
        .arg("--path")
        .arg(dir.path())
        .output()
        .unwrap();
    (dir, output)
}

#[derive(Debug)]
struct FakeRepo {
    _dir: tempfile::TempDir,
    root: std::path::PathBuf,
}

fn fake_repo() -> FakeRepo {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "fake-repo"
version = "0.1.0"
edition = "2021"

[workspace]
members = []
"#,
    )
    .unwrap();
    fs::create_dir_all(root.join("examples")).unwrap();
    FakeRepo { _dir: dir, root }
}

// A valid example that passes every hygiene rule.
fn write_valid_example(repo: &FakeRepo, name: &str) {
    let example = repo.root.join("examples").join(name);
    fs::create_dir_all(example.join("src")).unwrap();
    fs::write(
        example.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"
description = "A Cougr example for {}"

[dependencies]
cougr = {{ path = "../.." }}
"#,
            name
        ),
    )
    .unwrap();
    fs::write(example.join("src/lib.rs"), "").unwrap();
    fs::write(
        example.join("README.md"),
        format!("# Cougr Example: {}", name),
    )
    .unwrap();
    fs::write(example.join(".gitignore"), "/target\n").unwrap();
}

#[test]
fn default_export_matches_new_template() {
    let (dir, output) = export(r#"{"board_width":3,"board_height":3,"win_length":3}"#);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let project = dir.path().join("demo");
    for file in [
        "Cargo.toml",
        "README.md",
        "src/lib.rs",
        "src/components.rs",
        "src/systems.rs",
        "src/test.rs",
    ] {
        assert!(project.join(file).is_file(), "missing {file}");
    }
    let components = fs::read_to_string(project.join("src/components.rs")).unwrap();
    assert!(components.contains("BOARD_WIDTH: u32 = 3"));
    assert!(components.contains("BOARD_HEIGHT: u32 = 3"));
    assert!(components.contains("WIN_LENGTH: u32 = 3"));
    let readme = fs::read_to_string(project.join("README.md")).unwrap();
    assert!(readme.contains("cargo test"));
    assert!(readme.contains("stellar contract build"));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected_cd = if project.display().to_string().contains(' ') {
        format!("cd \"{}\"", project.display())
    } else {
        format!("cd {}", project.display())
    };
    assert!(stdout.contains(&expected_cd));
    assert!(stdout.contains("cargo test"));
    assert!(stdout.contains("stellar contract build"));
    let check = Command::new(env!("CARGO_BIN_EXE_cougr"))
        .arg("check")
        .arg("--path")
        .arg(&project)
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
}

#[test]
fn non_default_export_applies_config() {
    let (dir, output) = export(r#"{"board_width":5,"board_height":4,"win_length":4}"#);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let project = dir.path().join("demo");
    let components = fs::read_to_string(project.join("src/components.rs")).unwrap();
    for expected in [
        "BOARD_WIDTH: u32 = 5",
        "BOARD_HEIGHT: u32 = 4",
        "WIN_LENGTH: u32 = 4",
    ] {
        assert!(components.contains(expected), "missing {expected}");
    }
    assert!(fs::read_to_string(project.join("README.md"))
        .unwrap()
        .contains("5×4"));
    let check = Command::new(env!("CARGO_BIN_EXE_cougr"))
        .arg("check")
        .arg("--path")
        .arg(&project)
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
}

#[test]
fn invalid_config_creates_no_project() {
    for config in [
        r#"{"board_width":2,"board_height":3,"win_length":3}"#,
        r#"{"board_width":4,"board_height":3,"win_length":4}"#,
        "{}",
    ] {
        let (dir, output) = export(config);
        assert!(!output.status.success());
        assert!(!dir.path().join("demo").exists());
    }
}

#[test]
fn export_never_overwrites_existing_directory() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("demo");
    fs::create_dir(&project).unwrap();
    fs::write(project.join("keep.txt"), "mine").unwrap();
    let config_path = dir.path().join("config.json");
    fs::write(
        &config_path,
        r#"{"board_width":3,"board_height":3,"win_length":3}"#,
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_cougr"))
        .args(["export", "demo", "--config"])
        .arg(&config_path)
        .arg("--path")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(
        fs::read_to_string(project.join("keep.txt")).unwrap(),
        "mine"
    );
}

#[test]
fn export_missing_config_file_fails() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("no-such-config.json");
    let output = Command::new(env!("CARGO_BIN_EXE_cougr"))
        .args(["export", "demo", "--config"])
        .arg(&missing)
        .arg("--path")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(!dir.path().join("demo").exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no-such-config.json"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn export_malformed_json_fails() {
    let (dir, output) = export("{\"board_width\": 3,");
    assert!(!output.status.success());
    assert!(!dir.path().join("demo").exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("json"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn export_rejects_out_of_bounds_config() {
    let (dir, output) = export(r#"{"board_width":9,"board_height":3,"win_length":3}"#);
    assert!(!output.status.success());
    assert!(!dir.path().join("demo").exists());
}

#[test]
fn add_unknown_piece_fails() {
    let repo = fake_repo();
    write_valid_example(&repo, "demo");
    let output = Command::new(env!("CARGO_BIN_EXE_cougr"))
        .arg("add")
        .arg("not-a-real-piece")
        .arg("--path")
        .arg(repo.root.join("examples").join("demo"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not-a-real-piece"),
        "unexpected stderr: {stderr}"
    );
    assert!(
        !stderr.contains("package not found"),
        "unknown piece should not report a missing package: {stderr}"
    );
}
