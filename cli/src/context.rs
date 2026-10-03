//! Shared context resolution for `cougr check` and `cougr check --verified`.
//!
//! Determines the repository root and which examples to check, supporting
//! auto-detection from cwd, explicit `--path`, and `--example` flags.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Metadata for a single example discovered under `examples/`.
#[derive(Clone, Debug)]
pub struct Example {
    pub name: String,
}

/// The resolved context for a check run.
pub struct CheckContext {
    pub repo_root: PathBuf,
    pub examples: Vec<Example>,
}

// ---------------------------------------------------------------------------
// Resolution
// ---------------------------------------------------------------------------

/// Determine the repo root and which examples to check.
pub fn resolve(
    cwd: &Path,
    explicit_root: Option<&str>,
    single_example: Option<&str>,
) -> Result<CheckContext> {
    // 1. Determine repo root
    let repo_root = if let Some(r) = explicit_root {
        let p = PathBuf::from(r);
        p.canonicalize()
            .context(format!("explicit --path does not exist: {}", r))?
    } else {
        find_repo_root(cwd)?
    };

    // 2. Determine examples to check
    let examples = if let Some(name) = single_example {
        let example_dir = repo_root.join("examples").join(name);
        if !example_dir.is_dir() {
            anyhow::bail!("example '{}' not found at {}", name, example_dir.display());
        }
        vec![Example {
            name: name.to_string(),
        }]
    } else if explicit_root.is_none() && is_inside_example_dir(cwd) {
        // Auto-detect: if cwd is inside examples/<name>/, just check that one.
        // Only auto-detect when no explicit --path was given.
        let name = cwd
            .file_name()
            .and_then(|n| n.to_str())
            .map(String::from)
            .context("cannot determine example name from current directory")?;
        vec![Example { name }]
    } else {
        discover_examples(&repo_root)?
    };

    Ok(CheckContext {
        repo_root,
        examples,
    })
}

/// Return the absolute path to an example's directory.
pub fn example_dir(repo_root: &Path, name: &str) -> PathBuf {
    repo_root.join("examples").join(name)
}

/// Return the 10 currently-canonical example names per EXAMPLE_STANDARD.md §7.
pub fn canonical_example_names() -> &'static [&'static str] {
    &[
        "spawn_and_move",
        "tic_tac_toe",
        "session_arena",
        "hidden_hand",
        "fog_explorer",
        "dice_duel",
        "blind_auction",
        "snake",
        "battleship",
        "guild_arena",
    ]
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Walk upward from `cwd` looking for a directory that contains both
/// `Cargo.toml` and an `examples/` subdirectory (the repo root).
fn find_repo_root(cwd: &Path) -> Result<PathBuf> {
    let mut current = cwd.to_path_buf();
    loop {
        if current.join("Cargo.toml").is_file() && current.join("examples").is_dir() {
            return Ok(current);
        }
        if !current.pop() {
            anyhow::bail!(
                "could not find repo root (no Cargo.toml + examples/ found above {:?}). \
                 Use --path to specify the repo root explicitly.",
                cwd
            );
        }
    }
}

/// True when `cwd` is inside an `examples/<name>/` directory.
fn is_inside_example_dir(cwd: &Path) -> bool {
    if let Some(parent) = cwd.parent() {
        parent.file_name().map(|n| n == "examples").unwrap_or(false)
    } else {
        false
    }
}

/// Discover all example directories under `examples/` that contain a Cargo.toml.
fn discover_examples(repo_root: &Path) -> Result<Vec<Example>> {
    let examples_dir = repo_root.join("examples");
    let mut examples = Vec::new();

    if !examples_dir.is_dir() {
        anyhow::bail!(
            "examples/ directory not found at {}",
            examples_dir.display()
        );
    }

    for entry in fs::read_dir(&examples_dir).context("cannot read examples/ directory")? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let dir_name = entry.file_name();
            if let Some(name) = dir_name.to_str() {
                if entry.path().join("Cargo.toml").is_file() {
                    examples.push(Example {
                        name: name.to_string(),
                    });
                }
            }
        }
    }

    if examples.is_empty() {
        anyhow::bail!("no examples found under {}", examples_dir.display());
    }

    Ok(examples)
}
//! Hygiene check logic for `cougr check`.
//!
//! Mirrors the checks in `scripts/verify_hygiene.sh` and
//! `scripts/enforce_hygiene.sh`, ported to Rust for cross-platform
//! operation with no external script dependencies (shells out only for
//! `git ls-files` and `cargo metadata`, which are required regardless).

