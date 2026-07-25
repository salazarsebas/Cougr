//! `cougr new <name>` minimal scaffolding.
//!
//! Per issue #247 we only need a thin scaffolding here: the doctor pre-flight
//! is what `cougr new` is "really" doing in this issue. A fuller
//! template-driven scaffolder (with `--template starter|turn-based|...`)
//! tracks `docs/strategy/06-product-strategy.md` and lives behind a separate
//! epic.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// Plan or result of running [`scaffold`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScaffoldOutcome {
    /// The directory was created and populated. The wrapped path is the
    /// project root (relative or absolute, depending on the caller's CWD).
    Created(PathBuf),
    /// Nothing was written; this is the plan that would have been applied.
    DryRun(String),
}

/// Errors that `cougr new` may surface even when the doctor pre-flight passed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScaffoldError {
    /// Name violated a crate-identifier rule. The doctor pre-flight does NOT
    /// validate names; that belongs to the scaffolder.
    InvalidName(String),
    /// A filesystem write failed.
    Io { path: PathBuf, message: String },
}

impl fmt::Display for ScaffoldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScaffoldError::InvalidName(reason) => write!(f, "invalid project name: {reason}"),
            ScaffoldError::Io { path, message } => {
                write!(f, "could not write {}: {}", path.display(), message)
            }
        }
    }
}

impl std::error::Error for ScaffoldError {}

/// Validate a Rust crate identifier with the same rules Cargo applies.
/// Allowing underscores and hyphens has the same effect as Cargo's own loader
/// in practice: hyphens are mapped to underscores in the lib name.
pub fn validate_name(name: &str) -> Result<(), ScaffoldError> {
    if name.is_empty() {
        return Err(ScaffoldError::InvalidName("name is empty".into()));
    }
    if name.len() > 64 {
        return Err(ScaffoldError::InvalidName(
            "name is longer than 64 characters",
        ));
    }
    let first = name.chars().next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return Err(ScaffoldError::InvalidName(
            "first character must be ASCII alphabetic or '_'",
        ));
    }
    for c in name.chars() {
        if !(c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return Err(ScaffoldError::InvalidName(format!(
                "character `{}` is not allowed",
                c
            )));
        }
    }
    Ok(())
}

/// Template bodies live inline so the scaffolder has zero IO on the workspace
/// at run time. Updating them is a one-line change per template file. The
/// dependency on `cougr-core` is published-crate rather than a path dep so
/// `cougr new` works for users outside the `salazarsebas/Cougr` workspace.
/// Contributors iterating inside the workspace manually swap to
/// `path = "../../"`.
const CARGO_TOML_TEMPLATE: &str = r#"[package]
name = "{{name}}"
version = "0.1.0"
edition = "2021"
description = "A new Soroban game built on cougr-core"
license = "MIT"
publish = false
rust-version = "1.70.0"

[lib]
crate-type = ["cdylib"]

[features]
default = []
testutils = ["cougr-core/testutils"]

[dependencies]
cougr-core = "1.1"
soroban-sdk = "25.1.0"

[dev-dependencies]
soroban-sdk = { version = "25.1.0", features = ["testutils"] }

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
overflow-checks = true
"#;

const LIB_RS_TEMPLATE

const LIB_RS_TEMPLATE: &str = r#"#![no_std]

use cougr_core::prelude::*;
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct {{name_pascal}};

#[contractimpl]
impl {{name_pascal}} {
    pub fn hello(env: Env) -> u32 {
        let _ = env;
        42
    }
}
"#;

const GITIGNORE_TEMPLATE: &str = "target\n**/*.rs.bk\nCargo.lock\ntest_snapshots/\n";

const README_TEMPLATE: &str = r#"# {{name}}

A new Soroban game scaffolded by `cougr`.

## Build

```
cargo build --target wasm32v1-none --release
```

## Test

```
cargo test
```

## Deploy

```
stellar contract build
stellar contract deploy --wasm target/wasm32v1-none/release/{{name_snake}}.wasm
```

For more, see https://github.com/salazarsebas/Cougr.
"#;

