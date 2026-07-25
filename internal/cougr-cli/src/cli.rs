//! Clap-derived CLI definition and dispatch glue.
//!
//! Kept in a single module so test code can drive [`Cli::try_parse_from`]
//! against synthetic argv without having to re-derive the parser elsewhere.
//! Each dispatch helper returns a [`crate::CliError`] so the top-level
//! [`crate::run`] can convert them to exit codes uniformly.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::doctor::{self, DoctorConfig};
use crate::new;

/// Top-level `cougr <subcommand>` parser.
#[derive(Debug, Parser)]
#[command(
    name = "cougr",
    about = "Cougr CLI - toolchain diagnostics and project scaffolding",
    long_about = None,
    bin_name = "cougr",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// All subcommands supported in this version of the CLI.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run the local toolchain diagnostics and exit non-zero on any failure.
    Doctor(DoctorArgs),

    /// Scaffold a new Cougr project and run `doctor` non-fatally as a
    /// pre-flight check.
    #[command(visible_alias = "init")]
    New(NewArgs),
}

/// Arguments for `cougr doctor`.
#[derive(Debug, Parser)]
pub struct DoctorArgs {
    /// Path to a Cargo.toml whose `rust-version` should be used as the minimum
    /// required Rust version. If omitted, the doctor walks up to find a
    /// `Cargo.toml` from the current directory.
    #[arg(long, value_name = "PATH")]
    pub rust_manifest: Option<PathBuf>,

    /// Override the Stellar CLI minimum version. The default is the constant
    /// baked into this build of the CLI (`DOCTOR_DEFAULT_STELLAR_MIN_VERSION`).
    #[arg(long, value_name = "VERSION")]
    pub stellar_min: Option<String>,
}

/// Arguments for `cougr new <name>`.
#[derive(Debug, Parser)]
pub struct NewArgs {
    /// Project name. Must be a valid Rust crate identifier (lowercase,
    /// alphanumeric, optional hyphens and underscores). The directory
    /// `./<name>/` will be created relative to the current working directory.
    pub name: String,

    /// Skip the `cougr doctor` pre-flight check. Use only when you have
    /// already verified the toolchain manually and do not want warnings
    /// printed to stderr.
    #[arg(long)]
    pub no_doctor: bool,

    /// Do not generate any files; print what would be created and exit.
    #[arg(long)]
    pub dry_run: bool,
}

pub fn dispatch_doctor(args: DoctorArgs) -> Result<(), crate::CliError> {
    let config = DoctorConfig {
        rust_manifest: args.rust_manifest,
        stellar_min_version: args.stellar_min.unwrap_or_else(|| {
            doctor::DOCTOR_DEFAULT_STELLAR_MIN_VERSION.to_string()
        }),
    };
    let report = doctor::run(&config, &doctor::runner::SystemRunner);
    doctor::print_report(&report, false);
    if report.all_passed() {
        Ok(())
    } else {
        Err(crate::CliError::DoctorFailed)
    }
}

pub fn dispatch_new(args: NewArgs) -> Result<(), crate::CliError> {
    // Pre-flight doctor unless suppressed. The doctor must NOT abort the
    // scaffold: per issue #247 the pre-flight is "non-fatal, as a warning".
    // We only short-circuit if the scaffold itself reports a hard failure.
    let doctor_warnings = if !args.no_doctor {
        let config = DoctorConfig::default();
        let report = doctor::run(&config, &doctor::runner::SystemRunner);
        if !report.all_passed() {
            eprintln!(
                "warning: `cougr doctor` reported {}/{} checks failing; \
                 the new project may not build until they are addressed.",
                report.failed_count(),
                report.total_count(),
            );
            doctor::print_report(&report, true);
            true
        } else {
            false
        }
    } else {
        false
    };

    let result = new::scaffold(&args.name, args.dry_run);
    match (result, doctor_warnings) {
        (Err(err), _) => Err(crate::CliError::Scaffold(err.to_string())),
        (Ok(new::ScaffoldOutcome::Created(path)), true) => {
            println!(
                "created project at {} (cougr doctor printed warnings; see above)",
                path.display()
            );
            Err(crate::CliError::DoctorWarningsPrinted(0))
        }
        (Ok(new::ScaffoldOutcome::Created(path)), false) => {
            println!("created project at {}", path.display());
            Ok(())
        }
        (Ok(new::ScaffoldOutcome::DryRun(plan)), _) => {
            println!("dry run: would create the following files:");
            for line in plan.lines() {
                println!("  {line}");
            }
            Ok(())
        }
    }
}
