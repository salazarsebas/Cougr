//! `cougr` — command-line tooling for the Cougr ECS framework.
//!
//! Commands:
//! - [`cougr new`](commands::new) — scaffold a new Soroban game contract from a template.
//! - [`cougr add`](commands::add) — copy a reusable piece into an existing project.

mod commands;
mod error;
mod name;
mod template;

use std::path::PathBuf;
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
        path: Option<PathBuf>,
    },

    /// Add a reusable piece to the current project (shadcn-style: owned source).
    ///
    /// Run `cougr add --list` to see all available pieces.
    Add {
        /// Name of the piece to add (e.g. "session-auth", "hidden-hand",
        /// "standards/pausable"). Omit to use --list.
        piece: Option<String>,

        /// List all available pieces with descriptions and maturity tiers.
        #[arg(long, short = 'l')]
        list: bool,

        /// Project root directory (defaults to the current working directory).
        #[arg(long, short = 'p', default_value = ".")]
        project: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::New {
            name,
            template,
            path,
        } => match commands::new::run(&name, template, path.as_deref()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                report_cli_error(&err);
                ExitCode::FAILURE
            }
        },

        Command::Add {
            piece,
            list,
            project,
        } => {
            let result = if list {
                commands::add::run_list()
            } else if let Some(name) = piece {
                let root = if project == PathBuf::from(".") {
                    std::env::current_dir().expect("cannot determine current directory")
                } else {
                    project
                };
                commands::add::run_add(&name, &root)
            } else {
                eprintln!(
                    "error: specify a piece name or use --list.\n\n\
                     Examples:\n\
                     \tcougr add --list\n\
                     \tcougr add session-auth\n\
                     \tcougr add hidden-hand\n\
                     \tcougr add standards/pausable"
                );
                return ExitCode::FAILURE;
            };

            match result {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("error: {err}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

fn report_cli_error(err: &CliError) {
    eprintln!("error: {err}");
    if let Some(hint) = err.hint() {
        eprintln!("  help: {hint}");
    }
}
