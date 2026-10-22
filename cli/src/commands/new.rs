//! `cougr new <name> [--template <name>]`.
//!
//! Renders one embedded template into a fresh directory and prints the two
//! commands that take the developer from generated source to a green test run
//! and a WASM artifact.

use std::fs;
use std::path::Path;

use crate::error::CliError;
use crate::name::ProjectName;
use crate::template::{RenderedFile, Template, TurnBasedConfig};

pub fn run(raw_name: &str, template: Template, parent: Option<&Path>) -> Result<(), CliError> {
    run_with_config(raw_name, template, None, parent)
}

/// Export a studio config through the same renderer and writer as `cougr new`.
pub fn export(raw_name: &str, config_path: &Path, parent: Option<&Path>) -> Result<(), CliError> {
    let source = fs::read_to_string(config_path)
        .map_err(|err| CliError::io("read the config", config_path, err))?;
    let config: TurnBasedConfig =
        serde_json::from_str(&source).map_err(|err| CliError::InvalidConfig {
            reason: err.to_string(),
        })?;
    run_with_config(
        raw_name,
        Template::TurnBased,
        Some(config.validate()?),
        parent,
    )
}

fn run_with_config(
    raw_name: &str,
    template: Template,
    config: Option<TurnBasedConfig>,
    parent: Option<&Path>,
) -> Result<(), CliError> {
    let name = ProjectName::parse(raw_name)?;

    let target_parent = match parent {
        Some(dir) => dir.to_path_buf(),
        None => std::env::current_dir()
            .map_err(|err| CliError::io("resolve the current directory", ".", err))?,
    };
    let target = target_parent.join(name.crate_name());

    if target.exists() {
        return Err(CliError::TargetExists {
            is_file: target.is_file(),
            path: target,
        });
    }

    // Non-fatal environment advisory: warn before writing the project tree so
    // usage errors (bad name, target exists) stay free of doctor output.
    super::doctor::run_as_warning();

    let files = match config {
        Some(config) => template.render_with_config(&name, Some(config))?,
        None => template.render(&name)?,
    };

    fs::create_dir_all(&target)
        .map_err(|err| CliError::io("create the project directory", &target, err))?;

    // A half-written project is worse than none: on any write failure, remove
    // the tree so a retry does not trip the "already exists" check above.
    if let Err(err) = write_all(&target, &files) {
        let _ = fs::remove_dir_all(&target);
        return Err(err);
    }

    report_success(
        &target,
        name.crate_name(),
        template,
        &files,
        parent.is_some(),
    );
    Ok(())
}

fn write_all(target: &Path, files: &[RenderedFile]) -> Result<(), CliError> {
    for file in files {
        let path = target.join(&file.path);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)
                .map_err(|err| CliError::io("create the directory", dir, err))?;
        }
        fs::write(&path, &file.contents)
            .map_err(|err| CliError::io("write the file", &path, err))?;
    }
    Ok(())
}

fn report_success(
    target: &Path,
    crate_name: &str,
    template: Template,
    files: &[RenderedFile],
    has_path: bool,
) {
    println!(
        "Created `{crate_name}` from the `{}` template (based on examples/{}).",
        template.id(),
        template.source_example()
    );
    println!();
    println!("  {}/", target.display());
    for file in files {
        println!("    {}", file.path);
    }
    println!();
    println!("{}", next_steps(target, crate_name, has_path));
    println!();
    println!("`stellar contract build` needs the Stellar CLI and the wasm32v1-none target:");
    println!("  rustup target add wasm32v1-none");
    println!("  cargo install --locked stellar-cli");
}

pub(crate) fn next_steps(target: &Path, crate_name: &str, has_path: bool) -> String {
    let cd_target = format_cd_path(target, crate_name, has_path);
    format!(
        "Next steps:\n  cd {cd_target}\n  cargo test                # run the GameHarness test suite\n  stellar contract build    # compile the contract to WASM"
    )
}

pub(crate) fn format_cd_path(target: &Path, crate_name: &str, has_path: bool) -> String {
    if !has_path {
        return crate_name.to_string();
    }

    let path_str = if let Ok(cwd) = std::env::current_dir() {
        let norm_cwd = normalize_path(&cwd);
        let abs_target = if target.is_absolute() {
            normalize_path(target)
        } else {
            normalize_path(&norm_cwd.join(target))
        };

        if let Ok(rel) = abs_target.strip_prefix(&norm_cwd) {
            if rel.as_os_str().is_empty() {
                ".".to_string()
            } else {
                rel.display().to_string()
            }
        } else {
            abs_target.display().to_string()
        }
    } else {
        target.display().to_string()
    };

    if path_str.contains(' ') {
        format!("\"{path_str}\"")
    } else {
        path_str
    }
}

