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
