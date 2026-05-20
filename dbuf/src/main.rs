use clap::Parser;

use dbuf::Cli;
use dbuf::run_cli;

fn main() {
    let cli = Cli::parse();
    run_cli(cli);
}
