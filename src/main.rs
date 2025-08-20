// mod cli;
mod core;
mod command;
mod compile;

use command::{create_table, create_table_v2, view, insert};

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
        path: Vec<Utf8PathBuf>,
        /// Use v1 format parser
        #[arg(short = '1', long = "v1")]
        v1: bool,
        /// Output in JSON format
        #[arg(short, long)]
        json: bool,
        /// Merge locals and value stack into a single value_stack (for protobuf only)
        #[arg(long)]
        merged_stack: bool,
    },
    /// Insert a NOP instruction at a specific offset within a specific function
    Insert {
        /// Path to input WASM file
        #[arg(short, long)]
        input: Utf8PathBuf,
        /// Path to output WASM file
        #[arg(short, long)]
        output: Utf8PathBuf,
        /// Function index (0-based)
        function_index: u32,
        /// Offset within the function where to insert NOP
        offset: u32,
    }
}

#[derive(Debug, Parser)]
struct CreateArgs {
    /// Path to input file
    path: Utf8PathBuf,

    /// Use v2 format
    #[arg(long)]
    v2: bool,

    /// Set offset before execution
    #[arg(long)]
    before_execution: bool,
}


fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match cli.subcommand {
        SubCommands::Create(args) => {
            let path = args.path;
            if args.v2 {
                let result = create_table_v2::create_table_v2(path, args.before_execution);
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
        SubCommands::View { path, v1, json, merged_stack } => {
            let result = if path.len() == 1 {
                let single_path = path[0].clone();
                if v1 {
                    view::view_v1_format(single_path, json)
                } else {
                    view::view_protobuf(single_path, merged_stack)
                }
            } else {
                if v1 {
                    view::view_v1_format_multiple(path, json)
                } else {
                    log::error!("Protobuf viewing does not support multiple files");
                    Err(anyhow::anyhow!("Protobuf viewing does not support multiple files"))
                }
            };

            match result {
                Ok(_) => log::info!("Successfully displayed file(s)"),
                Err(err) => log::error!("Failed to view file(s): {}", err)
            }
        },
        SubCommands::Insert { input, output, function_index, offset } => {
            let result = insert::insert_nop(input, output, function_index, offset);
            match result {
                Ok(_) => log::info!("Successfully inserted NOP instruction"),
                Err(err) => log::error!("Failed to insert NOP instruction: {}", err)
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
