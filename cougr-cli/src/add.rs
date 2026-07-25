//! Implementation of `cougr add [--list] [<piece>]`.
//!
//! ## Behaviour
//!
//! * `cougr add --list`   — prints all available pieces with one-line descriptions.
//! * `cougr add <piece>`  — copies a piece's files into `src/`, appends `mod`
//!   declarations to `lib.rs`, and reports what was written.
//!
//! ## Conflict handling
//!
//! If any target file already exists the command refuses to overwrite it,
//! prints what would have been written, and exits with a non-zero status.
//! This follows the shadcn/ui principle: you own the code, so the tool must
//! never silently clobber local edits.

use std::fs;
use std::path::{Path, PathBuf};

use crate::catalog::{self, Piece, PieceFile};

// ── Public entry points ───────────────────────────────────────────────────────

/// Run `cougr add --list`.
///
/// Prints all pieces in the catalog, one per line, with maturity tag.
pub fn run_list() -> anyhow::Result<()> {
    let pieces = catalog::load();
    println!("Available pieces ({}):\n", pieces.len());
    for p in &pieces {
        println!(
            "  {:<30}  [{}]  {}",
            p.name,
            maturity_label(&p.maturity),
            p.description
        );
    }
    println!();
    println!("Add a piece to the current project:");
    println!("  cougr add <piece-name>");
    Ok(())
}

/// Run `cougr add <piece>` in `project_root`.
///
/// * Resolves the piece by name.
/// * Checks for conflicts before writing anything.
/// * On conflict, prints a diff-style preview and returns an error.
/// * On success, writes all files and updates `lib.rs`.
pub fn run_add(piece_name: &str, project_root: &Path) -> anyhow::Result<()> {
    let pieces = catalog::load();
    let piece = catalog::find(&pieces, piece_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown piece '{piece_name}'.\nRun `cougr add --list` to see available pieces."
        )
    })?;

    let src_dir = project_root.join("src");
    if !src_dir.exists() {
        anyhow::bail!(
            "No 'src/' directory found in '{}'.\n\
             Make sure you are running this command from the root of a Cougr project.",
            project_root.display()
        );
    }

    // ── 1. Conflict check ──────────────────────────────────────────────────
    let conflicts: Vec<&PieceFile> = piece
        .files
        .iter()
        .filter(|f| src_dir.join(&f.target).exists())
        .collect();

    if !conflicts.is_empty() {
        eprintln!("error: the following files already exist and will not be overwritten:\n");
        for f in &conflicts {
            let path = src_dir.join(&f.target);
            eprintln!("  {}", path.display());
        }
        eprintln!("\nTo preview what would be written, inspect the cougr-cli pieces.toml:");
        for f in &conflicts {
            eprintln!(
                "\n  # target: src/{}\n{}",
                f.target,
                indent(&f.content, "  ")
            );
        }
        anyhow::bail!("Aborting: resolve conflicts before running `cougr add` again.");
    }

    // ── 2. Write files ─────────────────────────────────────────────────────
    let mut written: Vec<PathBuf> = Vec::new();
    for file in &piece.files {
        let dest = src_dir.join(&file.target);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&dest, file.content.as_bytes())?;
        written.push(dest);
    }

    // ── 3. Wire mod declarations into lib.rs ───────────────────────────────
    let lib_rs = src_dir.join("lib.rs");
    if lib_rs.exists() {
        wire_mod_declarations(&lib_rs, piece)?;
    }

    // ── 4. Report ──────────────────────────────────────────────────────────
    println!("✓ Added piece '{}'  [{}]", piece.name, piece.maturity);
    println!();
    for path in &written {
        println!("  wrote  {}", path.display());
    }
    if lib_rs.exists() {
        println!("  updated  {}", lib_rs.display());
    }

    if !piece.cargo_deps.is_empty() {
        println!();
        println!("Ensure these lines are present in your Cargo.toml:\n");
        let (deps, dev_deps): (Vec<_>, Vec<_>) =
            piece.cargo_deps.iter().partition(|d| !d.dev);
        if !deps.is_empty() {
            println!("[dependencies]");
            for d in &deps {
                println!("  {}", d.line);
            }
        }
        if !dev_deps.is_empty() {
            println!("[dev-dependencies]");
            for d in &dev_deps {
                println!("  {}", d.line);
            }
        }
        println!();
        println!("(cougr add does not modify Cargo.toml automatically to avoid");
        println!(" conflicting with workspace-level dependency management.)");
    }

    if piece.maturity == "experimental" {
        println!();
        println!(
            "⚠  Note: '{}' is Experimental — the API may change in minor versions.",
            piece.name
        );
    } else if piece.maturity == "beta" {
        println!();
        println!(
            "ℹ  Note: '{}' is Beta — the API is stable but may see refinements.",
            piece.name
        );
    }

    Ok(())
}

// ── Mod declaration wiring ────────────────────────────────────────────────────

