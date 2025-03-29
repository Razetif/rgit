use crate::{
    GIT_DIR, OBJECTS_DIR, OBJECTS_INFO_DIR, OBJECTS_PACK_DIR, REFS_DIR, REFS_HEADS_DIR,
    REFS_TAGS_DIR,
};
use anyhow::Result;
use clap::Args;
use std::{fs, path};

#[derive(Args, Debug)]
pub struct InitArgs {}

pub fn run(_args: &InitArgs) -> Result<()> {
    let git_dir_path = path::absolute(GIT_DIR)?;
    let message = if git_dir_path.try_exists()? {
        "Reinitialized existing Git repository in"
    } else {
        "Initialized empty Git repository in"
    };
    println!("{} {}", message, git_dir_path.display());

    fs::create_dir_all(git_dir_path.join(OBJECTS_DIR).join(OBJECTS_INFO_DIR))?;
    fs::create_dir_all(git_dir_path.join(OBJECTS_DIR).join(OBJECTS_PACK_DIR))?;
    fs::create_dir_all(git_dir_path.join(REFS_DIR).join(REFS_HEADS_DIR))?;
    fs::create_dir_all(git_dir_path.join(REFS_DIR).join(REFS_TAGS_DIR))?;

    Ok(())
}
