use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

use crate::util::die;

pub fn read_data(infile: Option<PathBuf>) -> String {
    match infile {
        Some(path) => read_data_file(&path),
        None => read_data_stdin(),
    }
}

fn read_data_stdin() -> String {
    let mut data = String::new();
    if let Some(err) = io::stdin().read_to_string(&mut data).err() {
        die(format!("failed to read data from stdin: {}", err))
    } else {
        data
    }
}

fn read_data_file(path: &PathBuf) -> String {
    let mut file = File::open(path).unwrap_or_else(|err| {
        die(format!(
            "failed to open infile: {}: {}",
            path.display(),
            err
        ))
    });
    let mut data = String::new();
    match file.read_to_string(&mut data) {
        Ok(_) => data,
        Err(err) => die(format!(
            "failed to read infile: {}: {}",
            path.display(),
            err
        )),
    }
}
