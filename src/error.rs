use std::error;
use std::fmt;
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    description: &'static str,
    cause: Option<Box<error::Error>>,
}

#[derive(Debug)]
pub enum ErrorKind {
    Network,
    Video,
}

impl Error {
    pub fn new(kind: ErrorKind, description: &'static str) -> Error {
        Error { kind, description, cause: None }
    }

    pub fn network<T: error::Error + 'static>(description: &'static str, cause: Option<T>) -> Error {
        Error {
            kind: ErrorKind::Network,
            description,
            cause: cause.map(|e| Box::new(e) as Box<error::Error>),
        }
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
        match self.cause {
            Some(ref e) => Some(e.as_ref()),
            None => None,
        }
    }
}
