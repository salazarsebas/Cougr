//! Piece catalog — loaded from the `pieces.toml` file embedded in the binary
//! at compile time via `include_str!`.
//!
//! The catalog is parsed once on first access and is immutable for the lifetime
//! of the process.

use serde::Deserialize;

/// Raw TOML source embedded at compile time.
const CATALOG_TOML: &str = include_str!("../../pieces.toml");

// ── Data model ────────────────────────────────────────────────────────────────

/// A single file that will be written into the user's project.
#[derive(Debug, Deserialize)]
pub struct PieceFile {
    /// Destination path relative to the project's `src/` directory.
    pub target: String,
    /// Verbatim file content.
    pub content: String,
}

/// A dependency line to append to `Cargo.toml`.
#[derive(Debug, Deserialize)]
pub struct PieceDep {
    /// The exact line to append (e.g. `cougr-core = "1.1.0"`).
    pub line: String,
    /// If `true`, append to `[dev-dependencies]`; otherwise `[dependencies]`.
    #[serde(default)]
    pub dev: bool,
}

/// A single piece in the catalog.
#[derive(Debug, Deserialize)]
pub struct Piece {
    /// Unique CLI identifier — also used as the argument to `cougr add`.
    pub name: String,
    /// One-line description shown by `cougr add --list`.
    pub description: String,
    /// Maturity tier: "stable" | "beta" | "experimental".
    pub maturity: String,
    /// Canonical source path (informational only).
    #[allow(dead_code)]
    pub source: String,
    /// Files to write into `src/`.
    #[serde(default)]
    pub files: Vec<PieceFile>,
    /// Dependency lines to ensure are present in `Cargo.toml`.
    #[serde(default)]
    pub cargo_deps: Vec<PieceDep>,
}

/// The full catalog deserialized from `pieces.toml`.
#[derive(Debug, Deserialize)]
struct CatalogToml {
    piece: Vec<Piece>,
}

/// Load and parse the embedded catalog.
///
/// Panics at startup if `pieces.toml` is malformed — this is intentional;
/// a broken catalog is a compile-time/packaging bug, not a user error.
pub fn load() -> Vec<Piece> {
    let raw: CatalogToml =
        toml::from_str(CATALOG_TOML).expect("pieces.toml is malformed; this is a packaging bug");
    raw.piece
}

/// Look up a piece by name (case-insensitive for user convenience).
pub fn find<'a>(catalog: &'a [Piece], name: &str) -> Option<&'a Piece> {
    let needle = name.to_ascii_lowercase();
    catalog
        .iter()
        .find(|p| p.name.to_ascii_lowercase() == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_loads_successfully() {
        let pieces = load();
        assert!(!pieces.is_empty(), "catalog must contain at least one piece");
    }

    #[test]
    fn catalog_has_required_pieces() {
        let pieces = load();
        let names: Vec<&str> = pieces.iter().map(|p| p.name.as_str()).collect();
        assert!(
            names.contains(&"session-auth"),
            "catalog must contain session-auth"
        );
        assert!(
            names.contains(&"hidden-hand"),
            "catalog must contain hidden-hand"
        );
        assert!(
            names.contains(&"standards/pausable"),
            "catalog must contain standards/pausable"
        );
    }

    #[test]
    fn every_piece_has_at_least_one_file() {
        for piece in load() {
            assert!(
                !piece.files.is_empty(),
                "piece '{}' has no files",
                piece.name
            );
        }
    }

    #[test]
    fn find_is_case_insensitive() {
        let pieces = load();
        assert!(find(&pieces, "SESSION-AUTH").is_some());
        assert!(find(&pieces, "hidden-hand").is_some());
        assert!(find(&pieces, "does-not-exist").is_none());
    }
}
