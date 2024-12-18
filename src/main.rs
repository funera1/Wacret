// mod cli;
mod core;
mod create_table;

use env_logger;
// use log::{debug, error, log_enabled, info, Level};
use clap::{Parser, Subcommand, Args};
use camino::Utf8PathBuf;

#[derive(Debug, Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    arg_required_else_help = true,
)]
struct Cli {
    #[clap(subcommand)]
    subcommand: SubCommands,
}

#[derive(Debug, Subcommand)]
enum SubCommands {
    /// Create type stack tables for checkpointing a wasm app.
    Create {
        path: Utf8PathBuf,
    },
}

#[derive(Debug, Args)]
struct CreateArgs {
    path: Utf8PathBuf,
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match cli.subcommand {
        SubCommands::Create{ path } => {
            let result = create_table::create_table(path);
            match result {
                Ok(_) => log::info!("Success to create the type stack tables"),
                Err(err) => log::error!("Fialed to create the type stack table, {}", err)
            }
        },
    }
}
