mod file;
mod reporter;
mod run_compile;
mod run_lsp;

use clap::{Args, Parser, Subcommand};

/// A rich serialization protocol for dependently typed data.
#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start language server
    Lsp,
    /// Compile dbuf files to languages
    Compile(CompileParams),
}

#[derive(Args, Debug)]
struct CompileParams {
    /// Input file
    #[arg(short, long)]
    file: String,

    /// Output path
    #[arg(short, long, default_value = ".")]
    path: String,

    /// Output languages
    #[arg(short, long, num_args=1..)]
    output: Vec<String>,
}

pub fn parse_cli(cli: Cli) {
    match cli.command {
        Commands::Lsp => run_lsp::run(),
        Commands::Compile(compile_params) => run_compile::run(&compile_params),
    }
}
