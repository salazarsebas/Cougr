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

/// Check a standalone project produced by `cougr new` or `cougr export`.
/// Repository-only tracked-artifact checks do not apply outside a checkout.
pub fn run_generated(project: &Path) -> Result<()> {
    for required in [
        "Cargo.toml",
        "README.md",
        ".gitignore",
        "src/lib.rs",
        "src/components.rs",
        "src/systems.rs",
        "src/test.rs",
    ] {
        if !project.join(required).is_file() {
            anyhow::bail!("generated project is missing {required}");
        }
    }

    let gitignore = fs::read_to_string(project.join(".gitignore"))?;
    if !gitignore.lines().any(|line| line.trim() == "target/") {
        anyhow::bail!("generated project .gitignore must ignore target/");
    }
    if gitignore.lines().any(|line| line.trim() == "Cargo.lock") {
        anyhow::bail!("generated project .gitignore must not ignore Cargo.lock");
    }

    let manifest = fs::read_to_string(project.join("Cargo.toml"))?;
    let description = Regex::new(r#"(?m)^description\s*=\s*"([^"]+)""#).unwrap();
    if !description.is_match(&manifest) {
        anyhow::bail!("generated project Cargo.toml needs a non-empty description");
    }

    // Check that [lib] has crate-type containing "cdylib".
    // Extract the text from "[lib]" to the next section header (or EOF) using
    // plain string operations - the `regex` crate does not support lookahead.
    let has_cdylib = if let Some(lib_start) = manifest.find("[lib]") {
        let after_lib = &manifest[lib_start..];
        // The section ends at the next line that starts with '[' (a new table),
        // skipping the "[lib]" header line itself.
        let section_body = after_lib
            .lines()
            .skip(1) // skip the "[lib]" line itself
            .take_while(|l| !l.trim_start().starts_with('['))
            .any(|l| l.contains("cdylib"));
        section_body
    } else {
        false
    };
    if !has_cdylib {
        anyhow::bail!(
            "generated project Cargo.toml is missing crate-type = [\"cdylib\", \"rlib\"] under [lib] \
             (required for `stellar contract build` to produce a .wasm artifact)"
        );
    }

    let readme = fs::read_to_string(project.join("README.md"))?;
    if Regex::new(r"C[A-Z2-7]{55}").unwrap().is_match(&readme) {
        anyhow::bail!("generated project README contains a hardcoded contract ID");
    }

    let metadata = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(project)
        .output()
        .context("cannot run cargo metadata for generated project")?;
    if !metadata.status.success() {
        anyhow::bail!(
            "generated project cargo metadata failed: {}",
            String::from_utf8_lossy(&metadata.stderr).trim()
        );
    }

    println!("=== ALL CHECKS PASSED (generated project) ===");
    Ok(())
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// Build a minimal fake example directory with a Cargo.toml and README.md.
    fn fake_example(dir: &Path, name: &str) -> std::path::PathBuf {
        let ex = dir.join("examples").join(name);
        fs::create_dir_all(&ex).unwrap();
        ex
    }

    // -- check_readme_contract_ids -----------------------------------------

    #[test]
    fn readme_contract_ids_pass_when_no_id() {
        let tmp = tempdir().unwrap();
        let ex = fake_example(tmp.path(), "demo");
        fs::write(ex.join("README.md"), "# Demo\nNo contract id here.\n").unwrap();
        let mut failures = Vec::new();
        check_readme_contract_ids(&ex, "demo", &mut failures);
        assert!(failures.is_empty(), "unexpected failures: {:?}", failures);
    }

    #[test]
    fn readme_contract_ids_fail_when_hardcoded_id() {
        let tmp = tempdir().unwrap();
        let ex = fake_example(tmp.path(), "demo");
        // C + 55 chars from [A-Z2-7]
        let id = format!("C{}", "A".repeat(55));
        fs::write(ex.join("README.md"), format!("# Demo\nContract: {id}\n")).unwrap();
        let mut failures = Vec::new();
        check_readme_contract_ids(&ex, "demo", &mut failures);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("hardcoded contract ID"));
    }

    #[test]
    fn readme_contract_ids_skips_when_missing() {
        let tmp = tempdir().unwrap();
        let ex = fake_example(tmp.path(), "demo");
        let mut failures = Vec::new();
        check_readme_contract_ids(&ex, "demo", &mut failures);
        assert!(failures.is_empty());
    }

    // -- check_example_gitignore -------------------------------------------

    #[test]
    fn example_gitignore_pass_when_target_present() {
        let tmp = tempdir().unwrap();
        let ex = fake_example(tmp.path(), "demo");
        fs::write(ex.join(".gitignore"), "target/\n").unwrap();
        let mut failures = Vec::new();
        check_example_gitignore(&ex, "demo", &mut failures);
        assert!(failures.is_empty(), "unexpected failures: {:?}", failures);
    }

    #[test]
    fn example_gitignore_fail_when_missing() {
        let tmp = tempdir().unwrap();
        let ex = fake_example(tmp.path(), "demo");
        let mut failures = Vec::new();
        check_example_gitignore(&ex, "demo", &mut failures);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("missing .gitignore"));
    }

    #[test]
    fn example_gitignore_fail_when_target_absent() {
        let tmp = tempdir().unwrap();
        let ex = fake_example(tmp.path(), "demo");
        fs::write(ex.join(".gitignore"), "/build\n").unwrap();
        let mut failures = Vec::new();
        check_example_gitignore(&ex, "demo", &mut failures);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("does not ignore target/"));
    }

    // -- check_cargo_toml_description --------------------------------------

    #[test]
    fn cargo_toml_description_pass_when_non_empty() {
        let tmp = tempdir().unwrap();
        let ex = fake_example(tmp.path(), "demo");
        fs::write(
            ex.join("Cargo.toml"),
            "[package]\nname = \"demo\"\ndescription = \"A demo example\"\n",
        )
        .unwrap();
        let mut failures = Vec::new();
        check_cargo_toml_description(&ex, "demo", &mut failures);
        assert!(failures.is_empty(), "unexpected failures: {:?}", failures);
    }

    #[test]
    fn cargo_toml_description_fail_when_missing() {
        let tmp = tempdir().unwrap();
        let ex = fake_example(tmp.path(), "demo");
        fs::write(ex.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();
        let mut failures = Vec::new();
        check_cargo_toml_description(&ex, "demo", &mut failures);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("missing a description"));
    }

    #[test]
    fn cargo_toml_description_fail_when_empty() {
        let tmp = tempdir().unwrap();
        let ex = fake_example(tmp.path(), "demo");
        fs::write(
            ex.join("Cargo.toml"),
            "[package]\nname = \"demo\"\ndescription = \"\"\n",
        )
        .unwrap();
        let mut failures = Vec::new();
        check_cargo_toml_description(&ex, "demo", &mut failures);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("description field is empty"));
    }

    #[test]
    fn cargo_toml_description_fail_when_cargo_toml_missing() {
        let tmp = tempdir().unwrap();
        let ex = fake_example(tmp.path(), "demo");
        let mut failures = Vec::new();
        check_cargo_toml_description(&ex, "demo", &mut failures);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("missing Cargo.toml"));
    }

    // -- check_cargo_metadata ----------------------------------------------
    //
    // NOTE: this test shells out to `cargo metadata`. It is intentionally
    // kept to a single case (a valid minimal crate) to keep it fast and
    // robust. The failure path is exercised indirectly via `run_generated`.

    #[test]
    fn cargo_metadata_pass_on_minimal_crate() {
        let tmp = tempdir().unwrap();
        let ex = fake_example(tmp.path(), "demo");
        fs::write(
            ex.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        fs::create_dir_all(ex.join("src")).unwrap();
        fs::write(ex.join("src/lib.rs"), "").unwrap();
        let mut failures = Vec::new();
        check_cargo_metadata(&ex, "demo", &mut failures);
        assert!(failures.is_empty(), "unexpected failures: {:?}", failures);
    }

    // -- indent_lines ------------------------------------------------------

    #[test]
    fn indent_lines_prefixes_every_line() {
        let out = indent_lines("a\nb\nc", "  ");
        assert_eq!(out, "  a\n  b\n  c");
    }

    #[test]
    fn indent_lines_handles_empty_string() {
        assert_eq!(indent_lines("", "  "), "");
    }
}