fn normalize_path(path: &Path) -> std::path::PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if let Some(std::path::Component::Normal(_)) = components.last() {
                    components.pop();
                } else {
                    components.push(component);
                }
            }
            c => components.push(c),
        }
    }
    components.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tempdir() -> tempfile::TempDir {
        tempfile::tempdir().expect("create temp dir")
    }

    #[test]
    fn generates_the_canonical_layout_on_disk() {
        let dir = tempdir();
        run("demo", Template::Starter, Some(dir.path())).unwrap();

        let project = dir.path().join("demo");
        for expected in [
            "Cargo.toml",
            "README.md",
            ".gitignore",
            "src/lib.rs",
            "src/components.rs",
            "src/systems.rs",
            "src/test.rs",
        ] {
            assert!(project.join(expected).is_file(), "missing {expected}");
        }
    }

    #[test]
    fn substitutes_the_project_name_into_the_manifest_and_lib() {
        let dir = tempdir();
        run("tower-siege", Template::TurnBased, Some(dir.path())).unwrap();

        let project = dir.path().join("tower-siege");
        let manifest = fs::read_to_string(project.join("Cargo.toml")).unwrap();
        assert!(manifest.contains("name = \"tower-siege\""));

        let lib = fs::read_to_string(project.join("src/lib.rs")).unwrap();
        assert!(lib.contains("pub struct TowerSiege;"));
        assert!(lib.contains("tower-siege"), "module doc names the project");
    }

    #[test]
    fn refuses_to_overwrite_an_existing_directory() {
        let dir = tempdir();
        fs::create_dir(dir.path().join("demo")).unwrap();

        let err = run("demo", Template::Starter, Some(dir.path())).unwrap_err();
        assert!(matches!(err, CliError::TargetExists { is_file: false, .. }));
        let msg = err.to_string();
        assert!(
            msg.contains("directory"),
            "message should say 'directory': {msg}"
        );
        let hint = err.hint().unwrap();
        assert!(
            hint.contains("directory"),
            "hint should say 'directory': {hint}"
        );
    }

    #[test]
    fn refuses_to_overwrite_an_existing_file() {
        let dir = tempdir();
        fs::write(dir.path().join("demo"), b"artifact").unwrap();

        let err = run("demo", Template::Starter, Some(dir.path())).unwrap_err();
        assert!(matches!(err, CliError::TargetExists { is_file: true, .. }));
        let msg = err.to_string();
        assert!(msg.contains("file"), "message should say 'file': {msg}");
        assert!(
            !msg.contains("directory"),
            "message must not say 'directory': {msg}"
        );
        let hint = err.hint().unwrap();
        assert!(hint.contains("file"), "hint should say 'file': {hint}");
        assert!(
            !hint.contains("directory"),
            "hint must not say 'directory': {hint}"
        );
    }

    #[test]
    fn rejects_an_invalid_name_before_touching_the_filesystem() {
        let dir = tempdir();
        let err = run("../escape", Template::Starter, Some(dir.path())).unwrap_err();

        assert!(matches!(err, CliError::InvalidName { .. }));
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    #[test]
    fn every_template_writes_a_complete_project() {
        for template in [
            Template::Starter,
            Template::TurnBased,
            Template::HiddenInfo,
            Template::SessionAuth,
        ] {
            let dir = tempdir();
            run("demo", template, Some(dir.path())).unwrap();
            let project = dir.path().join("demo");

            let manifest = fs::read_to_string(project.join("Cargo.toml")).unwrap();
            assert!(
                manifest.contains("cougr-core = \""),
                "{} manifest must depend on cougr-core",
                template.id()
            );
            assert!(
                fs::read_to_string(project.join("src/test.rs"))
                    .unwrap()
                    .contains("GameHarness"),
                "{} must ship a GameHarness test",
                template.id()
            );
        }
    }

    #[test]
    fn next_steps_without_path_uses_bare_crate_name() {
        let dir = tempdir();
        let target = dir.path().join("my-game");
        let text = next_steps(&target, "my-game", false);
        assert!(
            text.contains("  cd my-game\n"),
            "expected 'cd my-game' without --path, got:\n{text}"
        );
    }

    #[test]
    fn next_steps_with_path_uses_target_path() {
        let dir = tempdir();
        let target = dir.path().join("my-game");
        let text = next_steps(&target, "my-game", true);
        let expected = format!("  cd {}\n", target.display());
        assert!(
            text.contains(&expected),
            "expected '{expected}' with --path, got:\n{text}"
        );
    }

    #[test]
    fn next_steps_with_space_in_path_quotes_target() {
        let dir = tempdir();
        let space_dir = dir.path().join("my games");
        let target = space_dir.join("my-game");
        let text = next_steps(&target, "my-game", true);
        let expected = format!("  cd \"{}\"\n", target.display());
        assert!(
            text.contains(&expected),
            "expected '{expected}' for path with spaces, got:\n{text}"
        );
    }

    #[test]
    fn next_steps_with_relative_path_under_cwd() {
        let rel_target = Path::new("sub/my-game");
        let text = next_steps(rel_target, "my-game", true);
        assert!(
            text.contains("  cd sub/my-game\n"),
            "expected relative path 'cd sub/my-game', got:\n{text}"
        );
    }

    #[test]
    fn next_steps_with_relative_path_containing_space() {
        let rel_target = Path::new("my games/my-game");
        let text = next_steps(rel_target, "my-game", true);
        assert!(
            text.contains("  cd \"my games/my-game\"\n"),
            "expected quoted relative path 'cd \"my games/my-game\"', got:\n{text}"
        );
    }
}
