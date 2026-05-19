mod cli;
mod file;
mod file_content;
mod reporter;
mod run_compile;
mod run_format;
mod run_lsp;

pub use cli::Cli;

pub fn run_cli(cli: cli::Cli) -> ! {
    match cli.command {
        cli::Commands::Lsp => run_lsp::run(),
        cli::Commands::Compile(compile_params) => run_compile::run(&compile_params),
        cli::Commands::Format(format_params) => run_format::run(&format_params),
    }
}
