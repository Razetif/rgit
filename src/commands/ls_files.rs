use crate::index::Index;
use crate::utils::{self, INDEX_FILE};
use anyhow::Result;
use clap::Args;
use std::{fs, path::PathBuf};

#[derive(Args, Debug)]
pub struct LsFilesArgs {
    files: Vec<PathBuf>,
}

pub fn run(args: &LsFilesArgs) -> Result<()> {
    let index_file_path = utils::resolve_path(&[INDEX_FILE])?;
    let buf = fs::read(index_file_path)?;
    let index = Index::parse(buf)?;

    let entries = index
        .entries
        .iter()
        .filter(|entry| {
            args.files.is_empty()
                || args
                    .files
                    .iter()
                    .any(|file_path| file_path == &entry.file_path)
        })
        .collect::<Vec<_>>();
    for entry in entries {
        println!("{}", entry.file_path.display());
    }

    Ok(())
}
