//! cougr — the Cougr CLI.
//!
//! # Commands
//!
//! * `cougr add --list`   — list all available pieces.
//! * `cougr add <piece>`  — copy a piece into the current project's `src/`.

mod add;
mod catalog;

use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

/// The Cougr CLI — scaffold projects and add pieces from the canonical example catalog.
#[derive(Parser, Debug)]
#[command(
    name    = "cougr",
    version = env!("CARGO_PKG_VERSION"),
    about   = "The Cougr CLI for Soroban game development",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add a reusable piece to the current project (shadcn-style: owned source).
    ///
    /// Run `cougr add --list` to see all available pieces.
    Add {
        /// Name of the piece to add (e.g. "session-auth", "hidden-hand",
        /// "standards/pausable").  Omit to use --list.
        piece: Option<String>,

        /// List all available pieces with descriptions and maturity tiers.
        #[arg(long, short = 'l')]
        list: bool,

        /// Project root directory (defaults to the current working directory).
        #[arg(long, short = 'p', default_value = ".")]
        project: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Add {
            piece,
            list,
            project,
        } => {
            if list {
                add::run_list()
            } else if let Some(name) = piece {
                let root = if project == PathBuf::from(".") {
                    std::env::current_dir()
                        .expect("cannot determine current directory")
                } else {
                    project
                };
                add::run_add(&name, &root)
            } else {
                // Neither --list nor a piece name was given — print help.
                eprintln!(
                    "error: specify a piece name or use --list.\n\n\
                     Examples:\n\
                     \tcougr add --list\n\
                     \tcougr add session-auth\n\
                     \tcougr add hidden-hand\n\
                     \tcougr add standards/pausable"
                );
                process::exit(1);
            }
        }
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
