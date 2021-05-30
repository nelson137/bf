use std::{error::Error, fmt, io, path::PathBuf};

pub type BfResult<T> = Result<T, BfError>;

#[derive(Debug)]
pub enum BfError {
    Simple(String),
    IO(io::Error),
    Print(io::Error),
    Crossterm(crossterm::ErrorKind),
    FileOpen(io::Error, PathBuf),
    FileRead(io::Error, PathBuf),
    FileWrite(io::Error, PathBuf),
}

impl Error for BfError {}

impl From<&'static str> for BfError {
    fn from(err: &'static str) -> Self {
        Self::Simple(err.to_string())
    }
}

impl From<String> for BfError {
    fn from(err: String) -> Self {
        Self::Simple(err)
    }
}

impl From<io::Error> for BfError {
    fn from(err: io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<crossterm::ErrorKind> for BfError {
    fn from(err: crossterm::ErrorKind) -> Self {
        Self::Crossterm(err)
    }
}

impl fmt::Display for BfError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use BfError::*;
        match self {
            Simple(msg) => write!(fmt, "{}", msg),
            IO(err) => write!(fmt, "{}", io::Error::from(err.kind())),
            Print(err) => {
                write!(fmt, "failed to print: {}", io::Error::from(err.kind()))
            }
            FileOpen(err, path) => write!(
                fmt,
                "failed to open file: {}: {}",
                io::Error::from(err.kind()),
                path.display(),
            ),
            FileRead(err, path) => write!(
                fmt,
                "failed to read file: {}: {}",
                io::Error::from(err.kind()),
                path.display(),
            ),
            FileWrite(err, path) => write!(
                fmt,
                "failed to write to file: {}: {}",
                io::Error::from(err.kind()),
                path.display(),
            ),
            Crossterm(err) => write!(fmt, "{}", err),
        }
    }
}

#[macro_export]
macro_rules! err {
    ($type:tt, $($args:tt)*) => {
        crate::util::err::BfError::$type($($args)*)
    };
}