use crate::context::{example_dir, CheckContext};
use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::Path;
use std::process::{exit, Command};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run hygiene checks.
pub fn run(ctx: &CheckContext) -> Result<()> {
    println!("=== Cougr Hygiene Check ===");
    println!("Repository root : {}", ctx.repo_root.display());
    if ctx.examples.len() == 1 {
        println!("Checking example: {}", ctx.examples[0].name);
    } else {
        println!("Checking        : {} examples", ctx.examples.len());
    }
    println!();

    let mut failures: Vec<String> = Vec::new();

    // Root-level checks always run - tracked artifacts and root .gitignore
    // issues would fail CI regardless of which example is being checked.

    // Check 1 - root .gitignore must NOT ignore Cargo.lock
    check_root_gitignore_cargo_lock(&ctx.repo_root, &mut failures);

    // Check 2 & 3 - no tracked build artifacts (git ls-files)
    check_tracked_artifacts(&ctx.repo_root, &mut failures);

    // Per-example checks
    for ex in &ctx.examples {
        let dir = example_dir(&ctx.repo_root, &ex.name);

        // Check 4 - no hardcoded contract IDs in README
        check_readme_contract_ids(&dir, &ex.name, &mut failures);

        // Check 5 - .gitignore exists and has target/
        check_example_gitignore(&dir, &ex.name, &mut failures);

        // Check 6 - .gitignore must NOT ignore Cargo.lock
        check_example_gitignore_cargo_lock(&dir, &ex.name, &mut failures);

        // Check 7 - Cargo.toml has description
        check_cargo_toml_description(&dir, &ex.name, &mut failures);

        // Check 8 - cargo metadata --no-deps
        check_cargo_metadata(&dir, &ex.name, &mut failures);
    }

    // Report
    if failures.is_empty() {
        println!("=== ALL CHECKS PASSED ===");
        Ok(())
    } else {
        eprintln!();
        eprintln!("=== {} CHECK(S) FAILED ===", failures.len());
        for f in &failures {
            eprintln!("  FAIL: {}", f);
        }
        exit(1);
    }
}

// ---------------------------------------------------------------------------
// Individual checks
// ---------------------------------------------------------------------------

/// Check 1: root `.gitignore` must NOT ignore `Cargo.lock`.
fn check_root_gitignore_cargo_lock(repo_root: &Path, failures: &mut Vec<String>) {
    let gitignore = repo_root.join(".gitignore");
    match fs::read_to_string(&gitignore) {
        Ok(contents) => {
            let re = Regex::new(r"(?m)^Cargo\.lock$").unwrap();
            if re.is_match(&contents) {
                failures.push(
                    "root .gitignore must not ignore Cargo.lock (examples are applications)"
                        .to_string(),
                );
            }
        }
        Err(e) => {
            failures.push(format!("cannot read root .gitignore: {}", e));
        }
    }
}

/// Checks 2 & 3: no tracked `target/` directories or `.wasm` files in examples/.
fn check_tracked_artifacts(repo_root: &Path, failures: &mut Vec<String>) {
    for (pattern, label) in &[
        ("examples/**/target/**", "target/"),
        ("examples/**/*.wasm", ".wasm"),
    ] {
        match run_git_ls_files(repo_root, pattern) {
            Ok(output) => {
                let trimmed = output.trim();
                if !trimmed.is_empty() {
                    failures.push(format!(
                        "tracked {} artifacts found:\n{}",
                        label,
                        indent_lines(trimmed, "    ")
                    ));
                }
            }
            Err(e) => {
                failures.push(format!(
                    "failed to check for tracked {} artifacts: {}",
                    label, e
                ));
            }
        }
    }
}

