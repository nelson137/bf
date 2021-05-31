use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use crate::util::err::{err, BfResult};

pub fn read_data(infile: Option<PathBuf>) -> BfResult<String> {
    match infile {
        Some(path) => read_data_file(&path),
        None => read_data_stdin(),
    }
}

fn read_data_stdin() -> BfResult<String> {
    let mut data = String::new();
    match io::stdin().read_to_string(&mut data) {
        Ok(_) => Ok(data),
        Err(e) => Err(err!(FileRead, e, PathBuf::from("STDIN"))),
    }
}

fn read_data_file(path: &PathBuf) -> BfResult<String> {
    let mut file =
        File::open(path).map_err(|e| err!(FileOpen, e, path.clone()))?;
    let mut data = String::new();
    match file.read_to_string(&mut data) {
        Ok(_) => Ok(data),
        Err(e) => Err(err!(FileRead, e, path.clone())),
    }
}
