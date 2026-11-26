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

#[derive(Default)]
struct CatalogEntry {
    key: String,
    name: Option<String>,
    maturity: Option<String>,
}

/// Return the canonical example names listed in `examples/catalog.toml`.
pub fn canonical_example_names(repo_root: &Path) -> Result<Vec<String>> {
    let catalog_path = repo_root.join("examples").join("catalog.toml");
    let content = fs::read_to_string(&catalog_path).with_context(|| {
        format!(
            "failed to read example catalog at {}",
            catalog_path.display()
        )
    })?;

    let entries = parse_catalog(&content).with_context(|| {
        format!(
            "failed to parse example catalog at {}",
            catalog_path.display()
        )
    })?;

    if entries.is_empty() {
        anyhow::bail!("no [example.*] entries found in {}", catalog_path.display());
    }

    let mut names: Vec<String> = entries
        .into_iter()
        .filter(|entry| entry.maturity.as_deref() == Some("canonical"))
        .filter_map(|entry| entry.name)
        .collect();
    names.sort();
    Ok(names)
}

fn parse_catalog(source: &str) -> Result<Vec<CatalogEntry>> {
    let mut entries: Vec<CatalogEntry> = Vec::new();
    let mut current: Option<CatalogEntry> = None;

    for (index, raw_line) in source.lines().enumerate() {
        let line_num = index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') {
            if !line.ends_with(']') {
                anyhow::bail!("line {line_num}: unterminated table header: {line}");
            }
            let header = &line[1..line.len() - 1].trim();
            let Some(key) = header.strip_prefix("example.") else {
                anyhow::bail!("line {line_num}: unexpected table header [{header}]");
            };
            if key.is_empty() {
                anyhow::bail!("line {line_num}: empty example key in [{header}]");
            }
            if let Some(entry) = current.take() {
                validate_catalog_entry(&entry)?;
                entries.push(entry);
            }
            current = Some(CatalogEntry {
                key: key.to_string(),
                ..CatalogEntry::default()
            });
            continue;
        }

        let Some((raw_key, raw_value)) = line.split_once('=') else {
            anyhow::bail!("line {line_num}: expected key = value, got: {line}");
        };
        let key = raw_key.trim();
        let value = raw_value.trim();
        let entry = current
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("line {line_num}: key '{key}' outside [example.*]"))?;

        match key {
            "name" => entry.name = Some(parse_toml_string(value, line_num)?),
            "maturity" => entry.maturity = Some(parse_toml_string(value, line_num)?),
            _ => {}
        }
    }

    if let Some(entry) = current.take() {
        validate_catalog_entry(&entry)?;
        entries.push(entry);
    }

    Ok(entries)
}

fn validate_catalog_entry(entry: &CatalogEntry) -> Result<()> {
    if entry.name.as_deref().unwrap_or("").is_empty() {
        anyhow::bail!("example.{} is missing required field 'name'", entry.key);
    }
    if entry.maturity.as_deref().unwrap_or("").is_empty() {
        anyhow::bail!("example.{} is missing required field 'maturity'", entry.key);
    }
    Ok(())
}