/// Check 4: no hardcoded contract IDs (`C[A-Z2-7]{55}`) in example READMEs.
pub fn check_readme_contract_ids(example_dir: &Path, name: &str, failures: &mut Vec<String>) {
    let readme = example_dir.join("README.md");
    if !readme.is_file() {
        return;
    }

    match fs::read_to_string(&readme) {
        Ok(contents) => {
            let re = Regex::new(r"C[A-Z2-7]{55}").unwrap();
            if re.is_match(&contents) {
                failures.push(format!(
                    "hardcoded contract ID(s) in examples/{}/README.md",
                    name
                ));
            }
        }
        Err(e) => {
            failures.push(format!("cannot read examples/{}/README.md: {}", name, e));
        }
    }
}

/// Check 5: each example must have a `.gitignore` containing `target/`.
pub fn check_example_gitignore(example_dir: &Path, name: &str, failures: &mut Vec<String>) {
    let gitignore = example_dir.join(".gitignore");
    if !gitignore.is_file() {
        failures.push(format!("examples/{}: missing .gitignore", name));
        return;
    }

    match fs::read_to_string(&gitignore) {
        Ok(contents) => {
            let re = Regex::new(r"(?m)^target/").unwrap();
            if !re.is_match(&contents) {
                failures.push(format!(
                    "examples/{}: .gitignore does not ignore target/",
                    name
                ));
            }
        }
        Err(e) => {
            failures.push(format!("examples/{}: cannot read .gitignore: {}", name, e));
        }
    }
}

/// Check 6: example `.gitignore` must NOT ignore `Cargo.lock`.
fn check_example_gitignore_cargo_lock(example_dir: &Path, name: &str, failures: &mut Vec<String>) {
    let gitignore = example_dir.join(".gitignore");
    if !gitignore.is_file() {
        return; // already flagged by check_example_gitignore
    }

    match fs::read_to_string(&gitignore) {
        Ok(contents) => {
            let re = Regex::new(r"(?m)^Cargo\.lock$").unwrap();
            if re.is_match(&contents) {
                failures.push(format!(
                    "examples/{}: .gitignore must not ignore Cargo.lock (examples are applications)",
                    name,
                ));
            }
        }
        Err(e) => {
            failures.push(format!("examples/{}: cannot read .gitignore: {}", name, e));
        }
    }
}

/// Check 7: `Cargo.toml` must have a non-empty `description` field.
pub fn check_cargo_toml_description(example_dir: &Path, name: &str, failures: &mut Vec<String>) {
    let cargo_toml = example_dir.join("Cargo.toml");
    if !cargo_toml.is_file() {
        failures.push(format!("examples/{}: missing Cargo.toml", name));
        return;
    }

    match fs::read_to_string(&cargo_toml) {
        Ok(contents) => {
            let re = Regex::new(r#"(?m)^description\s*=\s*"([^"]*)""#).unwrap();
            if let Some(caps) = re.captures(&contents) {
                let desc = &caps[1];
                if desc.trim().is_empty() {
                    failures.push(format!(
                        "examples/{}: Cargo.toml description field is empty",
                        name
                    ));
                }
            } else {
                failures.push(format!(
                    "examples/{}: Cargo.toml is missing a description field",
                    name
                ));
            }
        }
        Err(e) => {
            failures.push(format!("examples/{}: cannot read Cargo.toml: {}", name, e));
        }
    }
}

/// Check 8: `cargo metadata --no-deps` must succeed.
pub fn check_cargo_metadata(example_dir: &Path, name: &str, failures: &mut Vec<String>) {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(example_dir)
        .output();

    match output {
        Ok(out) => {
            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                failures.push(format!(
                    "examples/{}: cargo metadata --no-deps FAILED:\n{}",
                    name,
                    indent_lines(stderr.trim(), "    ")
                ));
            }
        }
        Err(e) => {
            failures.push(format!(
                "examples/{}: cannot run cargo metadata: {}",
                name, e
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run `git ls-files <pattern>` from `cwd` and return stdout.
pub fn run_git_ls_files(cwd: &Path, pattern: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["ls-files", pattern])
        .current_dir(cwd)
        .output()
        .context("failed to run git ls-files")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git ls-files failed: {}", stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Indent every line by `prefix`.
pub fn indent_lines(s: &str, prefix: &str) -> String {
    s.lines()
        .map(|l| format!("{}{}", prefix, l))
        .collect::<Vec<_>>()
        .join("\n")
}
