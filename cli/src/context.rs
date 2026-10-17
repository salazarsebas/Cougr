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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// Build a fake repo with a Cargo.toml and the given example names
    /// (each with its own Cargo.toml so `discover_examples` picks them up).
    fn fake_repo(examples: &[&str]) -> tempfile::TempDir {
        let dir = tempdir().unwrap();
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

    // -- resolve: explicit --path ------------------------------------------

    #[test]
    fn resolve_explicit_root_discovers_examples() {
        let repo = fake_repo(&["alpha", "beta"]);
        let root = repo.path().canonicalize().unwrap();
        let ctx = resolve(&root, Some(root.to_str().unwrap()), None).unwrap();
        assert_eq!(ctx.repo_root, root);
        let mut names: Vec<_> = ctx.examples.iter().map(|e| e.name.clone()).collect();
        names.sort();
        assert_eq!(names, vec!["alpha", "beta"]);
    }

    #[test]
    fn resolve_explicit_root_missing_path_bails() {
        let repo = fake_repo(&["alpha"]);
        let missing = repo.path().join("does-not-exist");
        let err = resolve(repo.path(), Some(missing.to_str().unwrap()), None).unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("explicit --path does not exist"),
            "unexpected error: {msg}"
        );
    }

    // -- resolve: single_example (bail at :49) -----------------------------

    #[test]
    fn resolve_single_example_found() {
        let repo = fake_repo(&["alpha", "beta"]);
        let root = repo.path().canonicalize().unwrap();
        let ctx = resolve(&root, Some(root.to_str().unwrap()), Some("beta")).unwrap();
        assert_eq!(ctx.examples.len(), 1);
        assert_eq!(ctx.examples[0].name, "beta");
    }

    #[test]
    fn resolve_single_example_not_found_bails() {
        let repo = fake_repo(&["alpha"]);
        let root = repo.path().canonicalize().unwrap();
        let err = resolve(&root, Some(root.to_str().unwrap()), Some("nope")).unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("example 'nope' not found"),
            "unexpected error: {msg}"
        );
    }

    // -- resolve: auto-detect inside examples/<name>/ ----------------------

    #[test]
    fn resolve_auto_detects_inside_example_dir() {
        let repo = fake_repo(&["alpha", "beta"]);
        let root = repo.path().canonicalize().unwrap();
        let inside = root.join("examples").join("alpha");
        let ctx = resolve(&inside, None, None).unwrap();
        assert_eq!(ctx.examples.len(), 1);
        assert_eq!(ctx.examples[0].name, "alpha");
    }

    // -- resolve: find_repo_root walk (bail at :107) -----------------------

    #[test]
    fn resolve_no_repo_root_bails() {
        // A tempdir with no Cargo.toml and no examples/ anywhere above it
        // (tempdirs live under /tmp, which has neither).
        let dir = tempdir().unwrap();
        let err = resolve(dir.path(), None, None).unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("could not find repo root"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn resolve_walks_up_to_repo_root() {
        let repo = fake_repo(&["alpha"]);
        let root = repo.path().canonicalize().unwrap();
        let nested = root.join("examples").join("alpha").join("src");
        fs::create_dir_all(&nested).unwrap();
        // From a nested dir that is not directly inside examples/<name>/,
        // find_repo_root should walk up and discover all examples.
        let ctx = resolve(&nested, None, None).unwrap();
        assert_eq!(ctx.repo_root, root);
        assert_eq!(ctx.examples.len(), 1);
        assert_eq!(ctx.examples[0].name, "alpha");
    }

    // -- discover_examples: bail at :131 (no examples/ dir) ----------------

    #[test]
    fn resolve_missing_examples_dir_bails() {
        let dir = tempdir().unwrap();
        // Cargo.toml present so find_repo_root succeeds, but no examples/.
        fs::write(dir.path().join("Cargo.toml"), "[workspace]\n").unwrap();
        let root = dir.path().canonicalize().unwrap();
        // Pass explicit_root so we skip find_repo_root and hit discover_examples.
        let err = resolve(&root, Some(root.to_str().unwrap()), None).unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("examples/ directory not found"),
            "unexpected error: {msg}"
        );
    }

    // -- discover_examples: bail at :152 (empty examples/) -----------------

    #[test]
    fn resolve_empty_examples_dir_bails() {
        let repo = fake_repo(&[]);
        let root = repo.path().canonicalize().unwrap();
        let err = resolve(&root, Some(root.to_str().unwrap()), None).unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("no examples found"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn discover_examples_skips_dirs_without_cargo_toml() {
        let repo = fake_repo(&["alpha"]);
        let root = repo.path().canonicalize().unwrap();
        // A directory under examples/ that has no Cargo.toml must be skipped.
        fs::create_dir_all(root.join("examples").join("not_an_example")).unwrap();
        let ctx = resolve(&root, Some(root.to_str().unwrap()), None).unwrap();
        assert_eq!(ctx.examples.len(), 1);
        assert_eq!(ctx.examples[0].name, "alpha");
    }

    // -- example_dir / canonical_example_names -----------------------------

    #[test]
    fn example_dir_joins_examples_and_name() {
        let root = Path::new("/tmp/repo");
        assert_eq!(
            example_dir(root, "alpha"),
            PathBuf::from("/tmp/repo/examples/alpha")
        );
    }

    #[test]
    fn canonical_example_names_has_ten_entries() {
        assert_eq!(canonical_example_names().len(), 10);
    }
}