fn parse_toml_string(raw: &str, line_num: usize) -> Result<String> {
    let trimmed = raw.trim();
    if !trimmed.starts_with('"') {
        anyhow::bail!("line {line_num}: expected quoted string, got: {trimmed}");
    }
    let rest = &trimmed[1..];
    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                let trailing = chars.as_str().trim();
                if !trailing.is_empty() && !trailing.starts_with('#') {
                    anyhow::bail!(
                        "line {line_num}: unexpected trailing characters after string: {trailing}"
                    );
                }
                return Ok(out);
            }
            '\\' => {
                let escaped = chars
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("line {line_num}: unterminated escape"))?;
                match escaped {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    other => anyhow::bail!("line {line_num}: unsupported escape \\{other}"),
                }
            }
            other => out.push(other),
        }
    }
    anyhow::bail!("line {line_num}: unterminated string literal")
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

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_root() -> tempfile::TempDir {
        let root = tempfile::tempdir().expect("create fake repository");
        fs::write(root.path().join("Cargo.toml"), "[workspace]\n").expect("write root manifest");
        root
    }

    #[test]
    fn resolve_reports_an_unknown_example() {
        let root = fake_root();
        fs::create_dir(root.path().join("examples")).expect("create examples directory");
        let root_arg = root.path().to_str().expect("UTF-8 temp path");

        let error = resolve(root.path(), Some(root_arg), Some("missing"))
            .err()
            .expect("unknown example should fail");

        assert!(
            error.to_string().contains("example 'missing' not found"),
            "{error:#}"
        );
    }

    #[test]
    fn resolve_reports_when_the_repo_root_walk_finds_nothing() {
        let cwd = tempfile::tempdir().expect("create directory outside a repository");

        let error = resolve(cwd.path(), None, None)
            .err()
            .expect("repo root walk should fail");

        assert!(
            error.to_string().contains("could not find repo root"),
            "{error:#}"
        );
    }

    #[test]
    fn resolve_reports_a_missing_examples_directory() {
        let root = fake_root();
        let root_arg = root.path().to_str().expect("UTF-8 temp path");

        let error = resolve(root.path(), Some(root_arg), None)
            .err()
            .expect("missing examples directory should fail");

        assert!(
            error.to_string().contains("examples/ directory not found"),
            "{error:#}"
        );
    }

    #[test]
    fn resolve_reports_when_no_example_has_a_cargo_manifest() {
        let root = fake_root();
        fs::create_dir_all(root.path().join("examples").join("not-a-package"))
            .expect("create non-package example directory");
        let root_arg = root.path().to_str().expect("UTF-8 temp path");

        let error = resolve(root.path(), Some(root_arg), None)
            .err()
            .expect("empty examples directory should fail");

        assert!(error.to_string().contains("no examples found"), "{error:#}");
    }

    #[test]
    fn canonical_example_names_filters_canonical_entries_using_name_field() {
        let root = fake_root();
        let examples_dir = root.path().join("examples");
        fs::create_dir(&examples_dir).expect("create examples directory");
        fs::write(
            examples_dir.join("catalog.toml"),
            r#"
[example.alpha_key]
name = "alpha_dir"
category = "arcade"
maturity = "canonical"
cougr_features = ["ecs"]

[example.beta]
name = "beta"
category = "board"
maturity = "transitional"
cougr_features = ["ecs"]

[example.gamma]
name = "gamma"
category = "card"
maturity = "canonical"
cougr_features = ["privacy"]
"#,
        )
        .expect("write catalog");

        let names = canonical_example_names(root.path()).expect("parse catalog");
        assert_eq!(names, vec!["alpha_dir", "gamma"]);
    }

    #[test]
    fn canonical_example_names_reports_missing_catalog() {
        let root = fake_root();
        let error = canonical_example_names(root.path()).expect_err("missing catalog should fail");

        assert!(
            error.to_string().contains("failed to read example catalog"),
            "{error:#}"
        );
    }

    #[test]
    fn canonical_example_names_reports_malformed_catalog() {
        let root = fake_root();
        let examples_dir = root.path().join("examples");
        fs::create_dir(&examples_dir).expect("create examples directory");
        fs::write(
            examples_dir.join("catalog.toml"),
            "[example.broken\nname = \"oops\"\n",
        )
        .expect("write malformed catalog");

        let error =
            canonical_example_names(root.path()).expect_err("malformed catalog should fail");

        assert!(
            error
                .to_string()
                .contains("failed to parse example catalog"),
            "{error:#}"
        );
    }

    #[test]
    fn canonical_example_names_reports_empty_catalog() {
        let root = fake_root();
        let examples_dir = root.path().join("examples");
        fs::create_dir(&examples_dir).expect("create examples directory");
        fs::write(examples_dir.join("catalog.toml"), "# empty\n").expect("write empty catalog");

        let error = canonical_example_names(root.path()).expect_err("empty catalog should fail");

        assert!(
            error.to_string().contains("no [example.*] entries found"),
            "{error:#}"
        );
    }
}
