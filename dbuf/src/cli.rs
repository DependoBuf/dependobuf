//! Module exports structs for Command Line Interface.
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// A rich serialization protocol for dependently typed data.
#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Compile dbuf files to languages.
    Compile(CompileParams),
    /// Format dbuf files inplace.
    Format(FormatParams),
    /// Start language server.
    Lsp,
}

#[derive(Args)]
pub struct CompileParams {
    /// Input file.
    #[arg(short, long)]
    pub file: PathBuf,

    /// Output path.
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,

    /// Output languages.
    #[arg(short, long, num_args=1..)]
    pub output: Vec<String>,
}

#[derive(Args, Debug)]
pub struct FormatParams {
    /// Files to format.
    pub files: Vec<PathBuf>,

    /// Sizes of tabs in spaces.
    #[arg(short, long, default_value = "4")]
    pub tab_size: usize,

    /// Check if files are formatted instead of formatting them.
    #[arg(long, default_value = "false")]
    pub check: bool,
}
