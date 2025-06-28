// mod cli;
mod core;
mod command;
mod compile;

use command::{create_table, create_table_v2, view};

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
    /// Display stack tables in human-readable format
    Display {
        path: Utf8PathBuf,
    },
    /// View protobuf files in JSON format
    View {
        path: Utf8PathBuf,
        /// Use v1 format parser
        #[arg(short = '1', long = "v1")]
        v1: bool,
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
        },
        SubCommands::View { path, v1 } => {
            let result = if v1 {
                view::view_v1_format(path)
            } else {
                view::view_protobuf(path)
            };
            match result {
                Ok(_) => log::info!("Successfully displayed file"),
                Err(err) => log::error!("Failed to view file: {}", err)
            }
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
