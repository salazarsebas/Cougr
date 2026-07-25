use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "cougr",
    about = "Cougr CLI — scaffold, validate, and inspect Cougr on-chain game projects",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scaffold a new Soroban contract crate with cougr-core wired up
    New {
        /// Name of the project
        name: String,
    },
    /// Pull a specific component/system pair into an existing project
    Add {
        /// Piece to add (e.g., session-auth, hidden-hand, standards/pausable)
        piece: String,
    },
    /// Run hygiene and standard-compliance checks on the current project
    Check,
    /// Verify the local toolchain and give actionable fix instructions
    Doctor,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::New { name } => {
            eprintln!("error: 'cougr new' is not yet implemented (scaffolding {name})");
            std::process::exit(1);
        }
        Commands::Add { piece } => {
            eprintln!("error: 'cougr add' is not yet implemented (adding {piece})");
            std::process::exit(1);
        }
        Commands::Check => {
            eprintln!("error: 'cougr check' is not yet implemented");
            std::process::exit(1);
        }
        Commands::Doctor => {
            eprintln!("error: 'cougr doctor' is not yet implemented");
            std::process::exit(1);
        }
    }
}
