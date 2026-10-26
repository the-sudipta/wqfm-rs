//! `wqfm-rs`: weighted quartet amalgamation from the command line.

mod args;
mod commands;
mod ui;

use clap::Parser;

fn main() {
    let cli = args::Cli::parse();
    ui::set_quiet(cli.quiet);
    if let Err(e) = commands::run(cli.command) {
        ui::error(&e);
        std::process::exit(1);
    }
}
