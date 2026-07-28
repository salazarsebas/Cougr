//! Cougr CLI — development tooling for the Cougr ECS framework.
//!
//! ## Subcommands
//!
//! - `cougr check` — Run repository hygiene checks against examples/
//! - `cougr check --verified` — Run the full canonical-quality checklist
//!   (EXAMPLE_STANDARD.md) for the "Cougr Verified" badge.
//!
//! `cougr check` detects its context automatically:
//! - Run from the repo root (contains `examples/` and `Cargo.toml`): checks **all** examples.
//! - Run from `examples/<name>/` (contains `Cargo.toml` with a parent `examples/`):
//!   checks that **single example**.
//!
//! Pass `--path <PATH>` to override auto-detection and specify the repo root explicitly.

mod check;
mod context;
mod verify;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Cougr development CLI.
#[derive(Parser)]
#[command(name = "cougr", about = "Cougr development tooling", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
        ///
        /// When provided, all checks run as if invoked from the repo root.
        #[arg(short, long)]
        path: Option<String>,

        /// Check a single example by name (e.g. "snake"). Requires --path or repo-root cwd.
        #[arg(short, long)]
        example: Option<String>,

        /// Run the full canonical-quality checklist for the "Cougr Verified" badge.
        ///
        /// Evaluates every criterion in EXAMPLE_STANDARD.md: dependencies,
        /// module structure, README completeness, test coverage, classification,
        /// and Cargo.lock hygiene. Produces structured pass/fail data.
        #[arg(long)]
        verified: bool,

        /// Output results as JSON (for machine consumption by the showcase generator).
        #[arg(long)]
        json: bool,

        /// Also run heavy build-validation checks (cargo test, stellar contract build).
        /// These are skipped by default because they are slow.
        #[arg(long)]
        full: bool,

        /// Only check the 10 canonical examples (per EXAMPLE_STANDARD.md §7).
        /// When --verified is active, this filters to the canonical list.
        #[arg(long)]
        canonical_only: bool,

        /// Write verified badge results to a JSON file (for showcase/gallery consumption).
        ///
        /// When provided with --verified, the structured pass/fail data is written
        /// to this path in addition to stdout. Implies --json.
        #[arg(short = 'o', long)]
        output: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check {
            path,
            example,
            verified,
            json,
            full,
            canonical_only,
            output,
        } => {
            let cwd = std::env::current_dir()?;
            let ctx = context::resolve(&cwd, path.as_deref(), example.as_deref())?;

            if verified {
                verify::run(
                    &ctx,
                    json || output.is_some(),
                    full,
                    canonical_only,
                    output.as_deref(),
                )
            } else {
                check::run(&ctx)
            }
        }
    }
}
//! `cougr` — command-line tooling for the Cougr ECS framework.
//!
//! Currently exposes a single command, [`cougr new`](commands::new), which
//! scaffolds a Soroban game contract wired to `cougr-core` from one of four
//! embedded templates.

mod commands;
mod error;
mod name;
mod template;

use std::process::ExitCode;

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
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::New {
            name,
            template,
            path,
        } => commands::new::run(&name, template, path.as_deref()),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            report(&err);
            ExitCode::FAILURE
        }
    }
}

fn report(err: &CliError) {
    eprintln!("error: {err}");
    if let Some(hint) = err.hint() {
        eprintln!("  help: {hint}");
    }
}
