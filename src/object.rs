use std::fmt::{self, Formatter};
use std::{error::Error, fmt::Display};

pub enum Type {
    Blob,
    Tree,
    Commit,
}

impl Type {
    pub fn build(s: impl AsRef<str>) -> Result<Self, Box<dyn Error>> {
        match s.as_ref() {
            "blob" => Ok(Self::Blob),
            "tree" => Ok(Self::Tree),
            "commit" => Ok(Self::Commit),
            _ => Err("Invalid object type".into()),
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
