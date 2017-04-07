use hyper;
use std::error;
use std::fmt;
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    description: &'static str,
}

#[derive(Debug)]
pub enum ErrorKind {
    Network(hyper::Error),
    Video,
}

impl Error {
    pub fn new(kind: ErrorKind, description: &'static str) -> Error {
        Error { kind, description }
    }

    pub fn video(description: &'static str) -> Error {
        Error::new(ErrorKind::Video, description)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        self.description
    }

    fn cause(&self) -> Option<&error::Error> {
        match self.kind {
            ErrorKind::Network(ref error) => Some(error),
            _ => None,
        }
    }
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Error {
        Error {
            kind: ErrorKind::Network(error),
            description: "network error",
        }
    }    
}
