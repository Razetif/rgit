use crate::utils::{
    self, OBJECTS_DIR, OBJECTS_INFO_DIR, OBJECTS_PACK_DIR, REFS_DIR, REFS_HEADS_DIR, REFS_TAGS_DIR,
    resolve_path,
};
use anyhow::Result;
use clap::Args;
use std::fs;

#[derive(Args, Debug)]
pub struct InitArgs {}

pub fn run(_args: &InitArgs) -> Result<()> {
    let git_dir_path = utils::resolve_path(&[] as &[&str])?;
    let message = if git_dir_path.try_exists()? {
        "Reinitialized existing Git repository in"
    } else {
        "Initialized empty Git repository in"
    };
    println!("{} {}", message, git_dir_path.display());

    fs::create_dir_all(resolve_path(&[OBJECTS_DIR, OBJECTS_INFO_DIR])?)?;
    fs::create_dir_all(resolve_path(&[OBJECTS_DIR, OBJECTS_PACK_DIR])?)?;
    fs::create_dir_all(resolve_path(&[REFS_DIR, REFS_HEADS_DIR])?)?;
    fs::create_dir_all(resolve_path(&[REFS_DIR, REFS_TAGS_DIR])?)?;

    Ok(())
}
