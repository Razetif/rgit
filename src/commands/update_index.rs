use crate::utils::{self, INDEX_FILE};
use crate::{
    error::MalformedError,
    index::{Entry, Index},
};
use anyhow::Result;
use clap::Args;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, Write},
    os::unix::fs::MetadataExt,
    path::PathBuf,
};

#[derive(Args, Debug)]
pub struct UpdateIndexArgs {
    #[arg(long = "add")]
    add: bool,

    #[arg(long = "remove")]
    remove: bool,

    #[arg(long = "verbose")]
    verbose: bool,

    files: Vec<PathBuf>,
}

pub fn run(args: &UpdateIndexArgs) -> Result<()> {
    let index_file_path = utils::resolve_path(&[INDEX_FILE])?;
    let mut index_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&index_file_path)?;
    let mut index = if fs::metadata(&index_file_path)?.size() == 0 {
        Index::empty()
    } else {
        let mut buf = Vec::new();
        index_file.read_to_end(&mut buf)?;
        Index::parse(buf)?
    };

    let entries: Vec<_> = args
        .files
        .iter()
        .map(|filename| {
            let mut file = File::open(filename)?;
            let entry = Entry::from(filename.to_str().ok_or(MalformedError)?, &mut file);
            entry
        })
        .collect::<Result<_>>()?;
    for entry in entries {
        if args.remove {
            if index.entries.contains(&entry) {
                index.entries.retain(|e| *e != entry);
                if args.verbose {
                    println!("remove '{}'", entry.filename);
                }
            }
        }

        if args.add {
            if !index.entries.contains(&entry) {
                let filename = entry.filename.clone();
                index.entries.push(entry);
                if args.verbose {
                    println!("add '{}'", filename);
                }
            }
        }
    }

    let content = index.serialize()?;
    index_file.rewind()?;
    index_file.write_all(&content)?;
    index_file.set_len(content.len() as u64)?;

    Ok(())
}
