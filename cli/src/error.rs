//! Error type for the CLI.
//!
//! Every variant carries enough context to print an actionable message: what
//! went wrong, and - through [`CliError::hint`] - what the user can do about
//! it. Nothing in the command path is allowed to panic on user input.

use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum CliError {
    /// The project name is not usable as a Rust crate name.
    InvalidName { name: String, reason: String },

    /// The studio turn-based configuration is malformed or outside its bounds.
    InvalidConfig { reason: String },

    /// The target directory already exists.
    TargetExists { path: PathBuf },

    /// A filesystem operation failed.
    Io {
        action: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },

    /// A template file could not be read out of the embedded bundle. This is a
    /// packaging bug rather than a user error, but it must not panic either.
    MissingTemplateAsset {
        template: &'static str,
        file: String,
    },

    /// A requested embedded piece does not exist.
    UnknownPiece { name: String, available: String },

    /// The current directory is not a generated or otherwise valid project.
    InvalidProject { path: PathBuf },

    /// An add operation would overwrite an existing project file.
    PieceConflict { piece: String, files: Vec<PathBuf> },

    /// A piece file could not be read from the embedded bundle.
    MissingPieceAsset { piece: String, file: String },
}

impl CliError {
    pub fn io(action: &'static str, path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        CliError::Io {
            action,
            path: path.into(),
            source,
        }
    }

    /// A one-line suggestion printed under the error, when one applies.
    pub fn hint(&self) -> Option<String> {
        match self {
            CliError::InvalidName { .. } => Some(
                "crate names start with a letter and use letters, digits, `_` or `-` \
                 (for example: `my-game` or `dungeon_crawl`)"
                    .to_string(),
            ),
            CliError::InvalidConfig { .. } => Some(
                "use board_width and board_height in 3..=8, and win_length in 3..=min(width, height)".to_string(),
            ),
            CliError::TargetExists { path } => Some(format!(
                "pick a different name, or remove `{}` first",
                path.display()
            )),
            CliError::Io { .. } => None,
            CliError::MissingTemplateAsset { .. } => Some(
                "this is a bug in cougr-cli - please report it at \
                 https://github.com/salazarsebas/Cougr/issues"
                    .to_string(),
            ),
            CliError::UnknownPiece { .. } => {
                Some("run `cougr add --list` to see available pieces".to_string())
            }
            CliError::InvalidProject { .. } | CliError::PieceConflict { .. } => None,
            CliError::MissingPieceAsset { .. } => Some(
                "this is a bug in cougr-cli - please report it at https://github.com/salazarsebas/Cougr/issues"
                    .to_string(),
            ),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::InvalidName { name, reason } => {
                write!(f, "`{name}` is not a valid project name: {reason}")
            }
            CliError::InvalidConfig { reason } => write!(f, "invalid turn-based config: {reason}"),
            CliError::TargetExists { path } => {
                write!(f, "target directory `{}` already exists", path.display())
            }
            CliError::Io {
                action,
                path,
                source,
            } => write!(f, "failed to {action} `{}`: {source}", path.display()),
            CliError::MissingTemplateAsset { template, file } => write!(
                f,
                "template `{template}` is missing the embedded file `{file}`"
            ),
            CliError::UnknownPiece { name, available } => {
                write!(f, "unknown piece `{name}` (available: {available})")
            }
            CliError::InvalidProject { path } => write!(
                f,
                "`{}` is not a Cougr project (expected Cargo.toml and src/lib.rs)",
                path.display()
            ),
            CliError::PieceConflict { piece, files } => {
                write!(f, "cannot add `{piece}` because these files already exist:")?;
                for file in files {
                    write!(f, "\n  {} (would be written)", file.display())?;
                }
                Ok(())
            }
            CliError::MissingPieceAsset { piece, file } => {
                write!(f, "piece `{piece}` is missing the embedded file `{file}`")
            }
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CliError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
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