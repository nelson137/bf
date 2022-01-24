use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use anyhow::{Context, Result};

use crate::{err_file_open, err_file_read};

pub fn read_data(infile: Option<PathBuf>) -> Result<String> {
    match infile {
        Some(path) => read_data_file(&path),
        None => read_data_stdin(),
    }
}

fn read_data_stdin() -> Result<String> {
    let mut data = String::new();
    match io::stdin().read_to_string(&mut data) {
        Ok(_) => Ok(data),
        Err(e) => {
            Err(e).with_context(|| err_file_read!(PathBuf::from("STDIN")))
        }
    }
}

fn read_data_file(path: &PathBuf) -> Result<String> {
    let mut file = File::open(path).with_context(|| err_file_open!(path))?;
    let mut data = String::new();
    match file.read_to_string(&mut data) {
        Ok(_) => Ok(data),
        Err(e) => Err(e).with_context(|| err_file_read!(path)),
    }
}
