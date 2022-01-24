use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use anyhow::{Context, Result};

use crate::{err_file_open, err_file_read};

pub fn read_script(infile: &Option<PathBuf>) -> Result<Vec<u8>> {
    match infile {
        Some(path) if *path != PathBuf::from("-") => read_script_file(path),
        _ => read_script_stdin(),
    }
}

fn read_script_stdin() -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    match io::stdin().read_to_end(&mut buf) {
        Ok(_) => Ok(buf),
        Err(e) => {
            Err(e).with_context(|| err_file_read!(PathBuf::from("STDIN")))
        }
    }
}

pub fn read_script_file(path: &PathBuf) -> Result<Vec<u8>> {
    let mut file = File::open(path).with_context(|| err_file_open!(path))?;
    let mut buf = Vec::new();
    match file.read_to_end(&mut buf) {
        Ok(_) => Ok(buf),
        Err(e) => Err(e).with_context(|| err_file_read!(path)),
    }
}
