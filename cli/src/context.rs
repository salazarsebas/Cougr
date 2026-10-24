//! Shared context resolution for `cougr check` and `cougr check --verified`.
//!
//! Determines the repository root and which examples to check, supporting
//! auto-detection from cwd, explicit `--path`, and `--example` flags.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
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

/// Return the canonical example directory names from `examples/catalog.toml`.
pub fn canonical_example_names(repo_root: &Path) -> Result<Vec<String>> {
    let catalog_path = repo_root.join("examples").join("catalog.toml");
    let contents = fs::read_to_string(&catalog_path)
        .with_context(|| format!("cannot read example catalog at {}", catalog_path.display()))?;

    let catalog: Catalog = toml::from_str(&contents)
        .with_context(|| format!("malformed example catalog at {}", catalog_path.display()))?;

    if catalog.example.is_empty() {
        anyhow::bail!(
            "malformed example catalog at {}: no [example.<key>] entries found",
            catalog_path.display()
        );
    }

    let mut canonical = Vec::new();
    for (key, entry) in catalog.example {
        if entry.name.trim().is_empty() {
            anyhow::bail!(
                "malformed example catalog at {}: example.{} has an empty name",
                catalog_path.display(),
                key
            );
        }
        match entry.maturity.as_str() {
            "canonical" => canonical.push(entry.name),
            "transitional" => {}
            other => anyhow::bail!(
                "malformed example catalog at {}: example.{} has invalid maturity '{}'",
                catalog_path.display(),
                key,
                other
            ),
        }
    }

    Ok(canonical)
}

#[derive(Deserialize)]
struct Catalog {
    #[serde(default)]
    example: BTreeMap<String, CatalogEntry>,
}

#[derive(Deserialize)]
struct CatalogEntry {
    name: String,
    #[allow(dead_code)]
    category: String,
    maturity: String,
    #[allow(dead_code)]
    cougr_features: Vec<String>,
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

    fn write_catalog(repo_root: &Path, content: &str) {
        let examples_dir = repo_root.join("examples");
        fs::create_dir_all(&examples_dir).unwrap();
        fs::write(examples_dir.join("catalog.toml"), content).unwrap();
    }

    #[test]
    fn canonical_example_names_returns_canonical_names_from_name_field() {
        let dir = tempfile::tempdir().unwrap();
        write_catalog(
            dir.path(),
            r#"
[example.starter_key]
name = "spawn_and_move"
category = "other"
maturity = "canonical"
cougr_features = ["soroban-game", "ecs"]

[example.arcade_pong]
name = "pong"
category = "arcade"
maturity = "transitional"
cougr_features = ["game-app", "ecs"]

[example.board_key]
name = "tic_tac_toe"
category = "board"
maturity = "canonical"
cougr_features = ["soroban-game"]
"#,
        );

        let names = canonical_example_names(dir.path()).unwrap();
        assert_eq!(
            names,
            vec!["tic_tac_toe".to_string(), "spawn_and_move".to_string()]
        );
    }

    #[test]
    fn canonical_example_names_errors_when_catalog_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let err = canonical_example_names(dir.path()).unwrap_err();
        assert!(err.to_string().contains("cannot read example catalog"));
    }

    #[test]
    fn canonical_example_names_errors_when_catalog_is_malformed() {
        let dir = tempfile::tempdir().unwrap();
        write_catalog(dir.path(), "[example.broken\nname = 123");
        let err = canonical_example_names(dir.path()).unwrap_err();
        assert!(err.to_string().contains("malformed example catalog"));

        write_catalog(
            dir.path(),
            r#"
[example.invalid_maturity]
name = "demo"
category = "other"
maturity = "experimental"
cougr_features = ["ecs"]
"#,
        );
        let err = canonical_example_names(dir.path()).unwrap_err();
        assert!(err.to_string().contains("invalid maturity"));

        write_catalog(dir.path(), "# empty catalog\n");
        let err = canonical_example_names(dir.path()).unwrap_err();
        assert!(err.to_string().contains("no [example.<key>] entries found"));
    }
}
