use std::{error::Error, fmt, io, iter, path::PathBuf};

use sha1::{Digest, Sha1};

#[cfg(windows)]
pub const EOL: &str = "\r\n";
#[cfg(not(windows))]
pub const EOL: &str = "\n";

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
    fn from(err: &'static str) -> Self { Self::Simple(err.to_string()) }
}
impl From<String> for BfError {
    fn from(err: String) -> Self { Self::Simple(err) }
}

impl From<io::Error> for BfError {
    fn from(err: io::Error) -> Self { Self::IO(err) }
}

impl From<crossterm::ErrorKind> for BfError {
    fn from(err: crossterm::ErrorKind) -> Self { Self::Crossterm(err) }
}

impl fmt::Display for BfError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use BfError::*;
        match self {
            Simple(msg) => write!(fmt, "{}", msg),
            IO(err) => write!(fmt, "{}", io::Error::from(err.kind())),
            Print(err) => write!(
                fmt,
                "failed to print: {}",
                io::Error::from(err.kind())
            ),
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
        crate::util::BfError::$type($($args)*)
    };
}

pub trait StrExt {
    fn repeated(&self, n: usize) -> String;
}

impl StrExt for str {
    fn repeated(&self, n: usize) -> String {
        iter::repeat(self).take(n).collect::<String>()
    }
}

pub fn get_width(width: Option<usize>) -> i32 {
    (match width {
        Some(w) => w,
        None => match term_size::dimensions() {
            Some((w, _h)) if w > 5 => w,
            _ => 65, // Wide enough for 16 cells
        },
    }) as i32
}

pub fn is_valid_infile(value: String) -> Result<(), String> {
    if value == "-" {
        return Ok(());
    }

    let path = PathBuf::from(&value);
    if path.exists() {
        if path.is_dir() {
            Err(format!("file is a directory: {}", value))
        } else {
            Ok(())
        }
    } else {
        Err(format!("no such file exists: {}", value))
    }
}

pub fn is_valid_width(value: String) -> Result<(), String> {
    match value.parse::<i64>() {
        Ok(n) => {
            if n < 5 {
                Err("value must be an integer > 5".to_string())
            } else {
                Ok(())
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

pub type Sha1Digest = [u8; 20];

pub fn sha1_digest<D: AsRef<[u8]>>(data: D) -> Sha1Digest {
    Sha1::new().chain(data).finalize().into()
}
