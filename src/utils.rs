use std::path::{Path, PathBuf};

use anyhow::Result;

pub const GIT_DIR: &str = ".rgit";
pub const INDEX_FILE: &str = "index";
pub const OBJECTS_DIR: &str = "objects";
pub const OBJECTS_INFO_DIR: &str = "info";
pub const OBJECTS_PACK_DIR: &str = "pack";
pub const REFS_DIR: &str = "refs";
pub const REFS_HEADS_DIR: &str = "heads";
pub const REFS_TAGS_DIR: &str = "tags";

pub const OBJECT_ID_SPLIT_MID: usize = 2;

pub const CHECKSUM_LEN: usize = 20;

pub fn resolve_path(parts: &[impl AsRef<Path>]) -> Result<PathBuf> {
    let base = PathBuf::from(GIT_DIR);
    let path = parts.iter().fold(base, |acc, part| acc.join(part));
    Ok(path)
}
