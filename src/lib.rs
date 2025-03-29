use crate::commands::Commands;
use clap::Parser;

mod commands;
mod error;
mod index;
mod object;
mod utils;

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
