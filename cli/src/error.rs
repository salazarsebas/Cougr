//! Error type for the CLI.
//!
//! Every variant carries enough context to print an actionable message: what
//! went wrong, and — through [`CliError::hint`] — what the user can do about
//! it. Nothing in the command path is allowed to panic on user input.

use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum CliError {
    /// The project name is not usable as a Rust crate name.
    InvalidName { name: String, reason: String },

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
            CliError::TargetExists { path } => Some(format!(
                "pick a different name, or remove `{}` first",
                path.display()
            )),
            CliError::Io { .. } => None,
            CliError::MissingTemplateAsset { .. } => Some(
                "this is a bug in cougr-cli — please report it at \
                 https://github.com/salazarsebas/Cougr/issues"
                    .to_string(),
            ),
            CliError::UnknownPiece { .. } => {
                Some("run `cougr add --list` to see available pieces".to_string())
            }
            CliError::InvalidProject { .. } | CliError::PieceConflict { .. } => None,
            CliError::MissingPieceAsset { .. } => Some(
                "this is a bug in cougr-cli — please report it at https://github.com/salazarsebas/Cougr/issues"
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
            CliError::MissingPieceAsset { piece, file } => write!(
                f,
                "piece `{piece}` is missing the embedded file `{file}`"
            ),
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
