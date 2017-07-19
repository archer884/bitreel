use std::error;
use std::fmt;
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub(crate) struct Cause(Option<Box<error::Error>>);

impl Default for Cause {
    fn default() -> Self {
        Cause(None)
    }
}

impl<T: error::Error + 'static> From<T> for Cause {
    fn from(error: T) -> Self {
        Cause(Some(Box::new(error)))
    }
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    description: &'static str,
    cause: Cause,
}

#[derive(Debug)]
pub enum ErrorKind {
    Network,
    Query,
    Video,
}

impl Error {
    pub fn new(kind: ErrorKind, description: &'static str) -> Error {
        Error { kind, description, cause: Default::default() }
    }

    pub fn network(description: &'static str) -> Error {
        Error {
            kind: ErrorKind::Network,
            description,
            cause: Default::default(),
        }
    }

    pub fn video(description: &'static str) -> Error {
        Error::new(ErrorKind::Video, description)
    }

    pub fn unknown_query_type() -> Error {
        Error {
            kind: ErrorKind::Query,
            description: "Unknown query type",
            cause: Default::default(),
        }
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
        match self.cause.0.as_ref() {
            Some(e) => Some(e.as_ref()),
            None => None,
        }
    }
}
