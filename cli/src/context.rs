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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Build a fake repo with a Cargo.toml and the given example names.
    fn fake_repo(examples: &[&str]) -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[workspace]\n").unwrap();
        let examples_dir = dir.path().join("examples");
        fs::create_dir_all(&examples_dir).unwrap();
        for name in examples {
            let ex = examples_dir.join(name);
            fs::create_dir_all(&ex).unwrap();
            fs::write(ex.join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
        }
        dir
    }

    #[test]
    fn resolve_explicit_root_discovers_examples() {
        let repo = fake_repo(&["alpha", "beta"]);
        let ctx = resolve(
            repo.path(),
            Some(repo.path().to_str().unwrap()),
            None,
        )
        .unwrap();
        let mut names: Vec<_> = ctx.examples.iter().map(|e| e.name.clone()).collect();
        names.sort();
        assert_eq!(names, vec!["alpha".to_string(), "beta".to_string()]);
    }

    #[test]
    fn resolve_explicit_root_missing_bails() {
        let repo = fake_repo(&["alpha"]);
        let missing = repo.path().join("does-not-exist");
        let err = resolve(repo.path(), Some(missing.to_str().unwrap()), None).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("explicit --path does not exist"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn resolve_single_example_found() {
        let repo = fake_repo(&["alpha", "beta"]);
        let ctx = resolve(
            repo.path(),
            Some(repo.path().to_str().unwrap()),
            Some("beta"),
        )
        .unwrap();
        assert_eq!(ctx.examples.len(), 1);
        assert_eq!(ctx.examples[0].name, "beta");
    }

    #[test]
    fn resolve_single_example_not_found_bails() {
        let repo = fake_repo(&["alpha"]);
        let err = resolve(
            repo.path(),
            Some(repo.path().to_str().unwrap()),
            Some("nope"),
        )
        .unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("example 'nope' not found"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn resolve_finds_repo_root_from_subdir() {
        let repo = fake_repo(&["alpha"]);
        let sub = repo.path().join("examples").join("alpha");
        let ctx = resolve(&sub, None, None).unwrap();
        // Auto-detect: cwd is inside examples/alpha, so only that one.
        assert_eq!(ctx.examples.len(), 1);
        assert_eq!(ctx.examples[0].name, "alpha");
        assert_eq!(ctx.repo_root, repo.path().canonicalize().unwrap());
    }

    #[test]
    fn resolve_no_repo_root_bails() {
        let dir = tempfile::tempdir().unwrap();
        // No Cargo.toml, no examples/ anywhere above.
        let err = resolve(dir.path(), None, None).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("could not find repo root"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn resolve_no_examples_found_bails() {
        let repo = fake_repo(&[]);
        let err = resolve(repo.path(), Some(repo.path().to_str().unwrap()), None).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("no examples found under"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn example_dir_joins_components() {
        let root = Path::new("/tmp/repo");
        assert_eq!(
            example_dir(root, "alpha"),
            PathBuf::from("/tmp/repo/examples/alpha")
        );
    }

    #[test]
    fn canonical_example_names_has_ten_entries() {
        let names = canonical_example_names();
        assert_eq!(names.len(), 10);
        assert!(names.contains(&"spawn_and_move"));
        assert!(names.contains(&"guild_arena"));
    }
}
