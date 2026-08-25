//! `cougr` — command-line tooling for the Cougr ECS framework.
//!
//! Exposes four commands:
//!
//! - [`cougr new`]    — scaffold a Soroban game contract wired to `cougr-core`
//!                      from one of four embedded templates.
//! - [`cougr add`]    — add a cougr-core capability to an existing project
//!                      (not yet implemented).
//! - [`cougr check`]  — run repository hygiene checks against `examples/`, or
//!                      with `--verified`, the full canonical-quality checklist
//!                      for the "Cougr Verified" badge.
//! - [`cougr doctor`] — diagnose the local environment for Cougr development
//!                      (not yet implemented).

mod check;
mod commands;
mod context;
mod error;
mod name;
mod template;
mod verify;

use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::error::CliError;
use crate::template::Template;

#[derive(Parser)]
#[command(
    name = "cougr",
    version,
    about = "Tooling for building on-chain games with cougr-core",
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new Cougr game contract crate.
    New {
        /// Name of the project. Becomes the crate name and the directory name.
        name: String,

        /// Starting point for the generated project.
        #[arg(short, long, value_enum, default_value_t = Template::Starter)]
        template: Template,

        /// Directory to create the project in. Defaults to the current directory.
        #[arg(long, value_name = "DIR")]
        path: Option<std::path::PathBuf>,
    },

    /// Add a cougr-core capability to an existing project.
    ///
    /// This command is a stub — implementation lands in a follow-up issue.
    Add,

    /// Run repository hygiene checks on examples/.
    ///
    /// Auto-detects whether run from repo root (checks all examples)
    /// or an individual example directory (checks one example).
    ///
    /// With --verified, runs the full canonical-quality checklist from
    /// EXAMPLE_STANDARD.md and produces pass/fail data suitable for the
    /// "Cougr Verified" badge in the showcase.
    Check {
        /// Explicit path to the repository root.
        #[arg(short, long)]
        path: Option<String>,

        /// Check a single example by name (e.g. "snake"). Requires --path or repo-root cwd.
        #[arg(short, long)]
        example: Option<String>,

        /// Run the full canonical-quality checklist for the "Cougr Verified" badge.
        #[arg(long)]
        verified: bool,

        /// Output results as JSON (for machine consumption by the showcase generator).
        #[arg(long)]
        json: bool,

        /// Also run heavy build-validation checks (cargo test, stellar contract build).
        #[arg(long)]
        full: bool,

        /// Only check the 10 canonical examples (per EXAMPLE_STANDARD.md §7).
        #[arg(long)]
        canonical_only: bool,

        /// Write verified badge results to a JSON file (for showcase/gallery consumption).
        #[arg(short = 'o', long)]
        output: Option<String>,
    },

    /// Diagnose the local environment for Cougr development.
    ///
    /// This command is a stub — implementation lands in a follow-up issue.
    Doctor,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::New {
            name,
            template,
            path,
        } => {
            let result: Result<()> =
                commands::new::run(&name, template, path.as_deref()).map_err(anyhow::Error::from);
            handle_result(result)
        }

        Command::Add => commands::add::run(),

        Command::Check {
            path,
            example,
            verified,
            json,
            full,
            canonical_only,
            output,
        } => {
            let result: Result<()> = (|| {
                let cwd = std::env::current_dir()?;
                let ctx = context::resolve(&cwd, path.as_deref(), example.as_deref())?;

                if verified {
                    verify::run(
                        &ctx,
                        json || output.is_some(),
                        full,
                        canonical_only,
                        output.as_deref(),
                    )?;
                } else {
                    check::run(&ctx)?;
                }
                Ok(())
            })();
            handle_result(result)
        }

        Command::Doctor => commands::doctor::run(),
    }
}

fn handle_result(result: Result<()>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");

            // Show hint for known error types
            if let Some(cli_err) = err.downcast_ref::<CliError>() {
                if let Some(hint) = cli_err.hint() {
                    eprintln!("  help: {hint}");
                }
            }

            ExitCode::FAILURE
        }
    }
}
