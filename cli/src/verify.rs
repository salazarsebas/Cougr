//! Canonical-quality verification for the "Cougr Verified" badge.
//!
//! Evaluates an example against every criterion in EXAMPLE_STANDARD.md's
//! canonical-vs-transitional quality checklist (§Quality Checklist).
//!
//! The base hygiene checks from `check.rs` are a subset; this module adds
//! the full canonical-quality assessment required for the verified badge.
//!
//! Part of #258: produces structured pass/fail data that the showcase/
//! example gallery generator consumes via `--output`, CI uploads the
//! resulting `verified.json` as a build artifact.

use crate::check::{
    check_cargo_metadata, check_cargo_toml_description, check_example_gitignore,
    check_readme_contract_ids, indent_lines, run_git_ls_files,
};
use crate::context::{canonical_example_names, example_dir, CheckContext};
use anyhow::{Context, Result};
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process::{exit, Command};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run verified-quality checks and return structured results.
///
/// When `output_path` is provided, writes the JSON report to that file
/// (for consumption by the showcase/gallery generator). The JSON is also
/// printed to stdout when `json` is true or when writing to a file.
pub fn run(
    ctx: &CheckContext,
    json: bool,
    run_build: bool,
    canonical_only: bool,
    output_path: Option<&str>,
) -> Result<()> {
    // Filter to canonical examples if requested
    let target_examples: Vec<&crate::context::Example> = if canonical_only {
        let canonicals = canonical_example_names();
        ctx.examples
            .iter()
            .filter(|ex| canonicals.contains(&ex.name.as_str()))
            .collect()
    } else {
        ctx.examples.iter().collect()
    };

    if target_examples.is_empty() {
        anyhow::bail!("no examples to check (canonical-only filter may exclude all)");
    }

    // Root-level checks run once (global, not per-example)
    let root_criteria = check_root_hygiene(&ctx.repo_root);

    let mut results: Vec<ExampleResult> = Vec::new();

    for ex in &target_examples {
        let dir = example_dir(&ctx.repo_root, &ex.name);
        let mut criteria: Vec<Criterion> = Vec::new();

        // Root-level hygiene (shared across all examples)
        criteria.extend(root_criteria.clone());

        // Per-example hygiene
        check_hygiene(&dir, &ex.name, &mut criteria);

        // Dependencies
        check_dependencies(&dir, &mut criteria);

        // Module structure
        check_module_structure(&dir, &mut criteria);

        // README completeness
        check_readme_sections(&dir, &mut criteria);

        // Test coverage
        check_test_coverage(&dir, &mut criteria);

        // Classification
        check_classification(&dir, &mut criteria);

        // Cargo.lock committed
        check_cargo_lock_committed(&ctx.repo_root, &ex.name, &mut criteria);

        // Build validation (heavy, optional)
        if run_build {
            check_cargo_test(&dir, &mut criteria);
            check_stellar_build(&dir, &mut criteria);
        }

        let all_pass = criteria.iter().all(|c| c.pass);
        results.push(ExampleResult {
            example: ex.name.clone(),
            verified: all_pass,
            criteria,
            unmet: if all_pass {
                vec![]
            } else {
                criteria
                    .iter()
                    .filter(|c| !c.pass)
                    .map(|c| c.id.clone())
                    .collect()
            },
        });
    }

    // Output: serialize once, then decide print vs file vs both
    let json_out: Option<String> = if json || output_path.is_some() {
        Some(serde_json::to_string_pretty(&results).context("failed to serialize JSON output")?)
    } else {
        None
    };

    if let Some(ref out) = json_out {
        println!("{}", out);
    }
    if let Some(path) = output_path {
        if let Some(ref out) = json_out {
            fs::write(path, out)
                .context(format!("failed to write verified badge report to {}", path))?;
            eprintln!("Verified badge report written to {}", path);
        }
    }
    if json_out.is_none() {
        print_human_summary(&results);
    }

    // Exit code
    let all_verified = results.iter().all(|r| r.verified);
    if !all_verified {
        exit(1);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Output types (serializable for JSON)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ExampleResult {
    example: String,
    verified: bool,
    criteria: Vec<Criterion>,
    unmet: Vec<String>,
}

#[derive(Serialize, Clone)]
struct Criterion {
    id: String,
    label: String,
    pass: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

impl Criterion {
    fn new(id: &str, label: &str, pass: bool, detail: Option<String>) -> Self {
        Criterion {
            id: id.to_string(),
            label: label.to_string(),
            pass,
            detail,
        }
    }
}

// ---------------------------------------------------------------------------
// Human-readable output
// ---------------------------------------------------------------------------

fn print_human_summary(results: &[ExampleResult]) {
    println!("=== Cougr Verified Check ===");
    println!();

    for r in results {
        let icon = if r.verified { "✓" } else { "✗" };
        println!("  {}  {}", icon, r.example);

        if r.verified {
            continue;
        }

        for c in &r.criteria {
            if !c.pass {
                println!(
                    "      ✗  {} — {}",
                    c.label,
                    c.detail.as_deref().unwrap_or("")
                );
            }
        }
    }

    println!();
    let passed = results.iter().filter(|r| r.verified).count();
    let total = results.len();
    println!("=== {}/{} VERIFIED ===", passed, total);

    if passed != total {
        println!();
        println!("Unmet criteria per example:");
        for r in results {
            if !r.verified {
                println!("  {}: {}", r.example, r.unmet.join(", "));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Root-level hygiene (tracked artifacts, root .gitignore)
// ---------------------------------------------------------------------------

/// Returns root-level criteria (same for every example — global, not per-example).
fn check_root_hygiene(repo_root: &Path) -> Vec<Criterion> {
    let mut criteria = Vec::new();

    // Tracked target/ artifacts
    match run_git_ls_files(repo_root, "examples/**/target/**") {
        Ok(output) => {
            let clean = output.trim().is_empty();
            criteria.push(Criterion::new(
                "hygiene_no_target_artifacts",
                "Hygiene: no tracked target/ artifacts",
                clean,
                if clean {
                    None
                } else {
                    Some("tracked target/ artifacts found in git".into())
                },
            ));
        }
        Err(e) => {
            criteria.push(Criterion::new(
                "hygiene_no_target_artifacts",
                "Hygiene: no tracked target/ artifacts",
                false,
                Some(format!("could not check: {}", e)),
            ));
        }
    }

    // Tracked .wasm artifacts
    match run_git_ls_files(repo_root, "examples/**/*.wasm") {
        Ok(output) => {
            let clean = output.trim().is_empty();
            criteria.push(Criterion::new(
                "hygiene_no_wasm_artifacts",
                "Hygiene: no tracked .wasm artifacts",
                clean,
                if clean {
                    None
                } else {
                    Some("tracked .wasm artifacts found in git".into())
                },
            ));
        }
        Err(e) => {
            criteria.push(Criterion::new(
                "hygiene_no_wasm_artifacts",
                "Hygiene: no tracked .wasm artifacts",
                false,
                Some(format!("could not check: {}", e)),
            ));
        }
    }

    // Root .gitignore must NOT ignore Cargo.lock
    let gitignore = repo_root.join(".gitignore");
    if let Ok(contents) = fs::read_to_string(&gitignore) {
        let re = Regex::new(r"(?m)^Cargo\.lock$").expect("valid regex");
        let cargo_lock_ignored = re.is_match(&contents);
        criteria.push(Criterion::new(
            "hygiene_root_gitignore",
            "Hygiene: root .gitignore does not ignore Cargo.lock",
            !cargo_lock_ignored,
            if cargo_lock_ignored {
                Some("root .gitignore ignores Cargo.lock".into())
            } else {
                None
            },
        ));
    }

    criteria
}

// ---------------------------------------------------------------------------
// Hygiene checks (reuse base check logic, adapted for criterion format)
// ---------------------------------------------------------------------------

fn check_hygiene(dir: &Path, name: &str, criteria: &mut Vec<Criterion>) {
    let mut failures: Vec<String> = Vec::new();

    // No hardcoded contract IDs in README
    check_readme_contract_ids(dir, name, &mut failures);
    criteria.push(Criterion::new(
        "readme_no_contract_ids",
        "README: no hardcoded contract IDs",
        failures.is_empty(),
        failures.first().cloned(),
    ));
    failures.clear();

    // .gitignore exists and has target/
    check_example_gitignore(dir, name, &mut failures);
    criteria.push(Criterion::new(
        "gitignore_exists",
        ".gitignore: exists and ignores target/",
        failures.is_empty(),
        failures.first().cloned(),
    ));
    failures.clear();

    // .gitignore must NOT ignore Cargo.lock
    check_gitignore_cargo_lock(dir, name, &mut failures);
    criteria.push(Criterion::new(
        "gitignore_no_cargo_lock",
        ".gitignore: does not ignore Cargo.lock",
        failures.is_empty(),
        failures.first().cloned(),
    ));
    failures.clear();

    // Cargo.toml has description
    check_cargo_toml_description(dir, name, &mut failures);
    criteria.push(Criterion::new(
        "cargo_toml_description",
        "Cargo.toml: has non-empty description",
        failures.is_empty(),
        failures.first().cloned(),
    ));
    failures.clear();

    // cargo metadata passes
    check_cargo_metadata(dir, name, &mut failures);
    criteria.push(Criterion::new(
        "cargo_metadata",
        "cargo metadata --no-deps passes",
        failures.is_empty(),
        failures.first().cloned(),
    ));
}

/// Check that example `.gitignore` does NOT contain `Cargo.lock`.
fn check_gitignore_cargo_lock(dir: &Path, name: &str, failures: &mut Vec<String>) {
    let gitignore = dir.join(".gitignore");
    match fs::read_to_string(&gitignore) {
        Ok(contents) => {
            let re = Regex::new(r"(?m)^Cargo\.lock$").expect("valid regex");
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

// ---------------------------------------------------------------------------
// Dependencies (§1)
// ---------------------------------------------------------------------------

fn check_dependencies(dir: &Path, criteria: &mut Vec<Criterion>) {
    let cargo_toml = dir.join("Cargo.toml");
    let contents = match fs::read_to_string(&cargo_toml) {
        Ok(c) => c,
        Err(e) => {
            criteria.push(Criterion::new(
                "deps_no_path",
                "Dependencies: no unannotated path dependency on cougr-core",
                false,
                Some(format!("cannot read Cargo.toml: {}", e)),
            ));
            return;
        }
    };

    // Check for path dependency on cougr-core
    let path_re = Regex::new(r#"cougr-core\s*=\s*\{[^}]*path\s*="#).expect("valid regex");
    if !path_re.is_match(&contents) {
        criteria.push(Criterion::new(
            "deps_no_path",
            "Dependencies: uses published cougr-core version (not path dep)",
            true,
            None,
        ));
    } else {
        let annotated = has_path_dep_annotation(&contents);
        criteria.push(Criterion::new(
            "deps_no_path",
            "Dependencies: path dep is annotated per §1.1",
            annotated,
            if annotated {
                None
            } else {
                Some("path dependency on cougr-core without annotation comment".into())
            },
        ));
    }

    // Check for wildcard version specifiers ("*")
    let wildcard_re = Regex::new(r#"(?m)^\w[\w-]*\s*=\s*"[*]""#).expect("valid regex");
    let has_wildcard = wildcard_re.is_match(&contents);
    criteria.push(Criterion::new(
        "deps_no_wildcard",
        "Cargo.toml: no wildcard version specifiers",
        !has_wildcard,
        if has_wildcard {
            Some("wildcard version (*) found in dependency".into())
        } else {
            None
        },
    ));
}

fn has_path_dep_annotation(contents: &str) -> bool {
    let annotation_re =
        Regex::new(r"(?m)^#\s*path dep\s*[—\-]\s*pending cougr-core").expect("valid regex");
    annotation_re.is_match(contents)
}

// ---------------------------------------------------------------------------
// Module structure (§3)
// ---------------------------------------------------------------------------

fn check_module_structure(dir: &Path, criteria: &mut Vec<Criterion>) {
    let src = dir.join("src");

    let has_components = src.join("components.rs").is_file();
    criteria.push(Criterion::new(
        "module_components",
        "Module: src/components.rs exists",
        has_components,
        if has_components {
            None
        } else {
            Some("missing src/components.rs".into())
        },
    ));

    let has_systems = src.join("systems.rs").is_file();
    criteria.push(Criterion::new(
        "module_systems",
        "Module: src/systems.rs exists",
        has_systems,
        if has_systems {
            None
        } else {
            Some("missing src/systems.rs".into())
        },
    ));

    // Heuristic: if components.rs exists, lib.rs should not contain impl_component!
    if has_components {
        check_lib_rs_separation(&src.join("lib.rs"), criteria);
    }
}

fn check_lib_rs_separation(lib: &Path, criteria: &mut Vec<Criterion>) {
    match fs::read_to_string(lib) {
        Ok(contents) => {
            let component_re = Regex::new(r"impl_component!\s*\(").expect("valid regex");
            let has_component_macro = component_re.is_match(&contents);

            let system_re = Regex::new(r"(?m)^pub\s+fn\s+\w+_system\s*\(").expect("valid regex");
            let has_system_fn = system_re.is_match(&contents);

            let clean = !has_component_macro && !has_system_fn;
            let mut detail = Vec::new();
            if has_component_macro {
                detail.push("impl_component! in lib.rs (should be in components.rs)");
            }
            if has_system_fn {
                detail.push("system functions in lib.rs (should be in systems.rs)");
            }
            criteria.push(Criterion::new(
                "module_lib_separation",
                "Module: lib.rs separation (no components/systems inline)",
                clean,
                if clean {
                    None
                } else {
                    Some(detail.join("; "))
                },
            ));
        }
        Err(_) => {
            criteria.push(Criterion::new(
                "module_lib_separation",
                "Module: lib.rs separation",
                false,
                Some("cannot read lib.rs".into()),
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// README completeness (§4)
// ---------------------------------------------------------------------------

fn check_readme_sections(dir: &Path, criteria: &mut Vec<Criterion>) {
    let readme = dir.join("README.md");
    let contents = match fs::read_to_string(&readme) {
        Ok(c) => c,
        Err(_) => {
            criteria.push(Criterion::new(
                "readme_sections",
                "README: all 8 required sections present",
                false,
                Some("README.md missing or unreadable".into()),
            ));
            return;
        }
    };

    let required: &[(&str, &[&str])] = &[
        ("purpose", &["## Purpose", "## Purpose and pattern"]),
        ("api", &["## Public contract API", "## Contract API", "## API"]),
        ("architecture", &["## Architecture", "## Architecture overview"]),
        ("storage", &["## Storage", "## Storage model"]),
        ("gameplay", &["## Main gameplay flow", "## Gameplay", "## Gameplay flow"]),
        ("cougr_apis", &["## Cougr APIs", "## Cougr APIs used"]),
        ("build", &["## Build", "## Build and test", "## Build and test commands"]),
        ("limitations", &["## Known limitations", "## Limitations"]),
    ];

    let mut missing: Vec<String> = Vec::new();
    for (id, headers) in required {
        let found = headers.iter().any(|h| {
            let escaped = regex::escape(h);
            let re = Regex::new(&format!("(?m)^{}", escaped)).expect("valid regex");
            re.is_match(&contents)
        });
        if !found {
            if *id == "build"
                && (contents.contains("cargo test") || contents.contains("stellar contract build"))
            {
                continue;
            }
            missing.push(id.to_string());
        }
    }

    criteria.push(Criterion::new(
        "readme_sections",
        "README: all required sections present",
        missing.is_empty(),
        if missing.is_empty() {
            None
        } else {
            Some(format!("missing section(s): {}", missing.join(", ")))
        },
    ));
}

// ---------------------------------------------------------------------------
// Test coverage (§5)
// ---------------------------------------------------------------------------

fn check_test_coverage(dir: &Path, criteria: &mut Vec<Criterion>) {
    let src = dir.join("src");

    let has_tests = src.join("test.rs").is_file()
        || src.join("tests.rs").is_file()
        || src.join("sandbox_tests.rs").is_file()
        || has_test_attr_in_lib(&src.join("lib.rs"));

    criteria.push(Criterion::new(
        "tests_exist",
        "Tests: test module or file present",
        has_tests,
        if has_tests {
            None
        } else {
            Some("no test file found (test.rs, tests.rs, sandbox_tests.rs, or #[test] in lib.rs)".into())
        },
    ));

    let test_count = count_tests(&src);
    let enough_tests = test_count >= 3;
    criteria.push(Criterion::new(
        "tests_count",
        "Tests: at least 3 test functions",
        enough_tests,
        if enough_tests {
            Some(format!("{} tests found", test_count))
        } else {
            Some(format!("only {} test(s) found (expect at least 3)", test_count))
        },
    ));

    let uses_testutils = check_testutils_usage(&src);
    criteria.push(Criterion::new(
        "tests_testutils",
        "Tests: uses soroban-sdk testutils",
        uses_testutils,
        if uses_testutils {
            None
        } else {
            Some("no soroban-sdk testutils import found in test files".into())
        },
    ));
}

fn has_test_attr_in_lib(lib: &Path) -> bool {
    match fs::read_to_string(lib) {
        Ok(contents) => contents.contains("#[test]"),
        Err(_) => false,
    }
}

fn count_tests(src: &Path) -> usize {
    let test_files = ["test.rs", "tests.rs", "sandbox_tests.rs", "lib.rs"];
    let test_re = Regex::new(r"#\[test\]").expect("valid regex");
    let mut count = 0;

    for fname in &test_files {
        let p = src.join(fname);
        if let Ok(contents) = fs::read_to_string(&p) {
            count += test_re.find_iter(&contents).count();
        }
    }
    count
}

fn check_testutils_usage(src: &Path) -> bool {
    let test_files = ["test.rs", "tests.rs", "sandbox_tests.rs", "lib.rs"];
    let testutils_re = Regex::new(r"soroban_sdk.*testutils|testutils").expect("valid regex");

    for fname in &test_files {
        let p = src.join(fname);
        if let Ok(contents) = fs::read_to_string(&p) {
            if testutils_re.is_match(&contents) {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Classification (§7)
// ---------------------------------------------------------------------------

fn check_classification(dir: &Path, criteria: &mut Vec<Criterion>) {
    let readme = dir.join("README.md");
    let contents = match fs::read_to_string(&readme) {
        Ok(c) => c,
        Err(_) => {
            criteria.push(Criterion::new(
                "classification",
                "Classification: marked as canonical or transitional",
                false,
                Some("README.md missing".into()),
            ));
            return;
        }
    };

    let is_canonical = contents.contains("Canonical example")
        || contents.contains("**Canonical")
        || contents.contains("canonical example");
    let is_transitional = contents.contains("Transitional example")
        || contents.contains("**Transitional")
        || contents.contains("transitional example");

    if is_canonical || is_transitional {
        let marker = if is_canonical { "canonical" } else { "transitional" };
        criteria.push(Criterion::new(
            "classification",
            "Classification: marked as canonical or transitional",
            true,
            Some(format!("marked as {}", marker)),
        ));
    } else {
        criteria.push(Criterion::new(
            "classification",
            "Classification: marked as canonical or transitional",
            false,
            Some("no 'Canonical example' or 'Transitional example' marker found in README".into()),
        ));
    }
}

// ---------------------------------------------------------------------------
// Cargo.lock committed
// ---------------------------------------------------------------------------

fn check_cargo_lock_committed(repo_root: &Path, name: &str, criteria: &mut Vec<Criterion>) {
    let lock_path = format!("examples/{}/Cargo.lock", name);
    match run_git_ls_files(repo_root, &lock_path) {
        Ok(output) => {
            let committed = !output.trim().is_empty();
            criteria.push(Criterion::new(
                "cargo_lock_committed",
                "Cargo.lock is committed",
                committed,
                if committed {
                    None
                } else {
                    Some("Cargo.lock is not tracked by git".into())
                },
            ));
        }
        Err(e) => {
            criteria.push(Criterion::new(
                "cargo_lock_committed",
                "Cargo.lock is committed",
                false,
                Some(format!("could not check: {}", e)),
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// Build validation (heavy, opt-in via --full)
// ---------------------------------------------------------------------------

fn check_cargo_test(dir: &Path, criteria: &mut Vec<Criterion>) {
    let output = Command::new("cargo")
        .args(["test"])
        .current_dir(dir)
        .output();

    match output {
        Ok(out) => {
            let pass = out.status.success();
            criteria.push(Criterion::new(
                "build_cargo_test",
                "Build: cargo test passes",
                pass,
                if pass {
                    None
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    Some(format!(
                        "cargo test failed:\n{}",
                        indent_lines(
                            &format!("{}\n{}", stdout.trim(), stderr.trim()),
                            "      "
                        )
                    ))
                },
            ));
        }
        Err(e) => {
            criteria.push(Criterion::new(
                "build_cargo_test",
                "Build: cargo test passes",
                false,
                Some(format!("could not run cargo test: {}", e)),
            ));
        }
    }
}

fn check_stellar_build(dir: &Path, criteria: &mut Vec<Criterion>) {
    let output = Command::new("stellar")
        .args(["contract", "build"])
        .current_dir(dir)
        .output();

    match output {
        Ok(out) => {
            let pass = out.status.success();
            criteria.push(Criterion::new(
                "build_stellar",
                "Build: stellar contract build passes",
                pass,
                if pass {
                    None
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    Some(format!(
                        "stellar contract build failed:\n{}",
                        indent_lines(stderr.trim(), "      ")
                    ))
                },
            ));
        }
        Err(e) => {
            criteria.push(Criterion::new(
                "build_stellar",
                "Build: stellar contract build passes",
                false,
                Some(format!(
                    "could not run stellar: {} (is stellar-cli installed?)",
                    e
                )),
            ));
        }
    }
}
