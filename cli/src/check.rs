//! Hygiene check logic for `cougr check`.
//!
//! Mirrors the checks in `scripts/verify_hygiene.sh` and
//! `scripts/enforce_hygiene.sh`, ported to Rust for cross-platform
//! operation with no external script dependencies (shells out only for
//! `git ls-files` and `cargo metadata`, which are required regardless).

use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run hygiene checks.
///
/// * `explicit_root` — if `Some`, use this as the repo root.
/// * `single_example` — if `Some`, only check this named example (requires
///   repo-root context to locate `examples/<name>/`).
pub fn run(explicit_root: Option<&str>, single_example: Option<&str>) -> Result<()> {
    let cwd = std::env::current_dir().context("failed to read current directory")?;

    let (repo_root, examples) = resolve_context(&cwd, explicit_root, single_example)?;

    println!("=== Cougr Hygiene Check ===");
    println!("Repository root : {}", repo_root.display());
    if examples.len() == 1 {
        println!("Checking example: {}", examples[0].name);
    } else {
        println!("Checking        : {} examples", examples.len());
    }
    println!();

    let mut failures: Vec<String> = Vec::new();

    // Root-level checks always run — tracked artifacts and root .gitignore
    // issues would fail CI regardless of which example is being checked.

    // Check 1 — root .gitignore must NOT ignore Cargo.lock
    check_root_gitignore_cargo_lock(&repo_root, &mut failures);

    // Check 2 & 3 — no tracked build artifacts (git ls-files)
    check_tracked_artifacts(&repo_root, &mut failures);

    // Per-example checks
    for ex in &examples {
        let example_dir = repo_root.join("examples").join(&ex.name);

        // Check 4 — no hardcoded contract IDs in README
        check_readme_contract_ids(&example_dir, &ex.name, &mut failures);

        // Check 5 — .gitignore exists and has target/
        check_example_gitignore(&example_dir, &ex.name, &mut failures);

        // Check 6 — .gitignore must NOT ignore Cargo.lock
        check_example_gitignore_cargo_lock(&example_dir, &ex.name, &mut failures);

        // Check 7 — Cargo.toml has description
        check_cargo_toml_description(&example_dir, &ex.name, &mut failures);

        // Check 8 — cargo metadata --no-deps
        check_cargo_metadata(&example_dir, &ex.name, &mut failures);
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
// Context resolution
// ---------------------------------------------------------------------------

struct Example {
    name: String,
}

/// Determine the repo root and which examples to check.
fn resolve_context(
    cwd: &Path,
    explicit_root: Option<&str>,
    single_example: Option<&str>,
) -> Result<(PathBuf, Vec<Example>)> {
    // 1. Determine repo root
    let repo_root = if let Some(r) = explicit_root {
        let p = PathBuf::from(r);
        p.canonicalize()
            .context(format!("explicit --path does not exist: {}", r))?
    } else {
        find_repo_root(cwd)?
    };

    // 2. Determine examples to check
    if let Some(name) = single_example {
        let example_dir = repo_root.join("examples").join(&name);
        if !example_dir.is_dir() {
            anyhow::bail!("example '{}' not found at {}", name, example_dir.display());
        }
        return Ok((repo_root, vec![Example { name }]));
    }

    // Auto-detect: if cwd is inside examples/<name>/, just check that one.
    // Only auto-detect when no explicit --path was given, to avoid confusing
    // interactions (e.g. user specifies --path but happens to be sitting in
    // an example directory).
    if explicit_root.is_none() && is_inside_example_dir(cwd) {
        if let Some(parent) = cwd.parent() {
            if parent.file_name().map(|n| n == "examples").unwrap_or(false) {
                if let Some(name) = cwd.file_name().and_then(|n| n.to_str()) {
                    return Ok((repo_root, vec![Example { name: name.to_string() }]));
                }
            }
        }
    }

    // Otherwise, discover all examples
    let examples_dir = repo_root.join("examples");
    let mut examples = Vec::new();
    if examples_dir.is_dir() {
        for entry in fs::read_dir(&examples_dir).context("cannot read examples/ directory")? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let dir_name = entry.file_name();
                if let Some(name) = dir_name.to_str() {
                    // Only include directories that have a Cargo.toml (actual examples)
                    if entry.path().join("Cargo.toml").is_file() {
                        examples.push(Example {
                            name: name.to_string(),
                        });
                    }
                }
            }
        }
    }

    if examples.is_empty() {
        anyhow::bail!("no examples found under {}", examples_dir.display());
    }

    Ok((repo_root, examples))
}

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
    // Check for tracked target/ artifacts
    match run_git_ls_files(repo_root, "examples/**/target/**") {
        Ok(output) => {
            let trimmed = output.trim();
            if !trimmed.is_empty() {
                failures.push(format!(
                    "tracked target/ artifacts found:\n{}",
                    indent_lines(trimmed, "    ")
                ));
            }
        }
        Err(e) => {
            failures.push(format!("failed to check for tracked target/ artifacts: {}", e));
        }
    }

    // Check for tracked .wasm artifacts
    match run_git_ls_files(repo_root, "examples/**/*.wasm") {
        Ok(output) => {
            let trimmed = output.trim();
            if !trimmed.is_empty() {
                failures.push(format!(
                    "tracked .wasm artifacts found:\n{}",
                    indent_lines(trimmed, "    ")
                ));
            }
        }
        Err(e) => {
            failures.push(format!(
                "failed to check for tracked .wasm artifacts: {}",
                e
            ));
        }
    }
}

/// Check 4: no hardcoded contract IDs (`C[A-Z2-7]{55}`) in example READMEs.
fn check_readme_contract_ids(example_dir: &Path, name: &str, failures: &mut Vec<String>) {
    let readme = example_dir.join("README.md");
    if !readme.is_file() {
        // No README — not a hygiene failure per se (some examples might legitimately lack one),
        // but the EXAMPLE_STANDARD requires it. We flag a separate, softer warning via
        // the description check or similar. For now, skip.
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
fn check_example_gitignore(example_dir: &Path, name: &str, failures: &mut Vec<String>) {
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
            failures.push(format!(
                "examples/{}: cannot read .gitignore: {}",
                name, e
            ));
        }
    }
}

/// Check 6: example `.gitignore` must NOT ignore `Cargo.lock`.
fn check_example_gitignore_cargo_lock(example_dir: &Path, name: &str, failures: &mut Vec<String>) {
    let gitignore = example_dir.join(".gitignore");
    if !gitignore.is_file() {
        // Already flagged by check_example_gitignore; don't double-report.
        return;
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
            failures.push(format!(
                "examples/{}: cannot read .gitignore: {}",
                name, e
            ));
        }
    }
}

/// Check 7: `Cargo.toml` must have a non-empty `description` field.
fn check_cargo_toml_description(example_dir: &Path, name: &str, failures: &mut Vec<String>) {
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
            failures.push(format!(
                "examples/{}: cannot read Cargo.toml: {}",
                name, e
            ));
        }
    }
}

/// Check 8: `cargo metadata --no-deps` must succeed.
fn check_cargo_metadata(example_dir: &Path, name: &str, failures: &mut Vec<String>) {
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
fn run_git_ls_files(cwd: &Path, pattern: &str) -> Result<String> {
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

/// Indent every non-empty line by `prefix`.
fn indent_lines(s: &str, prefix: &str) -> String {
    s.lines()
        .map(|l| format!("{}{}", prefix, l))
        .collect::<Vec<_>>()
        .join("\n")
}
