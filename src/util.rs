use std::{
    error::Error,
    fmt,
    iter,
    path::PathBuf,
    process::exit,
};

use sha1::{Digest, Sha1};

#[cfg(windows)]
pub const EOL: &str = "\r\n";
#[cfg(not(windows))]
pub const EOL: &str = "\n";

#[derive(Debug)]
pub struct BfError {
    msg: String,
}

impl BfError {
    pub fn from<S: AsRef<str>>(s: S) -> Self {
        Self {
            msg: String::from(s.as_ref()),
        }
    }
}

impl fmt::Display for BfError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n", self.msg)
    }
}

impl Error for BfError {
}

#[macro_export]
macro_rules! bf_err {
    ($msg:tt) => {
        crate::util::BfError::from($msg).into()
    };
    ($($args:tt)*) => {
        crate::util::BfError::from(format!($($args)*)).into()
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

pub fn die(msg: String) -> ! {
    eprintln!("bf: error: {}", msg);
    exit(1);
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
