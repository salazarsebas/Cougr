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
    use std::io;

    fn io_error() -> io::Error {
        io::Error::new(io::ErrorKind::NotFound, "boom")
    }

    fn all_variants() -> Vec<CliError> {
        vec![
            CliError::InvalidName {
                name: "1bad".to_string(),
                reason: "must start with a letter".to_string(),
            },
            CliError::InvalidConfig {
                reason: "board_width out of range".to_string(),
            },
            CliError::TargetExists {
                path: PathBuf::from("/tmp/my-game"),
                is_file: false,
            },
            CliError::TargetExists {
                path: PathBuf::from("/tmp/my-game"),
                is_file: true,
            },
            CliError::Io {
                action: "write",
                path: PathBuf::from("/tmp/out.txt"),
                source: io_error(),
            },
            CliError::MissingTemplateAsset {
                template: "turn-based",
                file: "src/lib.rs".to_string(),
            },
            CliError::UnknownPiece {
                name: "nope".to_string(),
                available: "score, timer".to_string(),
            },
            CliError::InvalidProject {
                path: PathBuf::from("/tmp/not-a-project"),
            },
            CliError::PieceConflict {
                piece: "score".to_string(),
                files: vec![PathBuf::from("src/score.rs")],
            },
            CliError::MissingPieceAsset {
                piece: "score".to_string(),
                file: "score.rs".to_string(),
            },
        ]
    }

    #[test]
    fn hint_is_some_for_actionable_variants() {
        let cases: Vec<(CliError, &str)> = vec![
            (
                CliError::InvalidName {
                    name: "1bad".to_string(),
                    reason: "must start with a letter".to_string(),
                },
                "crate names start with a letter",
            ),
            (
                CliError::InvalidConfig {
                    reason: "board_width out of range".to_string(),
                },
                "board_width and board_height in 3..=8",
            ),
            (
                CliError::TargetExists {
                    path: PathBuf::from("/tmp/my-game"),
                    is_file: false,
                },
                "directory `/tmp/my-game`",
            ),
            (
                CliError::TargetExists {
                    path: PathBuf::from("/tmp/my-game"),
                    is_file: true,
                },
                "file `/tmp/my-game`",
            ),
            (
                CliError::MissingTemplateAsset {
                    template: "turn-based",
                    file: "src/lib.rs".to_string(),
                },
                "bug in cougr-cli",
            ),
            (
                CliError::UnknownPiece {
                    name: "nope".to_string(),
                    available: "score, timer".to_string(),
                },
                "cougr add --list",
            ),
            (
                CliError::InvalidProject {
                    path: PathBuf::from("/tmp/not-a-project"),
                },
                "cougr add",
            ),
            (
                CliError::MissingPieceAsset {
                    piece: "score".to_string(),
                    file: "score.rs".to_string(),
                },
                "bug in cougr-cli",
            ),
        ];

        for (err, needle) in cases {
            let hint = err.hint().unwrap_or_else(|| {
                panic!("expected a hint for {err:?}, got None");
            });
            assert!(
                hint.contains(needle),
                "hint for {err:?} was {hint:?}, expected it to contain {needle:?}"
            );
        }
    }

    #[test]
    fn hint_is_none_for_non_actionable_variants() {
        let cases = vec![
            CliError::Io {
                action: "write",
                path: PathBuf::from("/tmp/out.txt"),
                source: io_error(),
            },
            CliError::PieceConflict {
                piece: "score".to_string(),
                files: vec![PathBuf::from("src/score.rs")],
            },
        ];

        for err in cases {
            assert!(
                err.hint().is_none(),
                "expected no hint for {err:?}, got {:?}",
                err.hint()
            );
        }
    }

    #[test]
    fn every_variant_has_a_hint_decision() {
        // Table-driven guard: every variant must be classified as Some or None
        // by `hint()`. This keeps the match exhaustive as variants are added.
        for err in all_variants() {
            let _ = err.hint();
        }
    }

    #[test]
    fn display_messages_are_non_empty() {
        for err in all_variants() {
            let msg = err.to_string();
            assert!(!msg.is_empty(), "display for {err:?} was empty");
        }
    }

    #[test]
    fn io_variant_exposes_source() {
        let err = CliError::io("write", "/tmp/out.txt", io_error());
        assert!(std::error::Error::source(&err).is_some());
    }

    #[test]
    fn non_io_variants_have_no_source() {
        let err = CliError::InvalidConfig {
            reason: "bad".to_string(),
        };
        assert!(std::error::Error::source(&err).is_none());
    }
}