/// Build a complete scaffold under `./<name>/` from the current working
/// directory. `dry_run` formats what would be written without touching disk.
pub fn scaffold(name: &str, dry_run: bool) -> Result<ScaffoldOutcome, ScaffoldError> {
    validate_name(name)?;

    let project_root = PathBuf::from(name);
    let cargo_toml_path = project_root.join("Cargo.toml");
    let lib_rs_path = project_root.join("src").join("lib.rs");
    let gitignore_path = project_root.join(".gitignore");
    let readme_path = project_root.join("README.md");

    let name_pascal = pascalize(name);
    let name_snake = snake_case(name);

    let cargo_toml = CARGO_TOML_TEMPLATE.replace("{{name}}", name);
    let lib_rs = LIB_RS_TEMPLATE
        .replace("{{name}}", name)
        .replace("{{name_pascal}}", &name_pascal);
    let readme = README_TEMPLATE
        .replace("{{name}}", name)
        .replace("{{name_snake}}", &name_snake);

    // One source of truth for the file list. Both branches walk the same
    // vector so adding a file only requires adding one tuple here.
    let files = [
        ("Cargo.toml", cargo_toml),
        ("src/lib.rs", lib_rs),
        (".gitignore", GITIGNORE_TEMPLATE.to_string()),
        ("README.md", readme),
    ];

    if dry_run {
        let mut plan = String::new();
        plan.push_str(&format!("[dry-run] would create {} file(s):\n", files.len()));
        for (relpath, _) in &files {
            plan.push_str(&format!("  - {relpath}\n"));
        }
        for (relpath, body) in &files {
            plan.push('\n');
            plan.push_str(&format!("--- {relpath} ---\n"));
            plan.push_str(body);
        }
        return Ok(ScaffoldOutcome::DryRun(plan));
    }

    for (relpath, body) in &files {
        write_file(&project_root.join(relpath), body)?;
    }

    Ok(ScaffoldOutcome::Created(project_root))
}

fn write_file(path: &Path, body: &str) -> Result<(), ScaffoldError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| ScaffoldError::Io {
            path: parent.to_path_buf(),
            message: e.to_string(),
        })?;
    }
    fs::write(path, body).map_err(|e| ScaffoldError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}

fn pascalize(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut upper_next = true;
    for c in name.chars() {
        if c == '_' || c == '-' {
            upper_next = true;
            continue;
        }
        if upper_next {
            out.extend(c.to_uppercase());
            upper_next = false;
        } else {
            out.push(c);
        }
    }
    out
}

fn snake_case(name: &str) -> String {
    name.replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_name_accepts_normal_crate_names() {
        assert!(validate_name("my_game").is_ok());
        assert!(validate_name("my-game").is_ok());
        assert!(validate_name("_underscored").is_ok());
        assert!(validate_name("abc123").is_ok());
    }

    #[test]
    fn validate_name_rejects_empty() {
        assert!(validate_name("").is_err());
    }

    #[test]
    fn validate_name_rejects_too_long() {
        let s = "a".repeat(65);
        assert!(validate_name(&s).is_err());
    }

    #[test]
    fn validate_name_rejects_leading_digit() {
        assert!(validate_name("1game").is_err());
    }

    #[test]
    fn validate_name_rejects_invalid_chars() {
        assert!(validate_name("hello world").is_err());
        assert!(validate_name("hello!").is_err());
    }

    #[test]
    fn pascalize_basic() {
        assert_eq!(pascalize("my_game"), "MyGame");
        assert_eq!(pascalize("my-game"), "MyGame");
        assert_eq!(pascalize("hello"), "Hello");
        assert_eq!(pascalize("a"), "A");
    }

    #[test]
    fn scaffold_dry_run_does_not_write_files() {
        let outcome = scaffold("demo_game", true).unwrap();
        match outcome {
            ScaffoldOutcome::DryRun(plan) => {
                assert!(plan.contains("Cargo.toml"));
                assert!(plan.contains("src/lib.rs"));
                assert!(plan.contains("cougr-core = \"1.1\""));
                assert!(plan.contains("MyGame"));
                assert!(plan.contains("demo_game"));
            }
            ScaffoldOutcome::Created(_) => panic!("dry_run produced Created"),
        }
    }

    #[test]
    fn scaffold_rejects_invalid_name() {
        let err = scaffold("1bad", false).unwrap_err();
        match err {
            ScaffoldError::InvalidName(_) => {}
            other => panic!("wrong error: {other:?}"),
        }
    }

    #[test]
    fn scaffold_writes_files_in_tempdir() {
        // Use a unique nested tempdir + a project name unique per test run so
        // parallel `cargo test` invocations do not stomp each other.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let project_name = format!("demo_game_{nanos}");
        let original = std::env::current_dir().unwrap();
        let scratch =
            std::env::temp_dir().join(format!("cougr-cli-scratch-{nanos}"));
        std::fs::create_dir_all(&scratch).unwrap();
        std::env::set_current_dir(&scratch).unwrap();

        let result = scaffold(&project_name, false);
        let project = scratch.join(&project_name);

        // Restore CWD BEFORE asserting so we can also clean up on success.
        std::env::set_current_dir(&original).unwrap();
        let cargo_toml_exists = project.join("Cargo.toml").is_file();
        let lib_rs_exists = project.join("src").join("lib.rs").is_file();
        let _ = std::fs::remove_dir_all(&project);
        let _ = std::fs::remove_dir_all(&scratch);

        assert!(result.is_ok(), "{:?}", result);
        assert!(cargo_toml_exists, "Cargo.toml was not created");
        assert!(lib_rs_exists, "src/lib.rs was not created");
    }
}
