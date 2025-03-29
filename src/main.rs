use anyhow::Result;
use clap::Parser;
use rgit::Cli;

fn main() -> Result<()> {
    Cli::parse().command.run()
}
