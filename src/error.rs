use std::error;
use std::fmt::{self, Display};

#[derive(Default, Debug)]
pub struct MalformedError;

impl Display for MalformedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Malformed data")
    }
}

impl error::Error for MalformedError {}
