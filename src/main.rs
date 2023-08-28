use clap::Parser;
use crate::args::Cli;

mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    Ok(())
}
