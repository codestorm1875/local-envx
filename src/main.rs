mod audit;
mod cli;
mod commands;
mod crypto;
mod error;
mod store;
mod tui;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    commands::run(cli.command)
}