/// Append `pub mod <module>;` (or `pub mod <parent> { pub mod <child>; }`) to
/// `lib.rs` for each file in the piece, without clobbering existing content.
///
/// Skips a declaration if an identical `mod <name>` line already appears in
/// the file — this is the idempotency guard for running `cougr add` twice.
fn wire_mod_declarations(lib_rs: &Path, piece: &Piece) -> anyhow::Result<()> {
    let original = fs::read_to_string(lib_rs)?;
    let mut additions: Vec<String> = Vec::new();

    for file in &piece.files {
        // e.g. "session_auth/mod.rs"  → top-level module "session_auth"
        //      "standards/pausable.rs" → sub-module: need "pub mod standards { pub mod pausable; }"
        let parts: Vec<&str> = file.target.trim_end_matches(".rs").split('/').collect();

        match parts.as_slice() {
            // "mod.rs" at top level: "session_auth/mod.rs" → pub mod session_auth;
            [module, "mod"] => {
                let decl = format!("pub mod {module};");
                if !original.contains(&decl) {
                    additions.push(decl);
                }
            }
            // single-level file: "session_auth.rs" → pub mod session_auth;
            [module] => {
                let decl = format!("pub mod {module};");
                if !original.contains(&decl) {
                    additions.push(decl);
                }
            }
            // two-level: "standards/pausable.rs" → pub mod standards { pub mod pausable; }
            // We don't create nested module blocks automatically because the
            // parent module might be a directory mod.  Instead we add both
            // declarations and leave it for the user to place them correctly.
            [parent, child] => {
                let parent_decl = format!("pub mod {parent};");
                let child_decl = format!("pub mod {child};");
                if !original.contains(&child_decl) {
                    // Write a comment block so the user knows exactly what to add.
                    let note = format!(
                        "// TODO(cougr add): wire the following inside src/{parent}/mod.rs:\n\
                         //   {child_decl}\n\
                         // Then add to lib.rs:\n\
                         //   {parent_decl}"
                    );
                    if !original.contains(&note) {
                        additions.push(note);
                    }
                }
            }
            _ => {
                // Deeper nesting — just leave a comment.
                let note = format!("// TODO(cougr add): manually wire module for {}", file.target);
                additions.push(note);
            }
        }
    }

    if additions.is_empty() {
        return Ok(());
    }

    let block = format!(
        "\n// ── Added by `cougr add {}` ──\n{}\n",
        piece.name,
        additions.join("\n")
    );
    let mut content = original;
    content.push_str(&block);
    fs::write(lib_rs, content.as_bytes())?;
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn maturity_label(maturity: &str) -> &str {
    match maturity {
        "stable" => "stable      ",
        "beta" => "beta        ",
        "experimental" => "experimental",
        other => other,
    }
}

fn indent(s: &str, prefix: &str) -> String {
    s.lines()
        .map(|l| format!("{prefix}{l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_project(dir: &TempDir) -> PathBuf {
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("lib.rs"), "#![no_std]\n\n").unwrap();
        dir.path().to_path_buf()
    }

    #[test]
    fn add_session_auth_creates_file() {
        let tmp = TempDir::new().unwrap();
        let root = make_project(&tmp);
        run_add("session-auth", &root).expect("add should succeed");

        let dest = root.join("src").join("session_auth").join("mod.rs");
        assert!(dest.exists(), "session_auth/mod.rs should be created");
        let content = fs::read_to_string(&dest).unwrap();
        assert!(
            content.contains("SessionAuth"),
            "file should contain SessionAuth"
        );
    }

    #[test]
    fn add_hidden_hand_creates_file() {
        let tmp = TempDir::new().unwrap();
        let root = make_project(&tmp);
        run_add("hidden-hand", &root).expect("add should succeed");

        let dest = root.join("src").join("hidden_hand").join("mod.rs");
        assert!(dest.exists(), "hidden_hand/mod.rs should be created");
    }

    #[test]
    fn add_standards_pausable_creates_file() {
        let tmp = TempDir::new().unwrap();
        let root = make_project(&tmp);
        run_add("standards/pausable", &root).expect("add should succeed");

        let dest = root.join("src").join("standards").join("pausable.rs");
        assert!(dest.exists(), "standards/pausable.rs should be created");
    }

    #[test]
    fn add_updates_lib_rs_with_mod_declaration() {
        let tmp = TempDir::new().unwrap();
        let root = make_project(&tmp);
        run_add("session-auth", &root).expect("add should succeed");

        let lib = fs::read_to_string(root.join("src").join("lib.rs")).unwrap();
        assert!(
            lib.contains("pub mod session_auth;"),
            "lib.rs should contain mod declaration; got:\n{lib}"
        );
    }

    #[test]
    fn double_add_is_refused() {
        let tmp = TempDir::new().unwrap();
        let root = make_project(&tmp);
        run_add("session-auth", &root).expect("first add should succeed");

        let result = run_add("session-auth", &root);
        assert!(
            result.is_err(),
            "second add should fail due to conflict detection"
        );
    }

    #[test]
    fn add_unknown_piece_gives_helpful_error() {
        let tmp = TempDir::new().unwrap();
        let root = make_project(&tmp);
        let err = run_add("not-a-real-piece", &root).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("Unknown piece"),
            "error should mention unknown piece; got: {msg}"
        );
        assert!(
            msg.contains("--list"),
            "error should suggest --list; got: {msg}"
        );
    }

    #[test]
    fn add_without_src_dir_gives_helpful_error() {
        let tmp = TempDir::new().unwrap();
        // No src/ directory created
        let err = run_add("session-auth", tmp.path()).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("src/"),
            "error should mention missing src/; got: {msg}"
        );
    }
}
