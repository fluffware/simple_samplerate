use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    NoError,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error>
    {
	use Error::*;
	match self {
	    NoError => write!(f, "No error")
	}
    }
}
