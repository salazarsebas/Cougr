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

    /// The target path already exists (as a file or a directory).
    TargetExists { path: PathBuf, is_file: bool },

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
            CliError::TargetExists { path, is_file } => Some(format!(
                "pick a different name, or remove the {} `{}` first",
                if *is_file { "file" } else { "directory" },
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
            CliError::InvalidProject { .. } => Some(
                "run `cougr add` from inside a project created with `cougr new`"
                    .to_string(),
            ),
            CliError::PieceConflict { .. } => None,
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
            CliError::TargetExists { path, is_file } => write!(
                f,
                "target {} `{}` already exists",
                if *is_file { "file" } else { "directory" },
                path.display()
            ),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Table-driven test asserting `hint()` for every `CliError` variant,
    /// including the ones that intentionally return `None`.
    #[test]
    fn hint_covers_every_variant() {
        let cases: Vec<(CliError, Option<&str>)> = vec![
            (
                CliError::InvalidName {
                    name: "1bad".to_string(),
                    reason: "must start with a letter".to_string(),
                },
                Some(
                    "crate names start with a letter and use letters, digits, `_` or `-` \
                     (for example: `my-game` or `dungeon_crawl`)",
                ),
            ),
            (
                CliError::InvalidConfig {
                    reason: "board_width out of range".to_string(),
                },
                Some(
                    "use board_width and board_height in 3..=8, and win_length in 3..=min(width, height)",
                ),
            ),
            (
                CliError::TargetExists {
                    path: PathBuf::from("/tmp/foo"),
                    is_file: false,
                },
                Some("pick a different name, or remove the directory `/tmp/foo` first"),
            ),
            (
                CliError::TargetExists {
                    path: PathBuf::from("/tmp/foo.txt"),
                    is_file: true,
                },
                Some("pick a different name, or remove the file `/tmp/foo.txt` first"),
            ),
            (
                CliError::io(
                    "write",
                    PathBuf::from("/tmp/foo"),
                    std::io::Error::new(std::io::ErrorKind::Other, "boom"),
                ),
                None,
            ),
            (
                CliError::MissingTemplateAsset {
                    template: "turn_based",
                    file: "src/lib.rs".to_string(),
                },
                Some(
                    "this is a bug in cougr-cli - please report it at \
                     https://github.com/salazarsebas/Cougr/issues",
                ),
            ),
            (
                CliError::UnknownPiece {
                    name: "foo".to_string(),
                    available: "bar, baz".to_string(),
                },
                Some("run `cougr add --list` to see available pieces"),
            ),
            (
                CliError::InvalidProject {
                    path: PathBuf::from("/tmp/foo"),
                },
                Some("run `cougr add` from inside a project created with `cougr new`"),
            ),
            (
                CliError::PieceConflict {
                    piece: "foo".to_string(),
                    files: vec![PathBuf::from("/tmp/foo/src/lib.rs")],
                },
                None,
            ),
            (
                CliError::MissingPieceAsset {
                    piece: "foo".to_string(),
                    file: "src/lib.rs".to_string(),
                },
                Some(
                    "this is a bug in cougr-cli - please report it at https://github.com/salazarsebas/Cougr/issues",
                ),
            ),
        ];

        for (err, expected) in cases {
            let actual = err.hint();
            match expected {
                Some(want) => {
                    assert_eq!(actual.as_deref(), Some(want), "unexpected hint for {err:?}")
                }
                None => assert_eq!(actual, None, "expected no hint for {err:?}"),
            }
        }
    }
}
