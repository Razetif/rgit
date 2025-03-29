use anyhow::Result;
use cat_file::CatFileArgs;
use clap::Subcommand;
use hash_object::HashObjectArgs;
use init::InitArgs;
use ls_files::LsFilesArgs;
use update_index::UpdateIndexArgs;

mod cat_file;
mod hash_object;
mod init;
mod ls_files;
mod update_index;

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init(InitArgs),
    HashObject(HashObjectArgs),
    CatFile(CatFileArgs),
    UpdateIndex(UpdateIndexArgs),
    LsFiles(LsFilesArgs),
}

impl Commands {
    pub fn run(&self) -> Result<()> {
        match self {
            Self::Init(args) => init::run(args),
            Self::HashObject(args) => hash_object::run(&args),
            Self::CatFile(args) => cat_file::run(&args),
            Self::UpdateIndex(args) => update_index::run(&args),
            Self::LsFiles(args) => ls_files::run(&args),
        }
    }
}
