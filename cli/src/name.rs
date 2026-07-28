//! Project-name validation and the case conversions the templates need.
//!
//! A single user-supplied name drives three different identifiers in the
//! generated crate: the Cargo package name, the Rust module/library name, and
//! the `#[contract]` struct name. [`ProjectName`] derives all three once so the
//! rest of the CLI never has to re-guess them.

use crate::error::CliError;

/// Rust keywords that cannot be used as a library target name. `cargo` rejects
/// these outright, so catching them here turns a confusing downstream build
/// failure into an immediate, readable error.
const RUST_KEYWORDS: &[&str] = &[
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
    "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in",
    "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "self", "static", "struct", "super", "trait", "true", "try", "type", "typeof",
    "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

/// Names Cargo refuses outright (`test` collides with the built-in test crate)
/// or that would shadow the framework the generated project depends on.
const RESERVED_NAMES: &[&str] = &["cougr", "cougr-core", "cougr_core", "cougr-cli", "test"];

/// The longest name crates.io accepts.
const MAX_NAME_LEN: usize = 64;

/// A validated project name plus its derived spellings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectName {
    /// The name exactly as typed — used for the Cargo package and directory.
    crate_name: String,
    /// `snake_case` form — the Rust library/module identifier.
    module_name: String,
    /// `PascalCase` form — the `#[contract]` struct identifier.
    type_name: String,
}

impl ProjectName {
    /// Validate `input` and derive the module and type spellings from it.
    pub fn parse(input: &str) -> Result<Self, CliError> {
        let invalid = |reason: &str| CliError::InvalidName {
            name: input.to_string(),
            reason: reason.to_string(),
        };

        if input.is_empty() {
            return Err(invalid("it is empty"));
        }
        if input.len() > MAX_NAME_LEN {
            return Err(invalid(&format!(
                "it is {} characters long, the maximum is {MAX_NAME_LEN}",
                input.len()
            )));
        }

        let first = input.chars().next().expect("checked non-empty above");
        if !first.is_ascii_alphabetic() {
            return Err(invalid("it must start with an ASCII letter"));
        }

        if let Some(bad) = input
            .chars()
            .find(|c| !c.is_ascii_alphanumeric() && *c != '_' && *c != '-')
        {
            return Err(invalid(&format!("`{bad}` is not allowed in a crate name")));
        }

        let module_name = input.replace('-', "_").to_ascii_lowercase();

        if RUST_KEYWORDS.contains(&module_name.as_str()) {
            return Err(invalid("it is a Rust keyword"));
        }
        let lowered = input.to_ascii_lowercase();
        if RESERVED_NAMES.contains(&lowered.as_str()) {
            return Err(invalid("that name is reserved and cannot be a crate name"));
        }

        Ok(ProjectName {
            type_name: pascal_case(&module_name),
            crate_name: input.to_string(),
            module_name,
        })
    }

    /// Cargo package name and generated directory name.
    pub fn crate_name(&self) -> &str {
        &self.crate_name
    }

    /// `snake_case` Rust identifier for the library target.
    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    /// `PascalCase` Rust identifier for the `#[contract]` struct.
    pub fn type_name(&self) -> &str {
        &self.type_name
    }
}

/// Convert a `snake_case` identifier to `PascalCase`.
fn pascal_case(snake: &str) -> String {
    let mut out = String::with_capacity(snake.len());
    for word in snake.split('_').filter(|w| !w.is_empty()) {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_all_three_spellings() {
        let name = ProjectName::parse("dungeon-crawl").unwrap();
        assert_eq!(name.crate_name(), "dungeon-crawl");
        assert_eq!(name.module_name(), "dungeon_crawl");
        assert_eq!(name.type_name(), "DungeonCrawl");
    }

    #[test]
    fn single_word_names_are_capitalised() {
        let name = ProjectName::parse("demo").unwrap();
        assert_eq!(name.module_name(), "demo");
        assert_eq!(name.type_name(), "Demo");
    }

    #[test]
    fn mixed_case_input_is_normalised() {
        let name = ProjectName::parse("MyGame").unwrap();
        assert_eq!(name.crate_name(), "MyGame");
        assert_eq!(name.module_name(), "mygame");
        assert_eq!(name.type_name(), "Mygame");
    }

    #[test]
    fn digits_are_allowed_after_the_first_character() {
        let name = ProjectName::parse("arena2").unwrap();
        assert_eq!(name.type_name(), "Arena2");
    }

    #[test]
    fn rejects_empty_name() {
        assert!(matches!(
            ProjectName::parse(""),
            Err(CliError::InvalidName { .. })
        ));
    }

    #[test]
    fn rejects_leading_digit() {
        assert!(ProjectName::parse("2fast").is_err());
    }

    #[test]
    fn rejects_path_separators_and_spaces() {
        assert!(ProjectName::parse("../escape").is_err());
        assert!(ProjectName::parse("my game").is_err());
        assert!(ProjectName::parse("a/b").is_err());
    }

    #[test]
    fn rejects_rust_keywords() {
        assert!(ProjectName::parse("move").is_err());
        assert!(ProjectName::parse("crate").is_err());
    }

    #[test]
    fn rejects_reserved_names() {
        assert!(ProjectName::parse("cougr").is_err());
        assert!(ProjectName::parse("cougr-core").is_err());
    }

    #[test]
    fn rejects_overlong_names() {
        assert!(ProjectName::parse(&"a".repeat(MAX_NAME_LEN + 1)).is_err());
    }
}
