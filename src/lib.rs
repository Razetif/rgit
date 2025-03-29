use crate::commands::Commands;
use clap::Parser;

mod commands;
mod error;
mod index;
mod object;

const GIT_DIR: &str = ".rgit";
const INDEX_FILE: &str = "index";
const OBJECTS_DIR: &str = "objects";
const OBJECTS_INFO_DIR: &str = "info";
const OBJECTS_PACK_DIR: &str = "pack";
const REFS_DIR: &str = "refs";
const REFS_HEADS_DIR: &str = "heads";
const REFS_TAGS_DIR: &str = "refs";

const SUBDIR_LEN: usize = 2;

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
