use std::fmt::{self, Display, Formatter};
use std::io;


/// An enum for the different kinds of errors that can happen in this tool
pub enum Error {
    /// An error that Hexmake generates from its own code
    Hexmake(String),

    /// An IO error
    Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::Hexmake(error) => write!(f, "{error}"),
            Error::Io(error) => write!(f, "{error}"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}

impl From<String> for Error {
    fn from(error: String) -> Error {
        Error::Hexmake(error)
    }
}
