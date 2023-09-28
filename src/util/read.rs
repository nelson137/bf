use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::PathBuf,
};

use anyhow::{Context, Result};

use crate::{err_file_open, err_file_read};

pub fn read_script(infile: Option<&PathBuf>) -> Result<Vec<String>> {
    match infile {
        Some(path) if *path != PathBuf::from("-") => read_script_file(path),
        _ => read_script_stdin(),
    }
}

fn read_script_stdin() -> Result<Vec<String>> {
    io::stdin()
        .lines()
        .collect::<Result<_, _>>()
        .with_context(|| err_file_read!(PathBuf::from("STDIN")))
}

pub fn read_script_file(path: &PathBuf) -> Result<Vec<String>> {
    let file = File::open(path).with_context(|| err_file_open!(path))?;
    BufReader::new(file)
        .lines()
        .collect::<Result<_, _>>()
        .with_context(|| err_file_read!(path))
}
