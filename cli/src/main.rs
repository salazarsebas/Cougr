//! Cougr CLI — development tooling for the Cougr ECS framework.
//!
//! ## Subcommands
//!
//! - `cougr check` — Run repository hygiene checks against examples/
//!
//! `cougr check` detects its context automatically:
//! - Run from the repo root (contains `examples/` and `Cargo.toml`): checks **all** examples.
//! - Run from `examples/<name>/` (contains `Cargo.toml` with a parent `examples/`):
//!   checks that **single example**.
//!
//! Pass `--path <PATH>` to override auto-detection and specify the repo root explicitly.

mod check;

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
    Check {
        /// Explicit path to the repository root.
        ///
        /// When provided, all checks run as if invoked from the repo root.
        #[arg(short, long)]
        path: Option<String>,

        /// Check a single example by name (e.g. "snake"). Requires --path or repo-root cwd.
        #[arg(short, long)]
        example: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { path, example } => check::run(path.as_deref(), example.as_deref()),
    }
}
