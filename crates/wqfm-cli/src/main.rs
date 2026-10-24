//! `wqfm-rs`: weighted quartet amalgamation from the command line.

mod args;
mod commands;

use clap::Parser;

fn main() {
    let cli = args::Cli::parse();
    if let Err(e) = commands::run(cli.command) {
        eprintln!("wqfm-rs: {e}");
        std::process::exit(1);
    }
}
