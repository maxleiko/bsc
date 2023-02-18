use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Bs(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(err) => err.fmt(f),
            Error::Bs(err) => err.fmt(f),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::Bs(value.to_string())
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Bs(value.to_string())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Bs(value)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(value: serde_yaml::Error) -> Self {
        Self::Bs(value.to_string())
    }
}
