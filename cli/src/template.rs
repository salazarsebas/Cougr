//! The embedded template layer.
//!
//! The four templates are derived from the canonical examples named in
//! `examples/EXAMPLE_STANDARD.md` and are compiled into the binary with
//! `rust-embed`, so `cougr new` works with no network access and no checkout of
//! this repository.
//!
//! Template files carry two spelling conventions that [`output_path`] undoes on
//! write:
//!
//! * `Cargo.toml.tmpl` → `Cargo.toml`, because Cargo skips subdirectories that
//!   contain a manifest when packaging a crate, which would drop the template
//!   manifests from the published `cougr-cli`.
//! * `gitignore` → `.gitignore`, because a real dotfile in `templates/` would be
//!   applied to this repository rather than shipped as content.

use std::borrow::Cow;

use clap::ValueEnum;
use rust_embed::RustEmbed;

use crate::error::CliError;
use crate::name::ProjectName;

#[derive(RustEmbed)]
#[folder = "templates/"]
struct Assets;

/// The `cougr-core` release generated projects depend on, pinned to the current
/// major/minor. `cougr-cli` and `cougr-core` share a release cadence, so this is
/// this crate's own `major.minor`; the `core_version_tracks_cli_version` test
/// fails if the two ever drift apart.
pub const COUGR_CORE_VERSION: &str = "1.1";

/// The `soroban-sdk` release every canonical example is validated against.
pub const SOROBAN_SDK_VERSION: &str = "25.1.0";

/// A curated starting point, each backed by one canonical example.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Template {
    /// Spawn entities and move them around a 2D world (from `spawn_and_move`).
    Starter,
    /// Two-player turn-based board game (from `tic_tac_toe`).
    TurnBased,
    /// Hidden information verified with ZK proofs (from `hidden_hand`).
    HiddenInfo,
    /// Approve a session once, then play without wallet prompts (from `session_arena`).
    SessionAuth,
}

impl Template {
    /// Directory name inside `templates/`, and the value accepted by `--template`.
    pub fn id(self) -> &'static str {
        match self {
            Template::Starter => "starter",
            Template::TurnBased => "turn-based",
            Template::HiddenInfo => "hidden-info",
            Template::SessionAuth => "session-auth",
        }
    }

    /// The canonical example this template was derived from.
    pub fn source_example(self) -> &'static str {
        match self {
            Template::Starter => "spawn_and_move",
            Template::TurnBased => "tic_tac_toe",
            Template::HiddenInfo => "hidden_hand",
            Template::SessionAuth => "session_arena",
        }
    }

    /// One-line summary, reused as the generated crate's `description`.
    pub fn summary(self) -> &'static str {
        match self {
            Template::Starter => "spawn entities and move them around a 2D world on Soroban",
            Template::TurnBased => "a two-player turn-based board game on Soroban",
            Template::HiddenInfo => "a hidden-information game with ZK-verified deals on Soroban",
            Template::SessionAuth => "a session-key game loop with no per-move wallet prompts",
        }
    }

    /// Every file in this template, sorted, as `(output path, rendered bytes)`.
    pub fn render(self, name: &ProjectName) -> Result<Vec<RenderedFile>, CliError> {
        let prefix = format!("{}/", self.id());

        let mut sources: Vec<Cow<'static, str>> = Assets::iter()
            .filter(|path| path.starts_with(&prefix))
            .collect();
        sources.sort();

        if sources.is_empty() {
            return Err(CliError::MissingTemplateAsset {
                template: self.id(),
                file: format!("{prefix}*"),
            });
        }

        let mut files = Vec::with_capacity(sources.len());
        for source in sources {
            let asset = Assets::get(&source).ok_or_else(|| CliError::MissingTemplateAsset {
                template: self.id(),
                file: source.to_string(),
            })?;

            let relative = source
                .strip_prefix(&prefix)
                .expect("filtered on this prefix above");

            files.push(RenderedFile {
                path: output_path(relative),
                contents: self.substitute(&String::from_utf8_lossy(&asset.data), name),
            });
        }

        Ok(files)
    }

    /// Replace every `{{placeholder}}` a template may contain.
    fn substitute(self, body: &str, name: &ProjectName) -> String {
        body.replace("{{crate_name}}", name.crate_name())
            .replace("{{module_name}}", name.module_name())
            .replace("{{ContractName}}", name.type_name())
            .replace("{{description}}", self.summary())
            .replace("{{template_id}}", self.id())
            .replace("{{source_example}}", self.source_example())
            .replace("{{cougr_core_version}}", COUGR_CORE_VERSION)
            .replace("{{soroban_sdk_version}}", SOROBAN_SDK_VERSION)
    }
}

