use std::fmt;
use std::error;
use std::result;
use std::io;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Other(String)
}

/*impl Error {
    fn other(message: &str) -> Self {
        Error::Other(String::from(message))
    }
}*/

impl error::Error for Error {
    fn description<'a>(&'a self) -> &'a str {
        /*match self {
            &Error::IO(err) => err.description(),
            &Error::Other(string) => &string
        }*/
        "Import/Export error"
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IO(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

