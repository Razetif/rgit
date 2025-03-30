use std::fmt::Display;
use std::fmt::{self, Formatter};

use anyhow::Result;

use crate::error::MalformedError;
use crate::utils::CHECKSUM_LEN;

pub enum Type {
    Blob,
    Tree,
    Commit,
}

impl Type {
    pub fn build(s: impl AsRef<str>) -> Result<Self> {
        match s.as_ref() {
            "blob" => Ok(Self::Blob),
            "tree" => Ok(Self::Tree),
            "commit" => Ok(Self::Commit),
            _ => Err(MalformedError.into()),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Type::Blob => write!(f, "blob"),
            Type::Tree => write!(f, "tree"),
            Type::Commit => write!(f, "commit"),
        }
    }
}

pub type ObjectId = [u8; CHECKSUM_LEN];