/// A single file to write into the generated project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedFile {
    /// Path relative to the project root, using `/` separators.
    pub path: String,
    pub contents: String,
}

/// Map a template-relative path to its path in the generated project.
fn output_path(relative: &str) -> String {
    let stripped = relative.strip_suffix(".tmpl").unwrap_or(relative);
    match stripped.rsplit_once('/') {
        Some((dir, "gitignore")) => format!("{dir}/.gitignore"),
        None if stripped == "gitignore" => ".gitignore".to_string(),
        _ => stripped.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Template; 4] = [
        Template::Starter,
        Template::TurnBased,
        Template::HiddenInfo,
        Template::SessionAuth,
    ];

    /// Truncate a semver string to its `major.minor` prefix.
    fn major_minor(version: &str) -> &str {
        match version.match_indices('.').nth(1) {
            Some((idx, _)) => &version[..idx],
            None => version,
        }
    }

    #[test]
    fn core_version_tracks_cli_version() {
        assert_eq!(COUGR_CORE_VERSION, major_minor(env!("CARGO_PKG_VERSION")));
        assert_eq!(major_minor("2.0.13"), "2.0");
        assert_eq!(major_minor("10.24.0-rc.1"), "10.24");
    }

    #[test]
    fn manifests_and_dotfiles_are_renamed_on_write() {
        assert_eq!(output_path("Cargo.toml.tmpl"), "Cargo.toml");
        assert_eq!(output_path("gitignore"), ".gitignore");
        assert_eq!(output_path("src/lib.rs"), "src/lib.rs");
        assert_eq!(output_path("nested/gitignore"), "nested/.gitignore");
    }

    #[test]
    fn every_template_ships_the_canonical_layout() {
        let name = ProjectName::parse("demo").unwrap();
        for template in ALL {
            let paths: Vec<_> = template
                .render(&name)
                .unwrap()
                .into_iter()
                .map(|f| f.path)
                .collect();
            for required in [
                "Cargo.toml",
                "README.md",
                ".gitignore",
                "src/lib.rs",
                "src/components.rs",
                "src/systems.rs",
                "src/test.rs",
            ] {
                assert!(
                    paths.iter().any(|p| p == required),
                    "template `{}` is missing `{required}` (has {paths:?})",
                    template.id()
                );
            }
        }
    }

    #[test]
    fn no_placeholder_survives_rendering() {
        let name = ProjectName::parse("demo").unwrap();
        for template in ALL {
            for file in template.render(&name).unwrap() {
                assert!(
                    !file.contents.contains("{{"),
                    "template `{}` left a placeholder in `{}`",
                    template.id(),
                    file.path
                );
            }
        }
    }

    #[test]
    fn generated_manifests_use_the_published_crate() {
        let name = ProjectName::parse("my-game").unwrap();
        for template in ALL {
            let manifest = template
                .render(&name)
                .unwrap()
                .into_iter()
                .find(|f| f.path == "Cargo.toml")
                .expect("every template has a manifest");

            assert!(manifest.contents.contains("name = \"my-game\""));
            assert!(
                manifest
                    .contents
                    .contains(&format!("cougr-core = \"{COUGR_CORE_VERSION}\"")),
                "`{}` must depend on the published cougr-core",
                template.id()
            );
            assert!(
                !manifest.contents.contains("path ="),
                "`{}` must not use a path dependency",
                template.id()
            );
        }
    }

    #[test]
    fn generated_tests_use_the_game_harness() {
        let name = ProjectName::parse("demo").unwrap();
        for template in ALL {
            let test = template
                .render(&name)
                .unwrap()
                .into_iter()
                .find(|f| f.path == "src/test.rs")
                .expect("every template has a test module");
            assert!(
                test.contents.contains("cougr_core::test::GameHarness"),
                "`{}` must test through GameHarness",
                template.id()
            );
        }
    }

    #[test]
    fn contract_type_name_follows_the_project_name() {
        let name = ProjectName::parse("dungeon-crawl").unwrap();
        let lib = Template::Starter
            .render(&name)
            .unwrap()
            .into_iter()
            .find(|f| f.path == "src/lib.rs")
            .unwrap();
        assert!(lib.contents.contains("pub struct DungeonCrawl;"));
    }

    #[test]
    fn template_ids_are_stable() {
        assert_eq!(
            ALL.map(Template::id),
            ["starter", "turn-based", "hidden-info", "session-auth"]
        );
    }
}
