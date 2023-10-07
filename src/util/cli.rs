use anyhow::Result;
use std::{error::Error, path::PathBuf};

pub trait SubCmd {
    fn run(self) -> Result<()>;
}

pub type ClapError = Box<dyn Error + Send + Sync + 'static>;

pub fn parse_infile(value: &str) -> Result<PathBuf, ClapError> {
    let path = PathBuf::from(&value);

    if value == "-" {
        Ok(path)
    } else if path.exists() {
        if path.is_dir() {
            Err(format!("file is a directory: {}", value).into())
        } else {
            Ok(path)
        }
    } else {
        Err(format!("no such file exists: {}", value).into())
    }
}

pub fn parse_width(value: &str) -> Result<u64, ClapError> {
    match value.parse::<i64>() {
        Ok(n) => {
            if n < 5 {
                Err("value must be an integer > 5".into())
            } else {
                Ok(n as u64)
            }
        }
        Err(err) => Err(err.into()),
    }
}
