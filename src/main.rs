// mod cli;
mod core;
mod command;
mod compile;

use command::{create_table, create_table_v2};

use env_logger;
// use log::{debug, error, log_enabled, info, Level};
use clap::{Parser, Subcommand};
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
    Create(CreateArgs),
    Display {
        path: Utf8PathBuf,
    }
}

#[derive(Debug, Parser)]
struct CreateArgs {
    /// Path to input file
    path: Utf8PathBuf,

    /// Use v2 format
    #[arg(long)]
    v2: bool,
}


fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match cli.subcommand {
        SubCommands::Create(args) => {
            let path = args.path;
            if args.v2 {
                let result = create_table_v2::create_table_v2(path);
                match result {
                    Ok(_) => log::info!("Success to create the type stack tables"),
                    Err(err) => log::error!("Failed to create the type stack table, {}", err)
                }
            } else {
                let result = create_table::create_table(path);
                match result {
                    Ok(_) => log::info!("Success to create the type stack tables"),
                    Err(err) => log::error!("Failed to create the type stack table, {}", err)
                }
            }
        },
        SubCommands::Display { .. } => {
            todo!();
        }
        // SubCommands::Display { path } => {
            // let result = display::main(path);
            // match result {
            //     Ok(_) => log::info!("Success to display"),
            //     Err(err) => log::error!("Failed to display"),
            // }
        // },
    }
}
