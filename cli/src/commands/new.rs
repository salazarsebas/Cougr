//! `cougr new <name> [--template <name>]`.
//!
//! Renders one embedded template into a fresh directory and prints the two
//! commands that take the developer from generated source to a green test run
//! and a WASM artifact.

use std::fs;
use std::path::Path;

use crate::error::CliError;
use crate::name::ProjectName;
use crate::template::{RenderedFile, Template};

pub fn run(raw_name: &str, template: Template, parent: Option<&Path>) -> Result<(), CliError> {
    let name = ProjectName::parse(raw_name)?;

    let parent = match parent {
        Some(dir) => dir.to_path_buf(),
        None => std::env::current_dir()
            .map_err(|err| CliError::io("resolve the current directory", ".", err))?,
    };
    let target = parent.join(name.crate_name());

    if target.exists() {
        return Err(CliError::TargetExists { path: target });
    }

    let files = template.render(&name)?;

    fs::create_dir_all(&target)
        .map_err(|err| CliError::io("create the project directory", &target, err))?;

    // A half-written project is worse than none: on any write failure, remove
    // the tree so a retry does not trip the "already exists" check above.
    if let Err(err) = write_all(&target, &files) {
        let _ = fs::remove_dir_all(&target);
        return Err(err);
    }

    report_success(&target, name.crate_name(), template, &files);
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

fn report_success(target: &Path, crate_name: &str, template: Template, files: &[RenderedFile]) {
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
    println!("Next steps:");
    println!("  cd {crate_name}");
    println!("  cargo test                # run the GameHarness test suite");
    println!("  stellar contract build    # compile the contract to WASM");
    println!();
    println!("`stellar contract build` needs the Stellar CLI and the wasm32v1-none target:");
    println!("  rustup target add wasm32v1-none");
    println!("  cargo install --locked stellar-cli");
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
        assert!(matches!(err, CliError::TargetExists { .. }));
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
}
