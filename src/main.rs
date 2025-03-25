use clap::Parser;
use rgit::Cli;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    Cli::parse().command.run()
}
